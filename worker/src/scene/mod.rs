use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

pub mod hierarchy;
pub mod resource;

use crate::{camera::Camera, file_fetcher::FileFetcher};
use resource::ReferenceMapping;
use serde::{Deserialize, Serialize};

use self::{
    hierarchy::{SceneNode, SceneNodeUnloaded},
    resource::{
        ReferenceReplacer, Resource, ResourceReferenceUninit, ResourceType,
        image::Image,
        material::{BoxedMaterial, Material},
        mesh::{Mesh, MeshUninit},
    },
};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneRootUninit(Box<dyn SceneNodeUnloaded>);

pub struct SceneRoot(Box<dyn SceneNode>);

impl Deref for SceneRoot {
    type Target = Box<dyn SceneNode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resource for SceneRootUninit {
    type Initialized = SceneRoot;

    fn load(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let data = String::from_utf8(data.to_vec()).unwrap();
        SceneRootUninit(serde_json::de::from_str(&data).unwrap())
    }

    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        self.0.collect_references()
    }

    fn init(self, reference_replacer: &mut dyn ReferenceReplacer) -> Self::Initialized {
        SceneRoot(self.0.init(reference_replacer))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SceneUninit {
    pub root: SceneRootUninit,
    pub camera: Camera,
}

pub struct Scene {
    pub hierarchy: SceneRoot,
    pub camera: Camera,

    pub materials: Vec<Box<dyn Material>>,
    pub meshes: Vec<Mesh>,
    pub images: Vec<Image>,
}

impl Scene {
    pub async fn load(file_fetcher: &impl FileFetcher, scene_path: &str) -> Scene {
        let scene_data = file_fetcher.fetch(scene_path).await;
        let scene_data = String::from_utf8(scene_data).unwrap();

        let SceneUninit { root, camera } = serde_json::from_str(&scene_data).unwrap();
        let mut references = ReferenceMapping::default();
        let root = root.init(&mut references);

        let mut loaded_materials: HashMap<usize, Box<dyn Material>> = HashMap::new();
        let mut loaded_meshes: HashMap<usize, Mesh> = HashMap::new();
        let mut loaded_images: HashMap<usize, Image> = HashMap::new();

        let mut scene = Scene {
            hierarchy: root,
            camera,
            materials: Vec::new(),
            meshes: Vec::new(),
            images: Vec::new(),
        };

        loop {
            let pending_processing: Vec<_> = references.get_pending_processing();
            if pending_processing.is_empty() {
                break;
            }

            // TODO: Generalize?
            for (resource_type, uninit_ref, init_ref) in pending_processing {
                let file_data = file_fetcher.fetch(&uninit_ref).await;

                match resource_type {
                    ResourceType::Mesh => {
                        let mesh = MeshUninit::load(&file_data).init(&mut references);
                        loaded_meshes.insert(init_ref, mesh);
                    }
                    ResourceType::Material => {
                        let material = BoxedMaterial::load(&file_data).init(&mut references);
                        loaded_materials.insert(init_ref, material);
                    }
                    ResourceType::Image => {
                        let image = Image::load(&file_data);
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
