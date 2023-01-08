mod obj_loader;
use obj_loader::*;

use kd_tree::aabb::AABB;
use kd_tree::tree::{Tree, TreeNode};

use super::triangle::*;
use super::Raytraceable;
use crate::ray::*;
use math::*;

pub struct KDTree {
    triangles: Vec<Triangle>,
    root: TreeNode,
    material_id: usize,
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

    fn trace_ray_recursively(&self, root: &TreeNode, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        if !root.bounding_box.trace_ray(ray) {
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

trait RaytraceableBool {
    fn trace_ray(&self, ray: &Ray) -> bool;
}

fn max_triple(a: f32, b: f32, c: f32) -> f32 {
    a.max(b.max(c))
}

fn min_triple(a: f32, b: f32, c: f32) -> f32 {
    a.min(b.min(c))
}

impl RaytraceableBool for AABB {
    fn trace_ray(&self, ray: &Ray) -> bool {
        let inv_dir = Vec3::new_xyz(1.0) / ray.direction;
        let t0 = (self.min - ray.source) * inv_dir;
        let t1 = (self.max - ray.source) * inv_dir;

        let tmin = max_triple(t0.x.min(t1.x), t0.y.min(t1.y), t0.z.min(t1.z));
        let tmax = min_triple(t0.x.max(t1.x), t0.y.max(t1.y), t0.z.max(t1.z));
        if tmin > tmax {
            return false;
        }
        //return (tmin > ray.min && tmin < ray.max) || (tmax > ray.min && tmax < ray.max)
        true
    }
}
