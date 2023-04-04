use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{
    Initializable, ReferenceReplacer, ResourceIdUninit, ResourceReferenceUninit, ResourceType,
    SceneNode, SceneNodeUnloaded,
};

use crate::ray::Ray;
use crate::renderer::cpu_renderer;
use crate::renderer::cpu_renderer::RayTraceResult;

pub type KdTreeUnloaded = KdTreeGeneric<String, Box<dyn SceneNodeUnloaded>>;
pub type KdTree = KdTreeGeneric<usize, Box<dyn SceneNode>>;

#[derive(Serialize, Deserialize)]
pub struct KdTreeGeneric<R, N> {
    pub path: R,
    pub child: N,
}

#[typetag::serde(name = "kd_tree")]
impl SceneNodeUnloaded for KdTreeUnloaded {
    fn collect_references(&self) -> HashSet<ResourceIdUninit> {
        let mut refs = self.child.collect_references();
        refs.insert(self.path.clone());
        refs
    }
}

impl Initializable for KdTreeUnloaded {
    type Initialized = Box<dyn SceneNode>;

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        let path_replacement = reference_replacer.get_replacement(ResourceReferenceUninit {
            ty: ResourceType::KdTree,
            path: self.path,
        });

        Box::from(KdTree {
            path: path_replacement.path,
            child: self.child.load(reference_replacer),
        })
    }
}

impl SceneNode for KdTree {}

impl cpu_renderer::SceneNode for KdTree {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        todo!()
    }
}
