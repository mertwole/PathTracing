use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::renderer::cpu_renderer;
use crate::renderer::cpu_renderer::RayTraceResult;
use crate::{ray::Ray, scene::Scene};

use super::{
    ReferenceReplacer, ResourceIdUninit, ResourceReferenceUninit, SceneNode, SceneNodeUnloaded,
};

pub type NodeCollectionUnloaded = NodeCollectionGeneric<Box<dyn SceneNodeUnloaded>>;
pub type NodeCollection = NodeCollectionGeneric<Box<dyn SceneNode>>;

#[derive(Deserialize, Serialize)]
pub struct NodeCollectionGeneric<R> {
    pub children: Vec<R>,
}

#[typetag::serde(name = "node_collection")]
impl SceneNodeUnloaded for NodeCollectionUnloaded {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        let mut refs = HashSet::new();
        for child in &self.children {
            refs.extend(child.collect_references());
        }
        refs
    }

    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        Box::from(NodeCollection {
            children: self
                .children
                .into_iter()
                .map(|child| child.init(reference_replacer))
                .collect(),
        })
    }
}

impl SceneNode for NodeCollection {}

impl cpu_renderer::SceneNode for NodeCollection {
    fn trace_ray(&self, scene: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        result.t = f32::INFINITY;
        for child in &self.children {
            let child_result = child.trace_ray(scene.clone(), ray);
            if child_result.hit && child_result.t < result.t {
                result = child_result;
            }
        }
        result
    }
}
