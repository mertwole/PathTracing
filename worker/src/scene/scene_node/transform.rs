use std::collections::HashSet;
use std::sync::Arc;

use math::{Mat3, Mat4, Vec4};
use serde::{Deserialize, Serialize};

use super::{
    ReferenceReplacer, ResourceIdUninit, ResourceReferenceUninit, SceneNode, SceneNodeUnloaded,
};

use crate::ray::Ray;
use crate::renderer::cpu_renderer;
use crate::renderer::cpu_renderer::RayTraceResult;
use crate::scene::Scene;

pub type TransformUnloaded = TransformGeneric<Box<dyn SceneNodeUnloaded>>;
pub type Transform = TransformGeneric<Box<dyn SceneNode>>;

#[derive(Deserialize, Serialize)]
pub struct TransformGeneric<R> {
    pub matrix: Mat4,
    #[serde(skip)]
    matrix_inverse: Mat4,
    #[serde(skip)]
    normal_matrix: Mat3,
    pub child: R,
}

#[typetag::serde(name = "transform")]
impl SceneNodeUnloaded for TransformUnloaded {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        self.child.collect_references()
    }

    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        Box::from(Transform {
            matrix_inverse: self.matrix.inverse(),
            normal_matrix: self.matrix.normal_matrix(),
            matrix: self.matrix,
            child: self.child.init(reference_replacer),
        })
    }
}

impl SceneNode for Transform {}

impl cpu_renderer::SceneNode for Transform {
    fn trace_ray(&self, scene: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let new_ray = ray.apply_transform(&self.matrix_inverse);
        let result = self.child.trace_ray(scene, &new_ray);

        if result.hit {
            let point = &self.matrix * Vec4::from_vec3(result.point);
            let t = (point - ray.source).length();

            RayTraceResult {
                hit: result.hit,
                hit_inside: result.hit_inside,
                point,
                normal: &self.normal_matrix * result.normal,
                uv: result.uv,
                t,
                material_id: result.material_id,
            }
        } else {
            result
        }
    }
}
