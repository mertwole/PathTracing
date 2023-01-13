use std::iter;

use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use super::{RayTraceResult, Raytraceable, RaytraceableUninit};
use crate::ray::Ray;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Triangle {
    vertices: [Vec3; 3],
    vertice_uvs: [Vec2; 3],

    normal: Vec3,

    material_id: usize,
}

impl Triangle {
    pub fn new(vertices: [Vec3; 3], normal: Vec3, material_id: usize) -> Triangle {
        Triangle {
            vertices,
            vertice_uvs: [Vec2::default(); 3],
            normal,
            material_id,
        }
    }
}

#[typetag::serde(name = "triangle")]
impl RaytraceableUninit for Triangle {
    fn init(mut self: Box<Self>) -> Box<dyn Raytraceable> {
        self.normal = self.normal.normalized();
        self
    }
}

impl Raytraceable for Triangle {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        // Moller-Trumbore algorithm
        let edge0 = self.vertices[1] - self.vertices[0];
        let edge1 = self.vertices[2] - self.vertices[0];
        let pvec = ray.direction.cross(edge1);
        let determinant = edge0.dot(pvec);
        // If determinant < 0 => ray is tracing from the back side of triangle
        // Ray is parallel to triangle plane
        if determinant < math::EPSILON && determinant > -math::EPSILON {
            return result;
        }
        let inv_determinant = 1.0 / determinant;
        let tvec = ray.source - self.vertices[0];
        let u = tvec.dot(pvec) * inv_determinant;
        if !(0.0..=1.0).contains(&u) {
            return result;
        }
        let qvec = tvec.cross(edge0);
        let v = ray.direction.dot(qvec) * inv_determinant;
        if v < 0.0 || u + v > 1.0 {
            return result;
        }
        result.t = edge1.dot(qvec) * inv_determinant;
        if result.t < ray.min || result.t > ray.max {
            return result;
        }

        result.point = ray.source + ray.direction * result.t;

        let vectors_to_vertices = [
            self.vertices[0] - result.point,
            self.vertices[1] - result.point,
            self.vertices[2] - result.point,
        ];
        let barycentric_coords: Vec<_> = [
            vectors_to_vertices[1].cross(vectors_to_vertices[2]),
            vectors_to_vertices[0].cross(vectors_to_vertices[2]),
            vectors_to_vertices[0].cross(vectors_to_vertices[1]),
        ]
        .iter()
        .map(|coord| coord.dot(self.normal).abs())
        .collect();
        let coords_sum: f32 = barycentric_coords.iter().sum();
        let barycentric_coords = barycentric_coords
            .into_iter()
            .map(|coord| coord / coords_sum);
        result.uv = iter::zip(self.vertice_uvs, barycentric_coords)
            .into_iter()
            .map(|(coord, uv)| coord * uv)
            .fold(Vec2::default(), |acc, x| acc + x);

        result.hit = true;
        result.material_id = self.material_id;
        result.normal = self.normal;
        result
    }
}
