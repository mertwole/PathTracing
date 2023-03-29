use crate::ray::Ray;
use serde::Deserialize;

use math::Vec3;

use crate::raytraceable::{Bounded, RayTraceResult, AABB};

mod tree_node;

use tree_node::TreeNode;

#[derive(Deserialize)]
pub struct KdTreeConfig {
    depth: usize,
    sah_samples: usize,
    min_primitives_in_node: usize,
}

pub struct KdTree {
    primitives: Vec<Box<dyn Bounded>>,
    root: TreeNode,
    config: KdTreeConfig,
}

impl KdTree {
    pub fn new(primitives: Vec<Box<dyn Bounded>>, config: KdTreeConfig) -> KdTree {
        KdTree {
            root: TreeNode::default(),
            primitives,
            config,
        }
    }

    pub fn build(&mut self) {
        self.init_root();
        // @TODO: Implement multithreading
        self.root
            .split_recursively(&self.primitives, 0, &self.config);
    }

    fn init_root(&mut self) {
        self.root.bounding_box = AABB::new(
            Vec3::new_xyz(std::f32::INFINITY),
            Vec3::new_xyz(std::f32::NEG_INFINITY),
        );

        for (primitive_id, primitive) in self.primitives.iter().enumerate() {
            let aabb = primitive.get_bounds();
            self.root.primitive_ids.push(primitive_id);
            self.root.bounding_box = self.root.bounding_box.unite(&aabb);
        }

        let threshold = Vec3::new_xyz(std::f32::EPSILON * 10.0);
        self.root.bounding_box.min = self.root.bounding_box.min - threshold;
        self.root.bounding_box.max = self.root.bounding_box.max + threshold;
    }

    pub fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        self.root.trace_ray_recursively(&self.primitives, ray)
    }
}
