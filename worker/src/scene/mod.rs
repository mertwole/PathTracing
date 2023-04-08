use std::collections::{HashMap, HashSet};

use image::RgbaImage;

pub mod hierarchy;
pub mod resource;

use crate::file_store::FileStore;
use resource::ReferenceMapping;

use self::{
    hierarchy::{SceneNode, SceneNodeUnloaded},
    resource::{
        material::{Material, MaterialUninit},
        mesh::{Mesh, MeshUninit},
        ReferenceReplacer, Resource, ResourceReferenceUninit, ResourceType,
    },
};

impl Resource for SceneHierarchyUninit {
    type Initialized = Box<dyn SceneNode>;

    fn load(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let data = String::from_utf8(data.to_vec()).unwrap();
        SceneHierarchyUninit(serde_json::de::from_str(&data).unwrap())
    }

    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        self.0.collect_references()
    }

    fn init(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Self::Initialized {
        self.0.init(reference_replacer)
    }
}

pub struct SceneHierarchyUninit(Box<dyn SceneNodeUnloaded>);

pub struct Scene {
    pub hierarchy: Box<dyn SceneNode>,
    pub materials: Vec<Box<dyn Material>>,
    pub meshes: Vec<Mesh>,
    pub images: Vec<RgbaImage>,
}

impl Scene {
    fn new(hierarchy: Box<dyn SceneNode>) -> Scene {
        Scene {
            hierarchy,
            materials: vec![],
            meshes: vec![],
            images: vec![],
        }
    }

    pub async fn load(file_store: &FileStore, scene_path: &str) -> Scene {
        let scene_data = file_store.fetch_file(scene_path).await;
        let scene_data = String::from_utf8(scene_data).unwrap();

        let hierarchy: Box<dyn SceneNodeUnloaded> = serde_json::de::from_str(&scene_data).unwrap();
        let mut references = ReferenceMapping::default();
        let hierarchy = hierarchy.init(&mut references);

        let mut loaded_materials: HashMap<usize, Box<dyn Material>> = HashMap::new();
        let mut loaded_meshes: HashMap<usize, Mesh> = HashMap::new();
        let mut loaded_images: HashMap<usize, RgbaImage> = HashMap::new();

        let mut scene = Scene::new(hierarchy);
        loop {
            let pending_processing: Vec<_> = references.get_pending_processing();
            if pending_processing.is_empty() {
                break;
            }

            // TODO: Generalize?
            for (resource_type, uninit_ref, init_ref) in pending_processing {
                let file_data = file_store.fetch_file(&uninit_ref).await;

                match resource_type {
                    ResourceType::Mesh => {
                        let mesh = MeshUninit::load_from_obj(&file_data).init();
                        loaded_meshes.insert(init_ref, mesh);
                    }
                    ResourceType::Material => {
                        let material_data = String::from_utf8(file_data)
                            .expect("Invalid data fetched from FileStore");
                        let material: Box<dyn MaterialUninit> =
                            serde_json::de::from_str(&material_data).unwrap();
                        let material = material.init(&mut references);
                        loaded_materials.insert(init_ref, material);
                    }
                    ResourceType::Image => {
                        let image = image::load_from_memory(&file_data)
                            .expect("Incorrect image format fetched from FileStore")
                            .to_rgba8();
                        loaded_images.insert(init_ref, image);
                    }
                    ResourceType::KdTree => {
                        todo!()
                    }
                }
            }
        }

        for id in 0..loaded_materials.len() {
            scene.materials.push(loaded_materials.remove(&id).unwrap());
        }
        for id in 0..loaded_meshes.len() {
            scene.meshes.push(loaded_meshes.remove(&id).unwrap());
        }
        for id in 0..loaded_images.len() {
            scene.images.push(loaded_images.remove(&id).unwrap());
        }

        scene
    }
}
