mod obj_loader;
use obj_loader::*;

extern crate kd_tree_lib;
use kd_tree_lib::tree::{TreeNode, Tree};

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
}

impl Raytraceable for KDTree{
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult{
        let result = RayTraceResult::void();

        // TODO : KD-tree traversing

        result
    }
}