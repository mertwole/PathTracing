use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::scene::Initializable;

use super::{ReferenceReplacer, ResourceIdUninit, SceneNode, SceneNodeUnloaded};

pub type NodeCollectionUnloaded = NodeCollectionGeneric<Box<dyn SceneNodeUnloaded>>;
pub type NodeCollection = NodeCollectionGeneric<Box<dyn SceneNode>>;

#[derive(Deserialize, Serialize)]
pub struct NodeCollectionGeneric<R> {
    pub children: Vec<R>,
}

#[typetag::serde(name = "node_collection")]
impl SceneNodeUnloaded for NodeCollectionUnloaded {
    fn collect_references(&self) -> HashSet<ResourceIdUninit> {
        let mut refs = HashSet::new();
        for child in &self.children {
            refs.extend(child.collect_references());
        }
        refs
    }
}

impl Initializable for NodeCollectionUnloaded {
    type Initialized = Box<dyn SceneNode>;

    fn load(self: Box<Self>, reference_replacer: &mut dyn ReferenceReplacer) -> Box<dyn SceneNode> {
        Box::from(NodeCollection {
            children: self
                .children
                .into_iter()
                .map(|child| child.load(reference_replacer))
                .collect(),
        })
    }
}

impl SceneNode for NodeCollection {}
