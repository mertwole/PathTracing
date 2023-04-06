use serde::Deserialize;

mod obj_loader;
pub mod triangle;

use triangle::{Triangle, TriangleUninit};

use crate::renderer::cpu_renderer;

use crate::ray::Ray;
use crate::renderer::cpu_renderer::RayTraceResult;
use crate::scene::Scene;
use std::sync::Arc;

use super::Initializable;

pub type MeshUninit = MeshGeneric<TriangleUninit>;
pub type Mesh = MeshGeneric<Triangle>;

#[derive(Deserialize)]
pub struct MeshGeneric<T> {
    triangles: Vec<T>,
}

impl MeshUninit {
    pub fn load_from_obj(file_data: &[u8]) -> MeshUninit {
        MeshUninit {
            triangles: obj_loader::load(file_data),
        }
    }

    pub fn init(self) -> Mesh {
        Mesh {
            triangles: self
                .triangles
                .into_iter()
                .map(TriangleUninit::init)
                .collect(),
        }
    }
}

impl cpu_renderer::SceneNode for Mesh {
    fn trace_ray(&self, scene: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        result.t = f32::INFINITY;
        for triangle in &self.triangles {
            let triangle_result = triangle.trace_ray(scene.clone(), ray);
            if triangle_result.hit && triangle_result.t < result.t {
                result = triangle_result;
            }
        }
        result
    }
}
