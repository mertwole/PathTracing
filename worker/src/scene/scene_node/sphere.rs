use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use math::Vec3;

use super::{
    Initializable, ReferenceReplacer, ResourceId, ResourceIdUninit, ResourceReferenceUninit,
    ResourceType, SceneNode, SceneNodeUnloaded,
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
    fn collect_references(&self) -> HashSet<ResourceIdUninit> {
        vec![self.material.clone()].into_iter().collect()
    }
}

impl Initializable for SphereUnloaded {
    type Initialized = Box<dyn SceneNode>;

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        let material_replacement = reference_replacer.get_replacement(ResourceReferenceUninit {
            ty: ResourceType::Material,
            path: self.material,
        });

        Box::from(Sphere {
            center: self.center,
            radius: self.radius,
            radius_sqr: self.radius_sqr,
            material: material_replacement.path,
        })
    }
}

impl SceneNode for Sphere {}

// #[typetag::serde(name = "sphere")]
// impl RaytraceableUninit for Sphere {
//     fn init(mut self: Box<Self>) -> Box<dyn Raytraceable> {
//         self.radius_sqr = self.radius.powi(2);
//         self
//     }
// }

// impl Raytraceable for Sphere {
//     fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
//         let mut result = RayTraceResult::void();

//         let a = self.center - ray.source;
//         //length(Direction * t + Source - Center) = radius
//         // A = center - source
//         //t^2 * dot(Direction, Direction) - 2 * t * dot(A, Direction) + dot(A, A) = Radius ^ 2
//         //Direction is normalized => dot(Direction, Direction) = 1
//         let half_second_k = -a.dot(ray.direction);
//         //Discriminant = second_k ^ 2 - 4 * first_k * third_k
//         let discriminant = 4.0 * (half_second_k * half_second_k - (a.dot(a) - self.radius_sqr));
//         if discriminant < 0.0 {
//             return result;
//         }
//         //roots are (-half_second_k * 2 +- sqrtD) / 2
//         let d_sqrt = discriminant.sqrt();
//         let t1 = -half_second_k + d_sqrt / 2.0;
//         let t2 = -half_second_k - d_sqrt / 2.0;

//         if t2 >= ray.min && t2 <= ray.max {
//             result.t = t2;
//         } else if t1 >= ray.min && t1 <= ray.max {
//             result.t = t1;
//             //if we choose max value of t it means that ray is traced from inside
//             result.hit_inside = true;
//         } else {
//             return result;
//         }

//         result.point = result.t * ray.direction + ray.source;
//         let normal_facing_outside = if result.hit_inside { -1.0 } else { 1.0 };
//         result.normal = (result.point - self.center) / (self.radius * normal_facing_outside);

//         let local_space_normal = &self.local_space_transform * result.normal;
//         let u = f32::atan2(local_space_normal.x, local_space_normal.z) / (2.0 * math::PI) + 0.5;
//         let v = f32::asin(local_space_normal.y) / math::PI + 0.5;
//         result.uv = Vec2::new(u, v);

//         result.hit = true;
//         result.material_id = self.material_id;

//         result
//     }

//     fn is_bounded(&self) -> bool {
//         true
//     }

//     fn get_bounded(self: Box<Self>) -> Option<Box<dyn Bounded>> {
//         Some(self)
//     }
// }

// impl Bounded for Sphere {
//     fn get_bounds(&self) -> AABB {
//         let r_vec = Vec3::new_xyz(self.radius);
//         AABB::new(self.center - r_vec, self.center + r_vec)
//     }

//     fn intersect_with_aabb(&self, aabb: &AABB) -> bool {
//         let closest_aabb_point = Vec3::new(
//             f32::max(aabb.min.x, f32::min(aabb.max.x, self.center.x)),
//             f32::max(aabb.min.y, f32::min(aabb.max.y, self.center.y)),
//             f32::max(aabb.min.z, f32::min(aabb.max.z, self.center.z)),
//         );
//         let distance_to_center = closest_aabb_point - self.center;
//         distance_to_center.sqr_length() <= self.radius_sqr
//     }
// }
