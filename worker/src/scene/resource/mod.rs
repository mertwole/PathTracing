use std::{
    collections::{HashMap, HashSet},
    iter,
};

pub mod image;
pub mod material;
pub mod mesh;

pub trait Resource {
    type Initialized;

    fn load(data: &[u8]) -> Self
    where
        Self: Sized;
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit>;
    fn init(self, reference_replacer: &mut dyn ReferenceReplacer) -> Self::Initialized;
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

    fn get_pending_processing(&mut self) -> Vec<(ResourceIdUninit, ResourceId)> {
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
            .map(|(uninit, init)| (uninit.clone(), *init))
            .collect()
    }
}

pub struct ReferenceMapping {
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
    pub fn get_pending_processing(&mut self) -> Vec<(ResourceType, ResourceIdUninit, ResourceId)> {
        self.references
            .iter_mut()
            .flat_map(|(ty, ref_collection)| {
                iter::repeat(ty)
                    .cloned()
                    .zip(ref_collection.get_pending_processing().into_iter())
                    .map(|(ty, (uninit, init))| (ty, uninit, init))
                    .collect::<Vec<(_, _, _)>>()
            })
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
            ty: reference.ty,
            path: refs.get_id_or_insert(reference),
        }
    }
}

pub type ResourceIdUninit = String;
pub type ResourceId = usize;

pub type ResourceReferenceUninit = ResourceReferenceGeneric<ResourceIdUninit>;
pub type ResourceReference = ResourceReferenceGeneric<ResourceId>;

pub trait ReferenceReplacer {
    fn get_replacement(&mut self, reference: ResourceReferenceUninit) -> ResourceReference;
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ResourceReferenceGeneric<I> {
    pub path: I,
    pub ty: ResourceType,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    Mesh,
    Material,
    KdTree,
    Image,
}

impl ResourceType {
    pub fn get_all_variants() -> Vec<ResourceType> {
        vec![Self::Mesh, Self::Material, Self::KdTree, Self::Image]
    }
}
