use std::path::Path;

use math::Mat4;
use serde::Deserialize;

use crate::raytraceable::{Raytraceable, RaytraceableUninit};

mod obj_loader;

#[derive(Deserialize)]
pub struct Mesh {
    path: String,
    #[serde(default)]
    transform: Mat4,
    material_id: usize,
}

impl Mesh {
    pub fn init(self) -> Vec<Box<dyn Raytraceable>> {
        obj_loader::load(Path::new(&self.path), self.material_id)
            .into_iter()
            .map(|mut triangle| {
                triangle.transform(&self.transform);
                (Box::new(triangle) as Box<dyn RaytraceableUninit>).init()
            })
            .collect()
    }
}
