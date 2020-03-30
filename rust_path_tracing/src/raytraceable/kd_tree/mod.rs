mod obj_loader;
use obj_loader::*;

use kd_tree::tree::{TreeNode, Tree};
use kd_tree::aabb::AABB;

use math::*;
use super::Raytraceable;
use super::triangle::*;
use crate::ray::*;

pub struct KDTree{
    triangles : Vec<Triangle>,
    root : TreeNode,
    material_id : usize
}

impl KDTree{
    pub fn new(material_id : usize) -> KDTree{
        KDTree { 
            triangles : Vec::new(), 
            root : TreeNode::new(), 
            material_id 
        }
    }

    pub fn load(&mut self, obj_path : &String, tree_path : &String){
        OBJLoader::load_obj(obj_path, &mut self.triangles);
        self.root = Tree::get_tree_from_file(tree_path);
    }

    fn trace_ray_recursively(&self, root : &TreeNode, ray : &Ray) -> RayTraceResult{
        let mut result = RayTraceResult::void();
        if !root.bounding_box.trace_ray(ray){ return result; }

        if root.left.is_none(){
            // Leaf
            result.t = std::f32::MAX;
            for &triangle_id in &root.triangle_ids{
                let triangle = &self.triangles[triangle_id];
                let triangle_result = triangle.trace_ray(ray);
                if triangle_result.hit && triangle_result.t < result.t {
                    result = triangle_result;
                }
            } 
            return result;  
        }
        else {
            // Node
            let left_result = self.trace_ray_recursively(root.left.as_ref().unwrap(), ray);
            let right_result = self.trace_ray_recursively(root.left.as_ref().unwrap(), ray);
            if !left_result.hit{
                if !right_result.hit { return result; }
                return right_result;
            }
            if !right_result.hit { return left_result; }
            return if left_result.t > right_result.t { right_result } else { left_result }
        }
    }
}

impl Raytraceable for KDTree{
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult{
        self.trace_ray_recursively(&self.root, ray)
    }
}

trait RaytraceableBool{
    fn trace_ray(&self, ray: &Ray) -> bool;
}

impl RaytraceableBool for AABB{
    fn trace_ray(&self, ray: &Ray) -> bool{
        let mut result  = false;
        let inv_dir = &Vec3::new_xyz(1.0) / &ray.direction;
        let t0 = &(&self.min - &ray.source) * &inv_dir;
        let t1 = &(&self.max - &ray.source) * &inv_dir;
        let tmin = Math::max_triple(Math::min(t0.x, t1.x), Math::min(t0.y, t1.y), Math::min(t0.z, t1.z));
        let tmax = Math::min_triple(Math::max(t0.x, t1.x), Math::max(t0.y, t1.y), Math::max(t0.z, t1.z));
        result = tmin < tmax;
        if (tmin > ray.min && tmin < ray.max) || (tmin > ray.min && tmin < ray.max) { result = false; }
        result
    }
}