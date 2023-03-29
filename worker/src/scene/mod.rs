pub mod scene_node;

use std::collections::HashMap;
use std::iter;

use crate::material::MaterialUninit;
use crate::scene::scene_node::ResourceType;
use crate::{file_store::FileStore, material::Material};

use self::scene_node::{
    ReferenceReplacer, ResourceId, ResourceIdUninit, ResourceReference, ResourceReferenceUninit,
    SceneNode, SceneNodeUnloaded,
};

pub trait Initializable {
    type Initialized;

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Self::Initialized;
}

#[derive(Default)]
struct ReferenceCollection {
    references: HashMap<ResourceIdUninit, ResourceId>,
    next_id: ResourceId,
    last_processed_id: Option<ResourceId>,
}

impl ReferenceCollection {
    fn get_id_or_insert(&mut self, reference: ResourceReferenceUninit) -> ResourceId {
        if !self.references.contains_key(&reference.path) {
            self.references.insert(reference.path.clone(), self.next_id);
            self.next_id += 1;
        }
        self.references[&reference.path]
    }

    fn get_pending_processing(&mut self) -> Vec<ResourceIdUninit> {
        let last_processed = self.last_processed_id;
        if self.next_id != 0 {
            self.last_processed_id = Some(self.next_id - 1);
        }

        self.references
            .iter()
            .filter(|(_, init)| {
                if let Some(last_processed) = last_processed {
                    **init > last_processed
                } else {
                    true
                }
            })
            .map(|(uninit, _)| uninit)
            .cloned()
            .collect()
    }
}

struct ReferenceMapping {
    references: HashMap<ResourceType, ReferenceCollection>,
}

impl Default for ReferenceMapping {
    fn default() -> ReferenceMapping {
        let mut references = HashMap::new();
        for ty in ResourceType::get_all_variants() {
            references.insert(ty, ReferenceCollection::default());
        }
        ReferenceMapping { references }
    }
}

impl ReferenceMapping {
    pub fn get_pending_processing(&mut self) -> Vec<(ResourceType, ResourceIdUninit)> {
        self.references
            .iter_mut()
            .map(|(ty, ref_collection)| {
                iter::repeat(ty)
                    .cloned()
                    .zip(ref_collection.get_pending_processing().into_iter())
                    .collect::<Vec<(_, _)>>()
            })
            .flatten()
            .collect()
    }
}

impl ReferenceReplacer for ReferenceMapping {
    fn get_replacement(&mut self, reference: ResourceReferenceUninit) -> ResourceReference {
        let refs = self
            .references
            .get_mut(&reference.ty)
            .unwrap_or_else(|| panic!("Unknown resource type"));

        ResourceReference {
            ty: reference.ty.clone(),
            path: refs.get_id_or_insert(reference),
        }
    }
}

pub struct Scene {
    hierarchy: Box<dyn SceneNode>,
    materials: Vec<Box<dyn Material>>,
}

impl Scene {
    fn new(hierarchy: Box<dyn SceneNode>) -> Scene {
        Scene {
            hierarchy,
            materials: vec![],
        }
    }

    pub async fn load(file_store: &FileStore, scene_path: &str) -> Scene {
        let scene_data = file_store.fetch_file(&scene_path).await;
        let scene_data = String::from_utf8(scene_data).unwrap();

        let hierarchy: Box<dyn SceneNodeUnloaded> = serde_json::de::from_str(&scene_data).unwrap();
        let mut references = ReferenceMapping::default();
        let hierarchy = hierarchy.load(&mut references);

        let mut scene = Scene::new(hierarchy);

        loop {
            let pending_processing: Vec<_> = references.get_pending_processing();
            if pending_processing.is_empty() {
                break;
            }

            for (resource_type, reference) in pending_processing {
                match resource_type {
                    ResourceType::Mesh => {
                        todo!()
                    }
                    ResourceType::Material => {
                        let material_data = file_store.fetch_file(&reference).await;
                        let material_data = String::from_utf8(material_data)
                            .expect("Invalid data fetched from FileStore");
                        let material: Box<dyn MaterialUninit> =
                            serde_json::de::from_str(&material_data).unwrap();
                        let material = material.init(&mut references);
                        scene.materials.push(material);
                    }
                    ResourceType::Texture => {
                        todo!()
                    }
                    ResourceType::KdTree => {
                        todo!()
                    }
                }
            }
        }

        scene
    }
}
