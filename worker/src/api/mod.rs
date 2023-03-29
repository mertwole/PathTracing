pub mod render_task;

pub mod scene {
    pub use crate::scene::scene_node::ResourceReference;
    pub use crate::scene::scene_node::SceneNodeUnloaded as SceneRoot;
}
