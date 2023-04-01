use serde::Deserialize;

mod obj_loader;
pub mod triangle;

use triangle::{Triangle, TriangleUninit};

pub type MeshUninit = MeshGeneric<TriangleUninit>;
pub type Mesh = MeshGeneric<Triangle>;

#[derive(Deserialize)]
pub struct MeshGeneric<T> {
    triangles: Vec<T>,
}

impl MeshUninit {
    pub fn load_from_obj(file_data: &[u8]) -> MeshUninit {
        MeshUninit {
            triangles: obj_loader::load(file_data),
        }
    }

    pub fn init(self) -> Mesh {
        Mesh {
            triangles: self
                .triangles
                .into_iter()
                .map(TriangleUninit::init)
                .collect(),
        }
    }
}
