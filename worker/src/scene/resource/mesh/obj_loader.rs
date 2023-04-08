use std::{convert::TryInto, io::BufReader};

use tobj::{LoadError, MTLLoadResult};

use math::{Vec2, Vec3};

use super::triangle::TriangleUninit;

pub fn load(file_data: &[u8]) -> Vec<TriangleUninit> {
    // @NOTE: tobj loads uvs as [f32, f32] even if there's third texture coord.
    let mut reader = BufReader::new(file_data);
    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..tobj::OFFLINE_RENDERING_LOAD_OPTIONS
        },
        |_| MTLLoadResult::Err(LoadError::GenericFailure),
    )
    .unwrap();

    let mut triangles = vec![];
    for model in models {
        let mesh = model.mesh;

        let verts: Vec<_> = mesh
            .positions
            .chunks(3)
            .map(|pos| Vec3::new(pos[0], pos[1], pos[2]))
            .collect();

        let normals: Vec<_> = mesh
            .normals
            .chunks(3)
            .map(|pos| Vec3::new(pos[0], pos[1], pos[2]))
            .collect();

        let uvs: Vec<_> = mesh
            .texcoords
            .chunks(2)
            .map(|pos| Vec2::new(pos[0], pos[1]))
            .collect();

        let mut face_vertice_ids = mesh.indices.chunks(3);
        let mut face_normal_ids = mesh.normal_indices.chunks(3);
        let mut face_uv_ids = mesh.texcoord_indices.chunks(3);

        loop {
            let (vertices, normals, uvs) = (
                face_vertice_ids.next().map(|ids| {
                    ids.iter()
                        .map(|&id| verts[id as usize])
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                }),
                face_normal_ids.next().map(|ids| {
                    ids.iter()
                        .map(|&id| normals[id as usize])
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                }),
                face_uv_ids.next().map(|ids| {
                    ids.iter()
                        .map(|&id| uvs[id as usize])
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                }),
            );

            if let Some(vertices) = vertices {
                let triangle = TriangleUninit::new(vertices, normals, uvs);
                triangles.push(triangle);
            } else {
                break;
            }
        }
    }
    triangles
}
