use std::collections::HashSet;

use crate::renderer::cpu_renderer;

pub mod kd_tree;
pub mod mesh;
pub mod node_collection;
pub mod plane;
pub mod sphere;
pub mod transform;

#[typetag::serde(tag = "type")]
pub trait SceneNodeUnloaded: Send + Sync {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit>;
    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode>;
}
pub trait SceneNode: cpu_renderer::SceneNode {}

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
