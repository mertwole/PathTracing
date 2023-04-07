use std::collections::HashSet;

use futures_util::{io::AsyncWriteExt, StreamExt};
use mongodb::{
    bson::doc,
    options::{GridFsBucketOptions, GridFsUploadOptions},
    Client,
};

use worker::api::scene::{Material, Mesh, Resource, ResourceType, SceneHierarchy};

pub struct Scene {
    file_references: HashSet<String>,
    pub md5: String,
}

impl Scene {
    pub fn load(path: &str) -> Scene {
        let absolute_path = format!("./scene_data/{}", path);
        let scene_data = &std::fs::read(&absolute_path).unwrap();
        let scene_md5 = format!("{:x}", md5::compute(&scene_data));
        let scene_data = SceneHierarchy::load(&scene_data);

        let mut staged_to_load = scene_data.collect_references();
        let mut loaded = HashSet::from([path.to_string()]);

        while !staged_to_load.is_empty() {
            loaded.extend(
                staged_to_load
                    .iter()
                    .map(|res| res.path.clone())
                    .collect::<HashSet<_>>(),
            );

            staged_to_load = staged_to_load
                .into_iter()
                .map(|to_load| {
                    let absolute_path = format!("./scene_data/{}", to_load.path);
                    let data = &std::fs::read(&absolute_path).unwrap();
                    match to_load.ty {
                        ResourceType::Image => {
                            // TODO: Implement abstraction over image and use it here
                            HashSet::new()
                        }
                        ResourceType::Mesh => {
                            let mesh = Mesh::load(&data);
                            mesh.collect_references()
                        }
                        ResourceType::Material => {
                            let material = Material::load(&data);
                            material.collect_references()
                        }
                        ResourceType::KdTree => {
                            unimplemented!()
                        }
                    }
                })
                .flatten()
                .collect();
        }

        Scene {
            file_references: loaded,
            md5: scene_md5,
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

            if found_files.len() > 0 {
                assert_eq!(found_files.len(), 1);

                let found_file = found_files.pop().unwrap().expect("TODO: propagate");
                let found_file_metadata = found_file
                    .metadata
                    .expect("Extraneous file in database: Expected metadata");
                let hash = found_file_metadata
                    .get("md5")
                    .expect(&format!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5",
                        found_file_metadata
                    ))
                    .as_str()
                    .expect(&format!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5 as string",
                        found_file_metadata
                    ));

                if hash != &file_md5 {
                    bucket.delete(found_file.id).await.expect("TODO: propagate");
                } else {
                    continue;
                }
            }

            let mut upload_stream = bucket.open_upload_stream(
                &reference,
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
