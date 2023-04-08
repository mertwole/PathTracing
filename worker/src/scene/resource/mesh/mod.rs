use std::collections::HashSet;
use std::sync::Arc;

use serde::Deserialize;

mod obj_loader;
pub mod triangle;

use crate::{
    ray::Ray,
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::{
        resource::{ReferenceReplacer, Resource, ResourceReferenceUninit},
        Scene,
    },
};
use triangle::{Triangle, TriangleUninit};

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

impl Resource for MeshUninit {
    type Initialized = Mesh;

    fn load(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        MeshUninit {
            triangles: obj_loader::load(data),
        }
    }

    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        HashSet::new()
    }

    fn init(self: Box<Self>, _: &mut dyn ReferenceReplacer) -> Self::Initialized {
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
