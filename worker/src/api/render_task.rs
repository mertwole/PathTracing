use serde::{Deserialize, Serialize};

use crate::camera::Camera;

#[derive(Deserialize, Serialize, Clone)]
pub struct RenderTaskUninit {
    pub scene: String,
    pub config: Config,
    pub camera: Camera,
}

impl RenderTaskUninit {
    pub fn init(self, scene_md5: String) -> RenderTask {
        RenderTask {
            scene: self.scene,
            scene_md5,
            config: self.config,
            camera: self.camera,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenderTask {
    pub scene: String,
    pub scene_md5: String,
    pub config: Config,
    pub camera: Camera,
}

impl RenderTask {
    pub fn md5(&self) -> String {
        let ser = self.scene.clone()
            + &self.scene_md5
            + &serde_json::ser::to_string(&self.config).unwrap()
            + &serde_json::ser::to_string(&self.camera).unwrap();
        format!("{:x}", md5::compute(ser))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub trace_depth: usize,
    pub iterations: usize,
}
