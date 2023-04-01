use std::collections::HashSet;

use super::Initializable;

pub mod kd_tree;
pub mod mesh;
pub mod node_collection;
pub mod sphere;
pub mod transform;

#[typetag::serde(tag = "type")]
pub trait SceneNodeUnloaded: Initializable<Initialized = Box<dyn SceneNode>> + Send + Sync {
    fn collect_references(&self) -> HashSet<ResourceIdUninit>;
}
pub trait SceneNode: Send + Sync {}

pub type ResourceIdUninit = String;
pub type ResourceId = usize;

pub type ResourceReferenceUninit = ResourceReferenceGeneric<ResourceIdUninit>;
pub type ResourceReference = ResourceReferenceGeneric<ResourceId>;

pub trait ReferenceReplacer {
    fn get_replacement(&mut self, reference: ResourceReferenceUninit) -> ResourceReference;
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ResourceReferenceGeneric<I> {
    pub path: I,
    pub ty: ResourceType,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    Mesh,
    Material,
    KdTree,
    Image,
}

impl ResourceType {
    pub fn get_all_variants() -> Vec<ResourceType> {
        vec![Self::Mesh, Self::Material, Self::KdTree, Self::Image]
    }
}
