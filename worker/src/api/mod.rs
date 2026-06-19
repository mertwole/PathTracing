pub mod render_task;

pub mod scene {
    pub use crate::scene::{
        SceneRootUninit, SceneUninit,
        resource::{
            Resource, ResourceReferenceUninit as ResourceReference, ResourceType, image::Image,
            material::BoxedMaterial as Material, mesh::MeshUninit as Mesh,
        },
    };
}
