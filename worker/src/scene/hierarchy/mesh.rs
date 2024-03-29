use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use super::{ReferenceReplacer, ResourceReferenceUninit, SceneNode, SceneNodeUnloaded};
use crate::{
    ray::Ray,
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::{
        resource::{ResourceId, ResourceIdUninit, ResourceType},
        Scene,
    },
};

pub type MeshUnloaded = MeshGeneric<ResourceIdUninit>;
pub type Mesh = MeshGeneric<ResourceId>;

#[derive(Serialize, Deserialize)]
pub struct MeshGeneric<R> {
    pub path: R,
    pub material: R,
}

#[typetag::serde(name = "mesh")]
impl SceneNodeUnloaded for MeshUnloaded {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        vec![
            ResourceReferenceUninit {
                path: self.path.clone(),
                ty: ResourceType::Mesh,
            },
            ResourceReferenceUninit {
                path: self.material.clone(),
                ty: ResourceType::Material,
            },
        ]
        .into_iter()
        .collect()
    }

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
