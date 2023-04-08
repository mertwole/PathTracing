use std::collections::HashSet;

use crate::{
    renderer::cpu_renderer,
    scene::resource::{ReferenceReplacer, ResourceReferenceUninit},
};

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
