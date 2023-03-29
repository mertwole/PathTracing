use serde::{Deserialize, Serialize};

use crate::camera::Camera;

#[derive(Deserialize, Serialize, Clone)]
pub struct RenderTask {
    #[serde(default)]
    pub id: usize,

    pub scene: String,
    #[serde(default)]
    pub scene_md5: String,
    pub config: Config,
    pub camera: Camera,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub trace_depth: usize,
    pub iterations: usize,
}
