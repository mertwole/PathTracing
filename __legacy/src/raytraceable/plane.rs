use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use super::{triangle::Triangle, Bounded, RayTraceResult, Raytraceable, RaytraceableUninit};
use crate::ray::Ray;

#[derive(Deserialize, Serialize)]
pub struct Plane {
    point: Vec3,
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,

    material_id: usize,
}

#[typetag::serde(name = "plane")]
impl RaytraceableUninit for Plane {
    fn init(mut self: Box<Self>) -> Box<dyn Raytraceable> {
        self.normal = self.normal.normalized();
        self.tangent = self.tangent.normalized();
        self.bitangent = self.bitangent.normalized();

        assert!(self.normal.dot(self.tangent).abs() < 0.001);
        assert!(self.normal.dot(self.bitangent).abs() < 0.001);
        assert!(self.tangent.dot(self.bitangent).abs() < 0.001);

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
        let normal_facing_dir = -ray.direction.dot(self.normal).signum();

        result.hit_inside = normal_facing_dir < 0.0;
        result.normal = self.normal * normal_facing_dir;
        let uv = result.point - self.point;
        // Project onto plane's surface.
        result.uv = Vec2::new(uv.dot(self.bitangent), uv.dot(self.tangent));
        result.t = t;
        result.material_id = self.material_id;

        result
    }

    fn is_bounded(&self) -> bool {
        false
    }

    fn get_bounded(self: Box<Self>) -> Option<Box<dyn Bounded>> {
        None
    }
}
