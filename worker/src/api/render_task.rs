use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct RenderTaskUninit {
    pub scene: String,
    pub config: Config,
}

impl RenderTaskUninit {
    pub fn init(self, scene_md5: String) -> RenderTask {
        RenderTask {
            scene: self.scene,
            scene_md5,
            config: self.config,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenderTask {
    pub scene: String,
    pub scene_md5: String,
    // TODO: Flatten.
    pub config: Config,
}

impl RenderTask {
    pub fn md5(&self) -> String {
        let mut config = self.config.clone();
        // RenderTasks are equal even if there's different iteration count in them
        config.iterations = 0;

        let ser =
            self.scene.clone() + &self.scene_md5 + &serde_json::ser::to_string(&config).unwrap();
        format!("{:x}", md5::compute(ser))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub trace_depth: usize,
    pub iterations: usize,
}
