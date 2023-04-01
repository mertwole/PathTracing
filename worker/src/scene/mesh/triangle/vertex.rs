use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

pub type Vertex = VertexGeneric<Vec3>;
pub type VertexUninit = VertexGeneric<Option<Vec3>>;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct VertexGeneric<N> {
    pub position: Vec3,
    #[serde(default)]
    pub uv: Vec2,
    #[serde(default)]
    pub normal: N,
}
