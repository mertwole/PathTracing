use std::collections::{HashMap, HashSet};

use futures_util::{io::AsyncWriteExt, StreamExt};
use mongodb::{
    bson::doc,
    options::{GridFsBucketOptions, GridFsUploadOptions},
    Client,
};

use worker::api::scene::{Image, Material, Mesh, Resource, ResourceType, SceneHierarchy};

pub struct Scene {
    file_references: HashSet<String>,
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

        let mut md5s = vec![(path.to_string(), scene_md5)];

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
                    md5s.push((to_load.path.clone(), md5));
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

        md5s.sort_by(|x, y| x.0.cmp(&y.0));
        let resulting_md5 = md5s
            .into_iter()
            .map(|(_, md5)| md5)
            .fold(String::new(), |acc, x| {
                format!("{:x}", md5::compute(acc + &x))
            });

        Scene {
            file_references: loaded,
            md5: resulting_md5,
        }
    }

    pub async fn upload_to_file_store(&self, mongodb: &Client) {
        let db = mongodb.database("scene_files");
        let bucket = db.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(self.md5.clone())
                .build(),
        ));

        for reference in &self.file_references {
            let file_data = std::fs::read(format!("./scene_data/{}", reference)).unwrap();
            let file_md5 = format!("{:x}", md5::compute(&file_data));

            let mut found_files = bucket
                .find(doc! { "filename": &reference }, None)
                .await
                .expect("TODO: propagate")
                .collect::<Vec<_>>()
                .await;

            if !found_files.is_empty() {
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
                } else {
                    continue;
                }
            }

            let mut upload_stream = bucket.open_upload_stream(
                reference,
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
