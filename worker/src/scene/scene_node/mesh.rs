use std::collections::HashSet;

use super::{
    Initializable, ReferenceReplacer, ResourceId, ResourceIdUninit, ResourceReferenceUninit,
    ResourceType, SceneNode, SceneNodeUnloaded,
};
use serde::{Deserialize, Serialize};

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

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
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
