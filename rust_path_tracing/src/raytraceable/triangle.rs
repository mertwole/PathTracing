use super::Raytraceable;
use crate::math::*;
use crate::ray::*;

pub struct Triangle {
    points: [Vec3; 3],
    normal: Vec3,

    material_id : usize
}

impl Triangle {
    pub fn new(points: [Vec3; 3], normal: Vec3, material_id : usize) -> Triangle {
        Triangle { points, normal, material_id }
    }
}

impl Raytraceable for Triangle {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        // Moller-Trumbore algorithm
        let edge0 = &self.points[1] - &self.points[0];
        let edge1 = &self.points[2] - &self.points[0];
        let pvec = ray.direction.cross(&edge1);
        let determinant = edge0.dot(&pvec);
        // If determinant < 0 => ray is tracing from the back side of triangle
        // Ray is parallel to triangle plane
        if determinant < math::EPSILON && determinant > -math::EPSILON { return result; }
        let inv_determinant = 1.0 / determinant;
        let tvec = &ray.source - &self.points[0];
        let u = tvec.dot(&pvec) * inv_determinant;
        if u < 0.0 || u > 1.0 { return result; }
        let qvec = tvec.cross(&edge0);
        let v = ray.direction.dot(&qvec) * inv_determinant;
        if v < 0.0 || u + v > 1.0 { return result; }
        result.t = edge1.dot(&qvec) * inv_determinant;
        if result.t < ray.min || result.t > ray.max { return result; }
        result.point = &ray.source + &(&ray.direction * result.t);
        result.hit = true;
        result.material_id = self.material_id;
        result.normal = self.normal;      
        result
    }
}
