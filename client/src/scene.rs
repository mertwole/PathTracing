use std::collections::{HashMap, HashSet};

use worker::api::scene::{Image, Material, Mesh, Resource, ResourceType, SceneUninit};

pub struct Scene {
    pub md5: String,
}

impl Scene {
    pub fn load(path: &str) -> Scene {
        let absolute_path = format!("./scene_data/{}", path);
        let scene_data = std::fs::read(absolute_path).unwrap();
        let scene_md5 = format!("{:x}", md5::compute(&scene_data));
        let scene_data: SceneUninit =
            serde_json::from_str(&String::from_utf8(scene_data).unwrap()).unwrap();

        let mut staged_to_load = scene_data.root.collect_references();
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

        let mut md5s: Vec<_> = md5s.into_iter().collect();
        md5s.sort_by(|x, y| x.0.cmp(&y.0));
        let resulting_md5 = md5s
            .into_iter()
            .map(|(_, md5)| md5)
            .fold(String::new(), |acc, x| {
                format!("{:x}", md5::compute(acc + &x))
            });

        Scene { md5: resulting_md5 }
    }
}
