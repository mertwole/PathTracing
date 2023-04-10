use std::collections::{HashMap, HashSet};

use worker::api::scene::{Image, Material, Mesh, Resource, ResourceType, SceneHierarchy};

use control_panel::api::*;

struct FileReference {
    path: String,
    md5: String,
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
            .map(|loaded| FileReference {
                md5: md5s.get(&loaded).unwrap().clone(),
                path: loaded,
            })
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

    pub async fn upload_to_control_panel(&self, control_panel_url: &str, render_task_md5: &str) {
        let client = reqwest::Client::new();

        // TODO: Upload only files that haven't yet been uploaded

        for reference in &self.file_references {
            let file_data = std::fs::read(format!("./scene_data/{}", reference.path)).unwrap();
            let body = UploadFileRequest {
                name: reference.path.clone(),
                data: file_data,
            };

            client
                .post(format!("{}/scene/{}/files", control_panel_url, self.md5))
                .json(&body)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }
    }
}
