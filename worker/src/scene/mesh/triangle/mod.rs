use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use math::{Mat4, Vec2, Vec3, Vec4};

mod vertex;

use vertex::{Vertex, VertexUninit};

pub type Triangle = TriangleGeneric<Vertex>;
pub type TriangleUninit = TriangleGeneric<VertexUninit>;

#[derive(Deserialize, Serialize)]
pub struct TriangleGeneric<V> {
    vertices: [V; 3],
    #[serde(skip)]
    true_normal: Vec3,
}

impl TriangleUninit {
    pub fn new(
        vertices: [Vec3; 3],
        normals: Option<[Vec3; 3]>,
        uvs: Option<[Vec2; 3]>,
    ) -> TriangleUninit {
        let normals = normals
            .map(|normals| normals.into_iter().map(|normal| Some(normal)).collect())
            .unwrap_or(vec![None; 3]);
        let uvs = uvs
            .map(|uvs| uvs.into_iter().map(|uv| Some(uv)).collect())
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
        }
    }

    pub fn transform(&mut self, matrix: &Mat4) {
        let normal_matrix = &matrix.normal_matrix();
        for vert in &mut self.vertices {
            vert.position = matrix * Vec4::from_vec3(vert.position);
            if let Some(normal) = vert.normal.as_mut() {
                *normal = (normal_matrix * *normal).normalized();
            }
        }
    }

    pub fn init(self) -> Triangle {
        let side_0 = self.vertices[1].position - self.vertices[0].position;
        let side_1 = self.vertices[1].position - self.vertices[2].position;
        let true_normal = side_0.cross(side_1).normalized();

        Triangle {
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
