use std::collections::{HashMap, HashSet};

use futures::{AsyncWriteExt, stream::StreamExt};
use mongodb::{
    bson::doc,
    options::{GridFsBucketOptions, GridFsUploadOptions},
};
use worker::api::scene::{Image, Material, Mesh, Resource, ResourceType, SceneHierarchy};

struct FileReference {
    path: String,
}

pub struct Scene {
    file_references: Vec<FileReference>,
    pub md5: String,
}

impl Scene {
    pub fn load(path: &str) -> Scene {
        let absolute_path = format!("./scene_data/{}", path);
        let scene_data = &std::fs::read(absolute_path).unwrap();
        let scene_md5 = format!("{:x}", md5::compute(scene_data));
        let scene_data = SceneHierarchy::load(scene_data);

        let mut staged_to_load = scene_data.collect_references();
        let mut loaded = HashSet::from([path.to_string()]);

        let mut md5s = HashMap::from([(path.to_string(), scene_md5)]);

        while !staged_to_load.is_empty() {
            loaded.extend(
                staged_to_load
                    .iter()
                    .map(|res| res.path.clone())
                    .collect::<HashSet<_>>(),
            );

            staged_to_load = staged_to_load
                .into_iter()
                .flat_map(|to_load| {
                    let absolute_path = format!("./scene_data/{}", to_load.path);
                    let data = &std::fs::read(absolute_path).unwrap();
                    let md5 = format!("{:x}", md5::compute(data));
                    md5s.insert(to_load.path.clone(), md5);
                    match to_load.ty {
                        ResourceType::Image => {
                            let image = Image::load(data);
                            image.collect_references()
                        }
                        ResourceType::Mesh => {
                            let mesh = Mesh::load(data);
                            mesh.collect_references()
                        }
                        ResourceType::Material => {
                            let material = Material::load(data);
                            material.collect_references()
                        }
                        ResourceType::KdTree => {
                            unimplemented!()
                        }
                    }
                })
                .collect();
        }

        let file_references = loaded
            .into_iter()
            .map(|loaded| FileReference { path: loaded })
            .collect();

        let mut md5s: Vec<_> = md5s.into_iter().collect();
        md5s.sort_by(|x, y| x.0.cmp(&y.0));
        let resulting_md5 = md5s
            .into_iter()
            .map(|(_, md5)| md5)
            .fold(String::new(), |acc, x| {
                format!("{:x}", md5::compute(acc + &x))
            });

        Scene {
            file_references,
            md5: resulting_md5,
        }
    }

    pub async fn upload_to_mongodb(&self, mongodb_url: &str) {
        let mongodb_options = mongodb::options::ClientOptions::parse(mongodb_url)
            .await
            .unwrap();
        let mongodb = mongodb::Client::with_options(mongodb_options).unwrap();

        let database = mongodb.database("scene_files");
        let bucket = database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(self.md5.clone())
                .build(),
        ));

        // TODO: Upload only files that haven't yet been uploaded

        for reference in &self.file_references {
            let file_data = std::fs::read(format!("./scene_data/{}", reference.path)).unwrap();
            let file_md5 = format!("{:x}", md5::compute(&file_data));

            let mut found_files = bucket
                .find(doc! { "filename": &reference.path }, None)
                .await
                .expect("TODO: propagate")
                .collect::<Vec<_>>()
                .await;

            let upload_file = if !found_files.is_empty() {
                assert_eq!(found_files.len(), 1);

                let found_file = found_files.pop().unwrap().expect("TODO: propagate");
                let found_file_metadata = found_file
                    .metadata
                    .expect("Extraneous file in database: Expected metadata");
                let hash = found_file_metadata
                    .get("md5")
                    .unwrap_or_else(|| panic!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5",
                        found_file_metadata
                    ))
                    .as_str()
                    .unwrap_or_else(|| panic!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5 as string",
                        found_file_metadata
                    ));

                if hash != file_md5 {
                    bucket.delete(found_file.id).await.expect("TODO: propagate");
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if upload_file {
                let mut upload_stream = bucket.open_upload_stream(
                    reference.path.clone(),
                    Some(
                        GridFsUploadOptions::builder()
                            .metadata(Some(doc! { "md5": file_md5 }))
                            .build(),
                    ),
                );

                upload_stream
                    .write_all(&file_data)
                    .await
                    .expect("TODO: propagate");
                upload_stream.close().await.expect("TODO: propagate");
            }
        }
    }
}
