use crate::aabb::*;
use crate::obj_loader::*;
use crate::triangle::*;
use crate::vec3::*;

const SAH_SAMPLES: u32 = 16;
const MAX_TRIANGLES: usize = 8;

pub struct Tree {
    triangles: Vec<Triangle>,
}

struct TreeNode {
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    bounding_box: AABB,

    triangle_ids: Vec<usize>,

    global_id: u32,
}

impl TreeNode {
    fn new() -> TreeNode {
        TreeNode {
            left: Option::None,
            right: Option::None,
            bounding_box: AABB::void(),
            triangle_ids: Vec::new(),
            global_id: 0,
        }
    }
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            triangles: Vec::new(),
        }
    }

    pub fn load_triangles(&mut self, path: &String) {
        OBJLoader::load_obj(path, &mut self.triangles);
    }

    pub fn build(&mut self, depth: u32) {
        let mut root = self.init_root();
        self.split_root(&mut root, depth);
    }

    pub fn save(&self) {}

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
            self.split(root, 0, depth);
        }
    }

    fn split_root_multithread(&self, root: &mut TreeNode, depth: u32) {
        self.split(root, 0, depth);
    }

    fn split(&self, root: &mut TreeNode, depth: u32, max_depth: u32) {
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

        let (left_aabb, right_aabb) =
            Tree::split_aabb(&root.bounding_box, &split_pos, &split_normal);
        left_node.bounding_box = left_aabb;
        right_node.bounding_box = right_aabb;

        for &triangle_id in &root.triangle_ids {
            if Tree::triangle_vs_aabb(&self.triangles[triangle_id], &left_node.bounding_box) {
                left_node.triangle_ids.push(triangle_id)
            }
            if Tree::triangle_vs_aabb(&self.triangles[triangle_id], &right_node.bounding_box) {
                right_node.triangle_ids.push(triangle_id)
            }
        }

        if left_node.triangle_ids.len() > MAX_TRIANGLES {
            self.split(&mut left_node, depth + 1, max_depth);
        }
        if right_node.triangle_ids.len() > MAX_TRIANGLES {
            self.split(&mut right_node, depth + 1, max_depth);
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
        let (left_aabb, right_aabb) =
            Tree::split_aabb(&node.bounding_box, &split_pos, &split_normal);

        let (mut left_triangles, mut right_triangles) = (0.0, 0.0);
        for &triangle_id in &node.triangle_ids {
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
        for i in 0..8 {
            let i_vert_side = triangle_normal.dot(aabb_vertices[i]) + triangle_d;
            if small_enought(i_vert_side) {
                continue;
            }
            if box_sign == 0 {
                box_sign = if i_vert_side > 0.0 { 1 } else { -1 };
                continue;
            }
            if (i_vert_side > 0.0 && box_sign == -1) || (i_vert_side < 0.0 && box_sign != -1) {
                intersection = true;
                break;
            }
        }

        if !intersection {
            return false;
        }

        let check_box_side = |point_ids: [usize; 3], opposite_side_point_id: usize| {
            let normal = (aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[1]])
                .cross(&(aabb_vertices[point_ids[0]] - aabb_vertices[point_ids[2]]));
            let d = -normal.dot(aabb_vertices[point_ids[0]]);

            let triangle_side = if normal.dot(aabb_vertices[opposite_side_point_id]) + d < 0.0 {
                1
            } else {
                -1
            };
            for i in 0..3 {
                let triangle_dot_side = normal.dot(&triangle.points[i]) + d;
                if small_enought(triangle_dot_side) {
                    continue;
                }
                let triangle_dot_side_i = if triangle_dot_side > 0.0 { 1 } else { -1 };
                if triangle_side != triangle_dot_side_i {
                    return true;
                }
            }

            false
        };

        if !check_box_side([0, 1, 2], 4) {
            return false;
        }
        if !check_box_side([4, 5, 6], 0) {
            return false;
        }
        if !check_box_side([1, 2, 7], 0) {
            return false;
        }
        if !check_box_side([2, 3, 4], 0) {
            return false;
        }
        if !check_box_side([0, 3, 4], 7) {
            return false;
        }
        if !check_box_side([0, 1, 6], 7) {
            return false;
        }

        true
    }

    fn split_aabb(aabb: &AABB, plane_pos: &Vec3, plane_normal: &Vec3) -> (AABB, AABB) {
        let split_pos_along_normal = plane_pos * plane_normal;
        let one_minus_normal = &Vec3::new(1.0, 1.0, 1.0) - &plane_normal;
        (
            AABB::new(
                aabb.min.clone(),
                &split_pos_along_normal + &(&one_minus_normal * &aabb.max),
            ),
            AABB::new(
                &split_pos_along_normal + &(&one_minus_normal * &aabb.min),
                aabb.max.clone(),
            ),
        )
    }
}

fn small_enought(a: f32) -> bool {
    a < std::f32::EPSILON * 10.0 && a > -std::f32::EPSILON * 10.0
}
