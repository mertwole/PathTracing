use serde::Deserialize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use threadpool::ThreadPool;

use math::{Color24bpprgb, UVec2};

use crate::material::{Material, MaterialUninit};
use crate::ray::Ray;
use crate::raytraceable::mesh::Mesh;
use crate::raytraceable::{RayTraceResult, Raytraceable, RaytraceableUninit};

pub mod camera;
mod image_buffer;
mod kd_tree;
mod work_group;

use camera::Camera;
use kd_tree::{KdTree, KdTreeConfig};
use work_group::WorkGroup;

#[derive(Deserialize)]
pub struct SceneConfig {
    num_threads: usize,
    trace_depth: usize,
    workgroup_size: UVec2,
}

#[derive(Deserialize)]
struct SceneDataUninit {
    config: SceneConfig,
    kd_tree_config: Option<KdTreeConfig>,

    camera: Camera,

    primitives: Vec<Box<dyn RaytraceableUninit>>,
    #[serde(default)]
    meshes: Vec<Mesh>,

    materials: Vec<Box<dyn MaterialUninit>>,
}

struct SceneData {
    config: SceneConfig,
    camera: Camera,

    kd_tree: Option<KdTree>,

    primitives: Vec<Box<dyn Raytraceable>>,

    materials: Vec<Box<dyn Material>>,
}

impl SceneDataUninit {
    fn init(self) -> SceneData {
        let mesh_triangles = self.meshes.into_iter().map(|mesh| mesh.init()).flatten();

        let primitives = self
            .primitives
            .into_iter()
            .map(|primitive| primitive.init())
            .chain(mesh_triangles);

        let (primitives, kd_tree) = if let Some(kd_tree_config) = self.kd_tree_config {
            let (primitives_bounded, primitives_unbounded): (Vec<_>, Vec<_>) =
                primitives.partition(|primitive| primitive.is_bounded());

            let mut kd_tree = KdTree::new(
                primitives_bounded
                    .into_iter()
                    .map(|prim| prim.get_bounded().unwrap())
                    .collect(),
                kd_tree_config,
            );
            kd_tree.build();

            (primitives_unbounded, Some(kd_tree))
        } else {
            (primitives.collect(), None)
        };

        SceneData {
            camera: self.camera,
            config: self.config,
            primitives,
            kd_tree,
            materials: self.materials.into_iter().map(|mat| mat.init()).collect(),
        }
    }
}

impl SceneData {
    pub fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();
        result.t = std::f32::MAX;

        if let Some(kd_tree) = &self.kd_tree {
            let tree_result = kd_tree.trace_ray(ray);
            if tree_result.hit && tree_result.t < result.t {
                result = tree_result;
            }
        }

        for primitive in &self.primitives {
            let primitive_result = primitive.trace_ray(ray);
            if primitive_result.hit && primitive_result.t < result.t {
                result = primitive_result;
            }
        }

        result
    }

    fn divide_to_workgroups(&self) -> (UVec2, Vec<WorkGroup>) {
        let mut workgroups = Vec::new();

        // Number of full-widthed and full-heighted workgroups
        let mut workgroup_count = self.camera.resolution / self.config.workgroup_size;
        let remainder = self.camera.resolution - workgroup_count * self.config.workgroup_size;
        if remainder.x != 0 {
            workgroup_count.x += 1;
        }
        if remainder.y != 0 {
            workgroup_count.y += 1;
        }

        workgroups.reserve(workgroup_count.x * workgroup_count.y);

        for row_id in 0..workgroup_count.y {
            let mut row_height = self.config.workgroup_size.y;
            // Last row can be not full-heighted
            if row_id == workgroup_count.y && remainder.y != 0 {
                row_height = remainder.y;
            }

            for column_id in 0..workgroup_count.x {
                let mut column_width = self.config.workgroup_size.x;
                // Last column can be not full-widthed
                if column_id == workgroup_count.x && remainder.x != 0 {
                    column_width = remainder.x;
                }

                let workgroup = WorkGroup::new(
                    column_id * self.config.workgroup_size.x,
                    row_id * self.config.workgroup_size.y,
                    column_width,
                    row_height,
                );

                workgroups.push(workgroup);
            }
        }

        (workgroup_count, workgroups)
    }
}

pub struct Scene {
    data: Arc<SceneData>,

    workgroup_count: UVec2,
    workgroups: Vec<WorkGroup>,
    thread_pool: ThreadPool,
}

impl Scene {
    pub fn from_json(json: &str) -> Scene {
        let data: SceneDataUninit = serde_json::de::from_str(json).unwrap();
        let data = data.init();
        let (workgroup_count, workgroups) = data.divide_to_workgroups();
        Scene {
            thread_pool: ThreadPool::new(data.config.num_threads),
            workgroup_count,
            workgroups,
            data: Arc::new(data),
        }
    }

    pub fn iterations(&mut self, num_iterations: usize) {
        let (tx, rx): (Sender<(WorkGroup, usize)>, Receiver<(WorkGroup, usize)>) = channel();

        for _ in 0..num_iterations {
            let workgroup_count = self.workgroup_count.x * self.workgroup_count.y;

            let mut workgroups_received = vec![];
            for _ in 0..workgroup_count {
                workgroups_received.push(None);
            }

            for i in 0..workgroup_count {
                let tx_ = tx.clone();
                let mut workgroup = self.workgroups.remove(0);
                let trace_depth = self.data.config.trace_depth;
                let scene_data = self.data.clone();
                self.thread_pool.execute(move || {
                    workgroup.iteration(scene_data.clone(), trace_depth);
                    tx_.send((workgroup, i)).unwrap();
                });
            }
            for _ in 0..workgroup_count {
                let (workgroup, id) = rx.recv().unwrap();
                workgroups_received[id] = Some(workgroup);
            }
            for workgroup in workgroups_received {
                self.workgroups.push(workgroup.unwrap());
            }
        }
    }

    pub fn get_raw_image(&self) -> Vec<u32> {
        let mut buffer: Vec<u32> =
            vec![0u32; self.data.camera.resolution.x * self.data.camera.resolution.y];

        for x in 0..self.workgroup_count.x {
            for y in 0..self.workgroup_count.y {
                let workgroup_buffer =
                    self.workgroups[x + y * self.workgroup_count.x].get_raw_image_data();

                for buf_x in 0..workgroup_buffer.len() {
                    for buf_y in 0..workgroup_buffer[0].len() {
                        let buf_pixel = workgroup_buffer[buf_x][buf_y];
                        let glob_x = x * self.data.config.workgroup_size.x + buf_x;
                        let glob_y = y * self.data.config.workgroup_size.y + buf_y;
                        let glob_adress = glob_x + glob_y * self.data.camera.resolution.x;

                        let buf_color = Color24bpprgb::from_hdr_tone_mapped(buf_pixel);

                        buffer[glob_adress] = (buf_color.r as u32)
                            + 256 * (buf_color.g as u32)
                            + 256 * 256 * (buf_color.b as u32)
                            + 256 * 256 * 256;
                    }
                }
            }
        }

        buffer
    }

    pub fn save_output(&self, path: &std::path::Path) {
        let raw_image = self
            .get_raw_image()
            .chunks(self.data.camera.resolution.x)
            .into_iter()
            .rev()
            .flatten()
            .map(|pixel| {
                vec![
                    (pixel % 256) as u8,
                    ((pixel >> 8) % 256) as u8,
                    ((pixel >> 16) % 256) as u8,
                ]
            })
            .flatten()
            .collect::<Vec<u8>>();

        image::save_buffer_with_format(
            path,
            &raw_image,
            self.data.camera.resolution.x as u32,
            self.data.camera.resolution.y as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Bmp,
        )
        .unwrap();
    }
}
