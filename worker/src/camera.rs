use serde::{Deserialize, Serialize};

use math::{Mat3, UVec2, Vec3};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BokehShape {
    Point,
    Circle,
    Square,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Camera {
    pub resolution: UVec2,
    pub rotation: Mat3,
    pub position: Vec3,

    #[serde(rename = "field_of_view")]
    pub fov: f32,
    pub near_plane: f32,
    pub focal_length: f32,

    pub bokeh_shape: BokehShape,
    pub bokeh_size: f32,
}
