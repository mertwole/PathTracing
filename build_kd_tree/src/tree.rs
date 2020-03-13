use crate::aabb::*;
use crate::triangle::*;
use crate::vec3::*;
use crate::aabb::*;

pub struct Tree {
    triangles: Vec<Triangle>,
}

struct TreeNode {
    left : Box<TreeNode>,
    right : Box<TreeNode>,
    parent : Box<TreeNode>,

    bounding_box : AABB,

    triangle_ids : Vec<usize>,

    global_id : u32
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            triangles: Vec::new(),
        }
    }

    pub fn load_triangles_obj(&mut self, path: &String) {}

    pub fn build(&mut self) {}

    pub fn save(&mut self) {}


    fn split() {}

    fn compute_sah(&self, node : TreeNode, split_normal : Vec3, split_pos : Vec3) -> f32{
        let diagonal = &node.bounding_box.max - &node.bounding_box.min;

        let aabb_areas = Vec3::new(diagonal.y * diagonal.z, diagonal.x * diagonal.z, diagonal.x * diagonal.y);
        let left_ratio = (&split_pos - &node.bounding_box.min).dot(&split_normal) / diagonal.dot(&split_normal);
        let left_multiplier = Vec3::new(
            if split_normal.x != 0.0 {1.0} else {left_ratio},
            if split_normal.y != 0.0 {1.0} else {left_ratio},
            if split_normal.z != 0.0 {1.0} else {left_ratio},
        );
        let left_part_area = aabb_areas.dot(&left_multiplier);
        let right_ratio = 1.0 - left_ratio;
        let right_multiplier = Vec3::new(
            if split_normal.x != 0.0 {1.0} else {right_ratio},
            if split_normal.y != 0.0 {1.0} else {right_ratio},
            if split_normal.z != 0.0 {1.0} else {right_ratio},
        );
        let right_part_area = aabb_areas.dot(&right_multiplier);

        let split_pos_along_normal = &split_normal * &split_pos;
        let one_minus_normal = &(&Vec3::new(1.0, 1.0, 1.0) - &split_normal);
        let separating_point_left = &split_pos_along_normal + &(one_minus_normal * &node.bounding_box.max);
        let separating_point_right = &split_pos_along_normal + &(one_minus_normal * &node.bounding_box.min);
        let left_aabb = AABB::new(node.bounding_box.min.clone(), separating_point_left);
        let right_aabb = AABB::new(separating_point_right, node.bounding_box.max.clone());
  
        let (mut left_triangles, mut right_triangles) = (0.0, 0.0);    
        for triangle_id in node.triangle_ids{
            if Tree::triangle_vs_aabb(&self.triangles[triangle_id], &left_aabb) { 
                left_triangles += 1.0; 
            }
            if Tree::triangle_vs_aabb(&self.triangles[triangle_id], &right_aabb) { 
                right_triangles += 1.0; 
            }
        }

        left_part_area * left_triangles + right_part_area * right_triangles
    }

    fn triangle_vs_aabb(triangle: &Triangle, aabb: &AABB) -> bool {
        let aabb_vertices: [&Vec3; 8] = [
            &aabb.min,
            &Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            &Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            &Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            &Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            &Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            &Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            &aabb.max,
        ];
        //facets are: 0-1-2-3||4-5-6-7||1-2-7-6||2-3-4-7||0-3-4-5||0-1-6-5
    
        //plane equality is normal.x * x + normal.y * y + normal.z * z + d = 0
        let triangle_normal = (&triangle.points[0] - &triangle.points[1])
            .cross(&(&triangle.points[0] - &triangle.points[2]));
        let triangle_d = -triangle_normal.dot(&triangle.points[0]);
    
        let mut intersection = false;
    
        let mut box_sign = 0;
        for i in 0..8{
            let i_vert_side = triangle_normal.dot(aabb_vertices[i]) + triangle_d;
            if small_enought(i_vert_side){
                continue;
            }
            if box_sign == 0{
                box_sign = if i_vert_side > 0.0 { 1 } else { -1 }; 
                continue;
            }
            if (i_vert_side > 0.0 && box_sign == -1) || (i_vert_side < 0.0 && box_sign != -1){
                intersection = true;
                break;
            }
        }
    
        if !intersection{
            return false;
        }
    
        let check_box_side = |point_ids : [usize; 3], opposite_side_point_id : usize| {
            let normal = (aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[1]])
            .cross(&(aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[2]]));
            let d = -normal.dot(aabb_vertices[point_ids[0]]);
    
            let triangle_side = 
            if normal.dot(aabb_vertices[opposite_side_point_id]) + d < 0.0 { 1 } else { -1 };
            for i in 0..3{
                let triangle_dot_side = normal.dot(&triangle.points[i]) + d;
                if small_enought(triangle_dot_side){
                    continue;
                }
                let triangle_dot_side_i = if triangle_dot_side > 0.0 {1} else {-1};
                if triangle_side != triangle_dot_side_i{
                    return true;
                }
            }
            
            false
        };
    
        if !check_box_side([0, 1, 2], 4){ return false; }
        if !check_box_side([4, 5, 6], 0){ return false; }
        if !check_box_side([1, 2, 7], 0){ return false; }
        if !check_box_side([2, 3, 4], 0){ return false; }
        if !check_box_side([0, 3, 4], 7){ return false; }
        if !check_box_side([0, 1, 6], 7){ return false; }
    
        true
    }  
}

fn small_enought(a : f32) -> bool{
    a < std::f32::EPSILON * 10.0 && a > -std::f32::EPSILON * 10.0
}