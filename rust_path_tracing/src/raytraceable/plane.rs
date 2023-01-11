use serde::{Deserialize, Serialize};

use math::Vec3;

use super::{RayTraceResult, Raytraceable, RaytraceableUninit};
use crate::ray::Ray;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Plane {
    point: Vec3,
    normal: Vec3,

    material_id: usize,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material_id: usize) -> Plane {
        Plane {
            point,
            normal: normal.normalized(),
            material_id,
        }
    }
}

#[typetag::serde(name = "plane")]
impl RaytraceableUninit for Plane {
    fn init(mut self: Box<Self>) -> Box<dyn Raytraceable> {
        self.normal = self.normal.normalized();
        self
    }
}

impl Raytraceable for Plane {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();

        //plane equality:
        //Nx(x - x0) + Ny(y - y0) + Nz(z - z0) = 0
        //where N - normal vector to plane
        //V[0](x0, y0, z0) - any point on this plane
        //point on ray = t * Direction + source
        //   =>
        //t = Dot(N, V[0] - Source) / Dot(N, Direction)
        //Dot(N, Direction) == 0 when Normal is perpendicular to direction => Direction parrallel to plane
        let t = self.normal.dot(self.point - ray.source) / self.normal.dot(ray.direction);

        if t < ray.min || t > ray.max {
            return result;
        }

        result.hit = true;
        result.point = ray.source + ray.direction * t;
        let normal_facing_dir = if ray.direction.dot(self.normal) > 0.0 {
            -1.0
        } else {
            1.0
        };
        result.hit_inside = normal_facing_dir < 0.0;
        result.normal = self.normal * normal_facing_dir;
        result.t = t;
        result.material_id = self.material_id;

        result
    }
}
