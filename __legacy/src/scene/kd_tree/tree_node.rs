use math::Vec3;

use crate::ray::Ray;
use crate::raytraceable::{Bounded, RayTraceResult, AABB};

use super::KdTreeConfig;

#[derive(Default)]
pub struct TreeNode {
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub bounding_box: AABB,

    pub primitive_ids: Vec<usize>,
}

impl TreeNode {
    pub fn split_recursively(
        &mut self,
        primitives: &[Box<dyn Bounded>],
        depth: usize,
        config: &KdTreeConfig,
    ) {
        if depth >= config.depth {
            return;
        }

        // Find optimal split plane
        let mut min_sah = std::f32::INFINITY;
        // Pick largest dimension
        let diagonal = self.bounding_box.max - self.bounding_box.min;
        let split_normal = if diagonal.x > diagonal.y && diagonal.x > diagonal.z {
            Vec3::new(1.0, 0.0, 0.0)
        } else if diagonal.y > diagonal.z {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };
        // Sample SAH
        let mut split_pos = Vec3::default();
        for i in 1..config.sah_samples {
            let pos = self.bounding_box.min + diagonal * (i as f32 / config.sah_samples as f32);
            let sah = Self::compute_sah(self, primitives, split_normal, pos);
            if sah < min_sah {
                split_pos = pos;
                min_sah = sah;
            }
        }
        // Init childs
        let mut left_node = TreeNode::default();
        let mut right_node = TreeNode::default();

        let (left_aabb, right_aabb) = self.bounding_box.split(split_pos, split_normal);
        left_node.bounding_box = left_aabb;
        right_node.bounding_box = right_aabb;

        for &primitive_id in &self.primitive_ids {
            if primitives[primitive_id].intersect_with_aabb(&left_node.bounding_box) {
                left_node.primitive_ids.push(primitive_id)
            }
            if primitives[primitive_id].intersect_with_aabb(&right_node.bounding_box) {
                right_node.primitive_ids.push(primitive_id)
            }
        }

        if left_node.primitive_ids.len() > config.min_primitives_in_node {
            Self::split_recursively(&mut left_node, primitives, depth + 1, config);
        }
        if right_node.primitive_ids.len() > config.min_primitives_in_node {
            Self::split_recursively(&mut right_node, primitives, depth + 1, config);
        }

        self.left = Some(Box::new(left_node));
        self.right = Some(Box::new(right_node));
    }

    fn compute_sah(
        &self,
        primitives: &[Box<dyn Bounded>],
        split_normal: Vec3,
        split_pos: Vec3,
    ) -> f32 {
        let diagonal = self.bounding_box.max - self.bounding_box.min;

        let aabb_areas = Vec3::new(
            diagonal.y * diagonal.z,
            diagonal.x * diagonal.z,
            diagonal.x * diagonal.y,
        );

        let left_ratio =
            (split_pos - self.bounding_box.min).dot(split_normal) / diagonal.dot(split_normal);
        let left_multiplier = Vec3::new(
            if split_normal.x != 0.0 {
                1.0
            } else {
                left_ratio
            },
            if split_normal.y != 0.0 {
                1.0
            } else {
                left_ratio
            },
            if split_normal.z != 0.0 {
                1.0
            } else {
                left_ratio
            },
        );
        let left_part_area = aabb_areas.dot(left_multiplier);

        let right_ratio = 1.0 - left_ratio;
        let right_multiplier = Vec3::new(
            if split_normal.x != 0.0 {
                1.0
            } else {
                right_ratio
            },
            if split_normal.y != 0.0 {
                1.0
            } else {
                right_ratio
            },
            if split_normal.z != 0.0 {
                1.0
            } else {
                right_ratio
            },
        );
        let right_part_area = aabb_areas.dot(right_multiplier);

        let (left_aabb, right_aabb) = &self.bounding_box.split(split_pos, split_normal);
        let (mut left_primitives, mut right_primitives) = (0.0, 0.0);
        for &primitive_id in &self.primitive_ids {
            if primitives[primitive_id].intersect_with_aabb(&left_aabb) {
                left_primitives += 1.0;
            }
            if primitives[primitive_id].intersect_with_aabb(&right_aabb) {
                right_primitives += 1.0;
            }
        }

        left_part_area * left_primitives + right_part_area * right_primitives
    }
}

impl TreeNode {
    pub fn trace_ray_recursively(
        &self,
        primitives: &[Box<dyn Bounded>],
        ray: &Ray,
    ) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        if !self.bounding_box.trace_ray(ray) {
            return result;
        }

        if self.left.is_none() {
            // Leaf
            result.t = std::f32::MAX;
            for &primitive_id in &self.primitive_ids {
                let primitive = &primitives[primitive_id];
                let primitive_result = primitive.trace_ray(ray);
                if primitive_result.hit && primitive_result.t < result.t {
                    result = primitive_result;
                }
            }
            result
        } else {
            // Node
            let left_result =
                Self::trace_ray_recursively(self.left.as_ref().unwrap(), primitives, ray);
            let right_result =
                Self::trace_ray_recursively(self.right.as_ref().unwrap(), primitives, ray);
            if !left_result.hit {
                right_result
            } else if !right_result.hit {
                left_result
            } else if left_result.t > right_result.t {
                right_result
            } else {
                left_result
            }
        }
    }
}
