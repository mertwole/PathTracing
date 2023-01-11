mod obj_loader;
use obj_loader::*;

use kd_tree::{
    aabb::AABB,
    tree::{Tree, TreeNode},
};

use super::{triangle::Triangle, RayTraceResult, Raytraceable};
use crate::ray::*;
use math::*;

pub struct KDTree {
    triangles: Vec<Triangle>,
    root: TreeNode,
    material_id: usize,
}

impl Default for KDTree {
    fn default() -> Self {
        KDTree {
            triangles: Vec::new(),
            root: TreeNode::new(),
            material_id: 0,
        }
    }
}

impl KDTree {
    pub fn new(material_id: usize) -> KDTree {
        KDTree {
            triangles: Vec::new(),
            root: TreeNode::new(),
            material_id,
        }
    }

    pub fn load(&mut self, obj_path: &str, tree_path: &String) {
        OBJLoader::load_obj(obj_path, &mut self.triangles);
        self.root = Tree::get_tree_from_file(tree_path);
    }

    fn trace_ray_aabb(aabb: &AABB, ray: &Ray) -> bool {
        let inv_dir = Vec3::new_xyz(1.0) / ray.direction;
        let t0 = (aabb.min - ray.source) * inv_dir;
        let t1 = (aabb.max - ray.source) * inv_dir;

        let max_triple = |a: f32, b: f32, c: f32| a.max(b.max(c));
        let min_triple = |a: f32, b: f32, c: f32| a.min(b.min(c));

        let tmin = max_triple(t0.x.min(t1.x), t0.y.min(t1.y), t0.z.min(t1.z));
        let tmax = min_triple(t0.x.max(t1.x), t0.y.max(t1.y), t0.z.max(t1.z));
        tmin <= tmax
    }

    fn trace_ray_recursively(&self, root: &TreeNode, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        if !Self::trace_ray_aabb(&root.bounding_box, &ray) {
            return result;
        }

        if root.left.is_none() {
            // Leaf
            result.t = std::f32::MAX;
            for &triangle_id in &root.triangle_ids {
                let triangle = &self.triangles[triangle_id];
                let triangle_result = triangle.trace_ray(ray);
                if triangle_result.hit && triangle_result.t < result.t {
                    result = triangle_result;
                    result.material_id = self.material_id;
                }
            }
            result
        } else {
            // Node
            let left_result = self.trace_ray_recursively(root.left.as_ref().unwrap(), ray);
            let right_result = self.trace_ray_recursively(root.right.as_ref().unwrap(), ray);
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

impl Raytraceable for KDTree {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = self.trace_ray_recursively(&self.root, ray);
        if result.hit {
            result.material_id = self.material_id;
        }
        result
    }
}
