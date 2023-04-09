pub mod render_task;

pub mod scene {
    pub use crate::scene::{
        resource::{
            image::Image, material::BoxedMaterial as Material, mesh::MeshUninit as Mesh, Resource,
            ResourceReferenceUninit as ResourceReference, ResourceType,
        },
        SceneHierarchyUninit as SceneHierarchy,
    };
}

pub mod render_store {
    pub use crate::render_store::RenderStore;
}
