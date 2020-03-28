use crate::aabb::*;
use crate::obj_loader::*;
use crate::triangle::*;

extern crate math_lib;
use math_lib::Vec3;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

const SAH_SAMPLES: u32 = 16;
const MAX_TRIANGLES: usize = 8;

impl AABB{
    fn split(&self, plane_pos: &Vec3, plane_normal: &Vec3) -> (AABB, AABB) {
        let split_pos_along_normal = plane_pos * plane_normal;
        let one_minus_normal = &Vec3::new(1.0, 1.0, 1.0) - &plane_normal;
        (
            AABB::new(
                self.min.clone(),
                &split_pos_along_normal + &(&one_minus_normal * &self.max),
            ),
            AABB::new(
                &split_pos_along_normal + &(&one_minus_normal * &self.min),
                self.max.clone(),
            ),
        )
    }
}

pub struct TreeNode {
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub bounding_box: AABB,

    pub triangle_ids: Vec<usize>,

    global_id: u32,
    parent_id: i32,
    left_id: u32,
    right_id: u32,

    is_leaf: bool,
}

impl TreeNode {
    pub fn new() -> TreeNode {
        TreeNode {
            left: Option::None,
            right: Option::None,
            bounding_box: AABB::void(),
            triangle_ids: Vec::new(),

            global_id: 0,
            parent_id: 0,
            left_id: 0,
            right_id: 0,

            is_leaf: false,
        }
    }

    fn get_text_description(&self) -> String {
        let box_min = "box_min ".to_string()
            + &self.bounding_box.min.x.to_string()
            + &" ".to_string()
            + &self.bounding_box.min.y.to_string()
            + &" ".to_string()
            + &self.bounding_box.min.z.to_string()
            + &" ".to_string();
        let box_max = " box_max ".to_string()
            + &self.bounding_box.max.x.to_string()
            + &" ".to_string()
            + &self.bounding_box.max.y.to_string()
            + &" ".to_string()
            + &self.bounding_box.max.z.to_string();
        let parent = " p ".to_string() + &self.parent_id.to_string();
        
            box_min
            + &box_max
            + &parent
            + &if self.is_leaf {
                let mut triangles_str = " tris".to_string();
                for triangle_id in &self.triangle_ids {
                    triangles_str += &" ";
                    triangles_str += &triangle_id.to_string();             
                }
                triangles_str
            } else {
                " l ".to_string()
                + &self.left_id.to_string()
                + &" r ".to_string()
                + &self.right_id.to_string()
            }
    }
}

struct TreeNodeAdditionalData{
    pub node : TreeNode,

    pub left : usize,
    pub right : usize,
    pub is_leaf : bool
}

impl TreeNodeAdditionalData{
    fn void() -> TreeNodeAdditionalData{
        TreeNodeAdditionalData { 
            node : TreeNode::new(), 
            left : 0, 
            right : 0, 
            is_leaf : false 
        }
    }
}

pub struct Tree {
    triangles: Vec<Triangle>,
    pub root: TreeNode,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            triangles: Vec::new(),
            root: TreeNode::new(),
        }
    }

    // region load from file
    pub fn load(&mut self, path : &String){
        let mut nodes = Tree::load_nodes(path);
        self.root = Tree::build_from_nodes_recursively(0, &mut nodes);
    }

    pub fn get_tree_from_file(path : &String) -> TreeNode{
        let mut nodes = Tree::load_nodes(path);
        Tree::build_from_nodes_recursively(0, &mut nodes)
    }

    fn build_from_nodes_recursively(root_id : usize, nodes : &mut Vec<TreeNodeAdditionalData>) -> TreeNode{
        // Hack to move out of Vec saving root parameters before
        let root_is_leaf = nodes[root_id].is_leaf;
        let left_id = nodes[root_id].left;
        let right_id = nodes[root_id].right;
        
        let mut root = nodes.remove(root_id).node;
        nodes.insert(root_id, TreeNodeAdditionalData::void());

        if root_is_leaf { return root; }

        root.left = Option::Some(Box::new(
            Tree::build_from_nodes_recursively(left_id, nodes)
        ));
        root.right = Option::Some(Box::new(
            Tree::build_from_nodes_recursively(right_id, nodes)
        ));

        root
    }

    fn load_nodes(path : &String) -> Vec<TreeNodeAdditionalData>{
        let reader = BufReader::new(File::open(path.as_str()).unwrap());
        let mut nodes : Vec<TreeNodeAdditionalData> = Vec::new();
        for line in reader.lines(){
            let line = line.unwrap();   
            nodes.push(Tree::load_node(&line));
        }
        nodes
    }

    fn load_node(line : &String) -> TreeNodeAdditionalData {
        let mut node = TreeNodeAdditionalData { node : TreeNode::new(), left : 0, right : 0, is_leaf : false };
        let mut word_iter = line.split_whitespace();
        loop {
            let word = word_iter.next();
            if word.is_none() { break; } 
            let word = word.unwrap();
            match word{
                "box_min" | "box_max" => { 
                    let x : f32 = word_iter.next().unwrap().parse().unwrap();
                    let y : f32 = word_iter.next().unwrap().parse().unwrap();
                    let z : f32 = word_iter.next().unwrap().parse().unwrap();
                    
                    if word == "box_min"{
                        node.node.bounding_box.min = Vec3::new(x, y, z);
                    }
                    else {
                        node.node.bounding_box.max = Vec3::new(x, y, z);
                    }               
                }
                "l" => { node.left = word_iter.next().unwrap().parse().unwrap(); }
                "r" => { node.right = word_iter.next().unwrap().parse().unwrap(); }
                "tris" => {
                    node.is_leaf = true;
                    loop{
                        let tr_id = word_iter.next();
                        match tr_id{
                            Some(id) => { node.node.triangle_ids.push(id.parse().unwrap()); }
                            None => { break; }
                        }
                    }
                } 
                _ => { }
            }
        }

        node
    }
    // endregion

    // region load triangles
    pub fn load_triangles(&mut self, path: &String) {
        OBJLoader::load_obj(path, &mut self.triangles);
    }
    // endregion

    // region build
    pub fn build(&mut self, depth: u32) {
        let mut root = self.init_root();
        self.split_root(&mut root, depth);
        self.root = root;
    }

    fn init_root(&mut self) -> TreeNode {
        let mut root = TreeNode::new();
        root.bounding_box = AABB::new(
            Vec3::new_xyz(std::f32::INFINITY),
            Vec3::new_xyz(std::f32::NEG_INFINITY),
        );
        // Get bounding box for all triangles
        for triangle in &self.triangles {
            for i in 0..3 {
                if root.bounding_box.min.x > triangle.points[i].x {
                    root.bounding_box.min.x = triangle.points[i].x;
                }
                if root.bounding_box.min.y > triangle.points[i].y {
                    root.bounding_box.min.y = triangle.points[i].y;
                }
                if root.bounding_box.min.z > triangle.points[i].z {
                    root.bounding_box.min.z = triangle.points[i].z;
                }

                if root.bounding_box.max.x < triangle.points[i].x {
                    root.bounding_box.max.x = triangle.points[i].x;
                }
                if root.bounding_box.max.y < triangle.points[i].y {
                    root.bounding_box.max.y = triangle.points[i].y;
                }
                if root.bounding_box.max.z < triangle.points[i].z {
                    root.bounding_box.max.z = triangle.points[i].z;
                }
            }
        }
        let threshold = Vec3::new_xyz(std::f32::EPSILON * 10.0);
        root.bounding_box.min = &root.bounding_box.min - &threshold;
        root.bounding_box.max = &root.bounding_box.max + &threshold;

        root.triangle_ids = (0..self.triangles.len()).collect();

        root
    }

    fn split_root(&self, root: &mut TreeNode, depth: u32) {
        if depth > 3 {
            self.split_root_multithread(root, depth);
        } else {
            self.split_recursively(root, 0, depth);
        }
    }

    fn split_root_multithread(&self, root: &mut TreeNode, depth: u32) {
        self.split_recursively(root, 0, depth);
    }

    fn split_recursively(&self, root: &mut TreeNode, depth: u32, max_depth: u32) {
        if depth > max_depth - 2 {
            return;
        }
        // Find optimal split plane
        let mut min_sah = std::f32::INFINITY;
        // Pick largest dimension
        let diagonal = &root.bounding_box.max - &root.bounding_box.min;
        let split_normal = if diagonal.x > diagonal.y && diagonal.x > diagonal.z {
            Vec3::new(1.0, 0.0, 0.0)
        } else if diagonal.y > diagonal.z {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };
        // Sample SAH
        let mut split_pos = Vec3::zero();
        for i in 1..SAH_SAMPLES {
            let pos = &root.bounding_box.min + &(&diagonal * (i as f32 / SAH_SAMPLES as f32));
            let sah = self.compute_sah(root, &split_normal, &pos);
            if sah < min_sah {
                split_pos = pos;
                min_sah = sah;
            }
        }
        // Init childs
        let mut left_node = TreeNode::new();
        let mut right_node = TreeNode::new();

        let (left_aabb, right_aabb) = root.bounding_box.split(&split_pos, &split_normal);
        left_node.bounding_box = left_aabb;
        right_node.bounding_box = right_aabb;

        for &triangle_id in &root.triangle_ids {
            if self.triangles[triangle_id].check_with_aabb(&left_node.bounding_box) {
                left_node.triangle_ids.push(triangle_id)
            }
            if self.triangles[triangle_id].check_with_aabb(&right_node.bounding_box) {
                right_node.triangle_ids.push(triangle_id)
            }
        }

        if left_node.triangle_ids.len() > MAX_TRIANGLES {
            self.split_recursively(&mut left_node, depth + 1, max_depth);
        }
        if right_node.triangle_ids.len() > MAX_TRIANGLES {
            self.split_recursively(&mut right_node, depth + 1, max_depth);
        }

        root.left = Some(Box::new(left_node));
        root.right = Some(Box::new(right_node));
    }

    fn compute_sah(&self, node: &TreeNode, split_normal: &Vec3, split_pos: &Vec3) -> f32 {
        let diagonal = &node.bounding_box.max - &node.bounding_box.min;

        let aabb_areas = Vec3::new(
            diagonal.y * diagonal.z,
            diagonal.x * diagonal.z,
            diagonal.x * diagonal.y,
        );

        let left_ratio =
            (split_pos - &node.bounding_box.min).dot(split_normal) / diagonal.dot(&split_normal);
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
        let left_part_area = aabb_areas.dot(&left_multiplier);

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
        let right_part_area = aabb_areas.dot(&right_multiplier);

        let (left_aabb, right_aabb) = &node.bounding_box.split(&split_pos, &split_normal);
        let (mut left_triangles, mut right_triangles) = (0.0, 0.0);
        for &triangle_id in &node.triangle_ids {
            if self.triangles[triangle_id].check_with_aabb(&left_aabb) {
                left_triangles += 1.0;
            }
            if self.triangles[triangle_id].check_with_aabb(&right_aabb) {
                right_triangles += 1.0;
            }
        }

        left_part_area * left_triangles + right_part_area * right_triangles
    }
    // endregion

    // region save
    pub fn save(&mut self, name: &str) {
        self.prepare_save();
        let mut file = File::create(name).unwrap();
        let mut node_list : Vec<&TreeNode> = Vec::new();
        Tree::get_node_list_recursively(&self.root, &mut node_list);
        node_list.sort_by(|a, b| a.global_id.cmp(&b.global_id));
        Tree::save_node_list(&node_list, &mut file);
    }

    fn get_node_list_recursively<'a>(node : &'a TreeNode, node_list : &mut Vec<&'a TreeNode>){
        match &node.left {
            Some(left) => {
                Tree::get_node_list_recursively(&left, node_list);
            }
            _ => {}
        }
        match &node.right {
            Some(right) => {
                Tree::get_node_list_recursively(&right, node_list);
            }
            _ => {}
        }

        node_list.push(node);
    }

    fn save_node_list(node_list : &Vec<&TreeNode>, file : &mut File){
        let mut first_line = true;
        for node in node_list{           
            if !first_line{
                file.write("\n".as_bytes()).unwrap();
            } else { first_line = false; }
            let line = node.get_text_description();         
            file.write(line.as_bytes()).unwrap();         
        }
    }

    fn prepare_save(&mut self) {
        self.root.parent_id = -1;
        let root_layer = vec![&mut self.root];
        Tree::set_layer_ids(root_layer, 0);
    }

    // Set parent, left, right and global ids to tree's layer
    fn set_layer_ids(curr_layer: Vec<&mut TreeNode>, curr_layer_max_id: i32) {
        let mut next_layer: Vec<&mut TreeNode> = Vec::new();
        let mut next_layer_max_id = curr_layer_max_id;
        let mut not_all_leaves = false;
        for node in curr_layer {
            match &mut node.left {
                Some(left) => {
                    not_all_leaves = true;
                    next_layer.push(left.as_mut());
                    let left = next_layer.last_mut().unwrap();
                    next_layer_max_id += 1;
                    left.global_id = next_layer_max_id as u32;
                    left.parent_id = node.global_id as i32;
                    node.left_id = left.global_id;
                }
                _ => {
                    node.is_leaf = true;
                }
            }
            match &mut node.right {
                Some(right) => {
                    not_all_leaves = true;
                    next_layer.push(right.as_mut());
                    let right = next_layer.last_mut().unwrap();
                    next_layer_max_id += 1;
                    right.global_id = next_layer_max_id as u32;
                    right.parent_id = node.global_id as i32;
                    node.right_id = right.global_id;
                }
                _ => {
                    node.is_leaf = true;
                }
            }
        }

        if not_all_leaves {
            Tree::set_layer_ids(next_layer, next_layer_max_id);
        }
    }
    // endregion
}
