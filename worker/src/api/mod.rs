pub mod render_task;

pub mod scene {
    pub use crate::scene::material::BoxedMaterial as Material;
    pub use crate::scene::mesh::MeshUninit as Mesh;

    pub use crate::scene::scene_node::ResourceType;
    pub use crate::scene::Resource;

    pub use crate::scene::scene_node::ResourceReferenceUninit as ResourceReference;
    pub use crate::scene::SceneHierarchyUninit as SceneHierarchy;
}
