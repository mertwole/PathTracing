use std::collections::HashSet;

use math::Mat4;
use serde::{Deserialize, Serialize};

use super::{Initializable, ReferenceReplacer, ResourceIdUninit, SceneNode, SceneNodeUnloaded};

use crate::ray::Ray;
use crate::renderer::cpu_renderer;
use crate::renderer::cpu_renderer::RayTraceResult;

pub type TransformUnloaded = TransformGeneric<Box<dyn SceneNodeUnloaded>>;
pub type Transform = TransformGeneric<Box<dyn SceneNode>>;

#[derive(Deserialize, Serialize)]
pub struct TransformGeneric<R> {
    pub matrix: Mat4,
    pub child: R,
}

#[typetag::serde(name = "transform")]
impl SceneNodeUnloaded for TransformUnloaded {
    fn collect_references(&self) -> HashSet<ResourceIdUninit> {
        self.child.collect_references()
    }
}

impl Initializable for TransformUnloaded {
    type Initialized = Box<dyn SceneNode>;

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        Box::from(Transform {
            matrix: self.matrix,
            child: self.child.load(reference_replacer),
        })
    }
}

impl SceneNode for Transform {}

impl cpu_renderer::SceneNode for Transform {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        todo!()
    }
}
