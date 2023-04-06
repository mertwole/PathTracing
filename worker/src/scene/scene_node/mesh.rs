use std::collections::HashSet;
use std::sync::Arc;

use super::{
    Initializable, ReferenceReplacer, ResourceId, ResourceIdUninit, ResourceReferenceUninit,
    ResourceType, SceneNode, SceneNodeUnloaded,
};
use serde::{Deserialize, Serialize};

use crate::ray::Ray;
use crate::renderer::cpu_renderer;
use crate::renderer::cpu_renderer::RayTraceResult;
use crate::scene::Scene;

pub type MeshUnloaded = MeshGeneric<ResourceIdUninit>;
pub type Mesh = MeshGeneric<ResourceId>;

#[derive(Serialize, Deserialize)]
pub struct MeshGeneric<R> {
    pub path: R,
    pub material: R,
}

#[typetag::serde(name = "mesh")]
impl SceneNodeUnloaded for MeshUnloaded {
    fn collect_references(&self) -> HashSet<ResourceIdUninit> {
        vec![self.path.clone(), self.material.clone()]
            .into_iter()
            .collect()
    }
}

impl Initializable for MeshUnloaded {
    type Initialized = Box<dyn SceneNode>;

    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        let material_replacement = reference_replacer.get_replacement(ResourceReferenceUninit {
            ty: ResourceType::Material,
            path: self.material,
        });

        let path_replacement = reference_replacer.get_replacement(ResourceReferenceUninit {
            ty: ResourceType::Mesh,
            path: self.path,
        });

        Box::from(Mesh {
            path: path_replacement.path,
            material: material_replacement.path,
        })
    }
}

impl SceneNode for Mesh {}

impl cpu_renderer::SceneNode for Mesh {
    fn trace_ray(&self, scene: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let mut result = scene.meshes[self.path].trace_ray(scene.clone(), ray);
        result.material_id = self.material;
        result
    }
}
