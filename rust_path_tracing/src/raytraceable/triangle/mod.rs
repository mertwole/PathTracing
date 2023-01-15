use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use super::{Bounded, RayTraceResult, Raytraceable, RaytraceableUninit, AABB};
use crate::ray::Ray;

mod vertex;

use vertex::{Vertex, VertexUninit};

pub type Triangle = TriangleGeneric<Vertex>;
pub type TriangleUninit = TriangleGeneric<VertexUninit>;

#[derive(Deserialize, Serialize)]
pub struct TriangleGeneric<V> {
    vertices: [V; 3],
    #[serde(skip)]
    true_normal: Vec3,
    material_id: usize,
}

impl TriangleUninit {
    pub fn new(
        vertices: [Vec3; 3],
        normals: Option<[Vec3; 3]>,
        uvs: Option<[Vec2; 3]>,
        material_id: usize,
    ) -> TriangleUninit {
        let normals = normals
            .map(|normals| normals.into_iter().map(|normal| Some(*normal)).collect())
            .unwrap_or(vec![None; 3]);
        let uvs = uvs
            .map(|uvs| uvs.into_iter().map(|uv| Some(*uv)).collect())
            .unwrap_or(vec![None; 3]);

        let vertices = itertools::izip!(vertices, normals, uvs)
            .map(|(position, normal, uv)| VertexUninit {
                position,
                uv: uv.unwrap_or_default(),
                normal,
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        TriangleUninit {
            vertices,
            true_normal: Vec3::default(),
            material_id,
        }
    }
}

#[derive(Clone, Copy)]
struct BarycentricCoords([f32; 3]);

impl Triangle {
    fn get_barycentric_coords(&self, point: Vec3) -> BarycentricCoords {
        let vectors_to_vertices = [
            self.vertices[0].position - point,
            self.vertices[1].position - point,
            self.vertices[2].position - point,
        ];
        let barycentric_coords: Vec<_> = [
            vectors_to_vertices[1].cross(vectors_to_vertices[2]),
            vectors_to_vertices[0].cross(vectors_to_vertices[2]),
            vectors_to_vertices[0].cross(vectors_to_vertices[1]),
        ]
        .iter()
        .map(|coord| coord.dot(self.true_normal).abs())
        .collect();
        let coords_sum: f32 = barycentric_coords.iter().sum();
        BarycentricCoords(
            barycentric_coords
                .into_iter()
                .map(|coord| coord / coords_sum)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn get_uv(&self, coords: BarycentricCoords) -> Vec2 {
        self.vertices[0].uv * coords.0[0]
            + self.vertices[1].uv * coords.0[1]
            + self.vertices[2].uv * coords.0[2]
    }

    fn get_normal(&self, coords: BarycentricCoords) -> Vec3 {
        self.vertices[0].normal * coords.0[0]
            + self.vertices[1].normal * coords.0[1]
            + self.vertices[2].normal * coords.0[2]
    }
}

#[typetag::serde(name = "triangle")]
impl RaytraceableUninit for TriangleUninit {
    fn init(self: Box<Self>) -> Box<dyn Raytraceable> {
        let side_0 = self.vertices[1].position - self.vertices[0].position;
        let side_1 = self.vertices[1].position - self.vertices[2].position;
        let true_normal = side_0.cross(side_1).normalized();

        Box::new(Triangle {
            vertices: self
                .vertices
                .iter()
                .map(|vert| Vertex {
                    position: vert.position,
                    uv: vert.uv,
                    normal: vert.normal.unwrap_or(true_normal),
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            true_normal,
            material_id: self.material_id,
        })
    }
}

impl Raytraceable for Triangle {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        // Moller-Trumbore algorithm
        let edge0 = self.vertices[1].position - self.vertices[0].position;
        let edge1 = self.vertices[2].position - self.vertices[0].position;
        let pvec = ray.direction.cross(edge1);
        let determinant = edge0.dot(pvec);
        // If determinant < 0 => ray is tracing from the back side of triangle
        // Ray is parallel to triangle plane
        if determinant < math::EPSILON && determinant > -math::EPSILON {
            return result;
        }
        let inv_determinant = 1.0 / determinant;
        let tvec = ray.source - self.vertices[0].position;
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
        result.hit = true;
        result.material_id = self.material_id;

        let barycentric_coords = self.get_barycentric_coords(result.point);
        result.uv = self.get_uv(barycentric_coords);
        result.normal = self.get_normal(barycentric_coords);

        result
    }

    fn is_bounded(&self) -> bool {
        true
    }

    fn get_bounded(self: Box<Self>) -> Option<Box<dyn Bounded>> {
        Some(self)
    }
}

impl Bounded for Triangle {
    fn get_bounds(&self) -> AABB {
        let mut min = self.vertices[0].position;
        let mut max = self.vertices[0].position;

        for i in 1..=2 {
            let pos = self.vertices[i].position;
            min.x = f32::min(min.x, pos.x);
            max.x = f32::max(max.x, pos.x);

            min.y = f32::min(min.y, pos.y);
            max.y = f32::max(max.y, pos.y);

            min.z = f32::min(min.z, pos.z);
            max.z = f32::max(max.z, pos.z);
        }

        AABB::new(min, max)
    }

    fn intersect_with_aabb(&self, aabb: &AABB) -> bool {
        let mut separating_plane_found = true;
        'outer: for aabb_side_normal in vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ] {
            let (min, max) = (
                aabb_side_normal.dot(aabb.min),
                aabb_side_normal.dot(aabb.max),
            );

            let projected_verts = self
                .vertices
                .iter()
                .map(|vert| vert.position.dot(aabb_side_normal));
            let mut triangle_side = 0;

            for projected_vert in projected_verts {
                let vert_side = match () {
                    () if projected_vert < min => -1,
                    () if projected_vert > max => 1,
                    _ => {
                        separating_plane_found = false;
                        break 'outer;
                    }
                };

                if triangle_side == 0 {
                    triangle_side = vert_side;
                } else if triangle_side != vert_side {
                    separating_plane_found = false;
                    break 'outer;
                }
            }
        }

        if separating_plane_found {
            return false;
        }

        let aabb_vertices: [Vec3; 8] = [
            aabb.min,
            Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            aabb.max,
        ];

        let vert_pos = self.vertices[0].position;
        let aabb_side = (aabb_vertices[0] - vert_pos).dot(self.true_normal);
        for &aabb_vert in &aabb_vertices[1..] {
            let side = (aabb_vert - vert_pos).dot(self.true_normal);
            if side * aabb_side < 0.0 {
                return true;
            }
        }

        false
    }
}
