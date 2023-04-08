use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use super::{ReferenceReplacer, ResourceReferenceUninit, SceneNode, SceneNodeUnloaded};
use crate::{
    ray::Ray,
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::{
        resource::{ResourceId, ResourceIdUninit, ResourceType},
        Scene,
    },
};

pub type SphereUnloaded = SphereGeneric<ResourceIdUninit>;
pub type Sphere = SphereGeneric<ResourceId>;

#[derive(Deserialize, Serialize)]
pub struct SphereGeneric<R> {
    pub center: Vec3,

    pub radius: f32,
    #[serde(skip)]
    radius_sqr: f32,

    pub material: R,
}

#[typetag::serde(name = "sphere")]
impl SceneNodeUnloaded for SphereUnloaded {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        vec![ResourceReferenceUninit {
            path: self.material.clone(),
            ty: ResourceType::Material,
        }]
        .into_iter()
        .collect()
    }

    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        let material_replacement = reference_replacer.get_replacement(ResourceReferenceUninit {
            ty: ResourceType::Material,
            path: self.material,
        });

        Box::from(Sphere {
            center: self.center,
            radius: self.radius,
            radius_sqr: self.radius * self.radius,
            material: material_replacement.path,
        })
    }
}

impl SceneNode for Sphere {}

impl cpu_renderer::SceneNode for Sphere {
    fn trace_ray(&self, _: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();

        let a = self.center - ray.source;
        //length(Direction * t + Source - Center) = radius
        // A = center - source
        //t^2 * dot(Direction, Direction) - 2 * t * dot(A, Direction) + dot(A, A) = Radius ^ 2
        //Direction is normalized => dot(Direction, Direction) = 1
        let half_second_k = -a.dot(ray.direction);
        //Discriminant = second_k ^ 2 - 4 * first_k * third_k
        let discriminant = 4.0 * (half_second_k * half_second_k - (a.dot(a) - self.radius_sqr));
        if discriminant < 0.0 {
            return result;
        }
        //roots are (-half_second_k * 2 +- sqrtD) / 2
        let d_sqrt = discriminant.sqrt();
        let t1 = -half_second_k + d_sqrt / 2.0;
        let t2 = -half_second_k - d_sqrt / 2.0;

        if t2 >= ray.min && t2 <= ray.max {
            result.t = t2;
        } else if t1 >= ray.min && t1 <= ray.max {
            result.t = t1;
            //if we choose max value of t it means that ray is traced from inside
            result.hit_inside = true;
        } else {
            return result;
        }

        result.point = result.t * ray.direction + ray.source;
        let normal_facing_outside = if result.hit_inside { -1.0 } else { 1.0 };
        result.normal = (result.point - self.center) / (self.radius * normal_facing_outside);

        let u = f32::atan2(result.normal.x, result.normal.z) / (2.0 * math::PI) + 0.5;
        let v = f32::asin(result.normal.y) / math::PI + 0.5;
        result.uv = Vec2::new(u, v);

        result.hit = true;
        result.material_id = self.material;

        result
    }
}
