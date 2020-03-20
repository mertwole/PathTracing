use crate::aabb::*;
use crate::obj_loader::*;
use crate::triangle::*;
use crate::vec3::*;

use std::fs::File;
use std::io::prelude::*;

const SAH_SAMPLES: u32 = 16;
const MAX_TRIANGLES: usize = 8;

pub struct Tree {
    triangles: Vec<Triangle>,
    root: TreeNode,
}

struct TreeNode {
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    bounding_box: AABB,

    triangle_ids: Vec<usize>,

    global_id: u32,
    parent_id: i32,
    left_id: u32,
    right_id: u32,

    is_leaf: bool,
}

impl TreeNode {
    fn new() -> TreeNode {
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
        let id = "id ".to_string() + &self.global_id.to_string() + &" ".to_string();
        let box_min = "box_min ".to_string()
            + &self.bounding_box.min.x.to_string()
            + &" ".to_string()
            + &self.bounding_box.min.y.to_string()
            + &" ".to_string()
            + &self.bounding_box.min.z.to_string()
            + &" ".to_string();
        let box_max = "box_max ".to_string()
            + &self.bounding_box.max.x.to_string()
            + &" ".to_string()
            + &self.bounding_box.max.y.to_string()
            + &" ".to_string()
            + &self.bounding_box.max.z.to_string()
            + &" ".to_string();
        let parent = "p ".to_string() + &self.parent_id.to_string() + &" ".to_string();
        id + &box_min
            + &box_max
            + &parent
            + &if self.is_leaf {
                let mut triangles_str = "tris ".to_string();
                for triangle_id in &self.triangle_ids {
                    triangles_str += &triangle_id.to_string();
                    triangles_str += &" ";
                }
                triangles_str
            } else {
                "l ".to_string()
                    + &self.left_id.to_string()
                    + &" ".to_string()
                    + &"r ".to_string()
                    + &self.right_id.to_string()
                    + &" ".to_string()
            }
    }
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            triangles: Vec::new(),
            root: TreeNode::new(),
        }
    }

    // Load

    pub fn load_triangles(&mut self, path: &String) {
        OBJLoader::load_obj(path, &mut self.triangles);
    }

    // Build

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

    // Save

    pub fn save(&mut self, name: &str) {
        self.prepare_save();
        let mut file = File::create(name).unwrap();
        Tree::save_recursively(&self.root, &mut file);
    }

    fn save_recursively(node: &TreeNode, file: &mut File) {
        let line = node.get_text_description();
        file.write(line.as_bytes()).unwrap();
        file.write("\n".as_bytes()).unwrap();

        match &node.left {
            Some(left) => {
                Tree::save_recursively(&left, file);
            }
            _ => {}
        }
        match &node.right {
            Some(right) => {
                Tree::save_recursively(&right, file);
            }
            _ => {}
        }
    }

    fn prepare_save(&mut self) {
        self.root.parent_id = -1;
        let root_layer = vec![&mut self.root];
        Tree::set_layer_ids(root_layer, 0);
    }

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
}
