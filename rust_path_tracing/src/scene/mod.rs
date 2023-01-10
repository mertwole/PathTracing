use serde::Deserialize;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

use crate::material::{self, Material, MaterialUninit};
use crate::raytraceable::Raytraceable;
use math::{Color24bpprgb, UVec2};

pub mod camera;
mod image_buffer;
mod work_group;

use camera::Camera;
use work_group::WorkGroup;

#[derive(Deserialize)]
struct SceneConfig {
    num_threads: usize,
    trace_depth: usize,
    workgroup_size: UVec2,
}

pub type Scene = SceneGeneric<Box<dyn Material>>;

#[derive(Deserialize)]
#[serde(transparent)]
struct SceneUninit(SceneGeneric<Box<dyn MaterialUninit>>);

#[derive(Deserialize)]
pub struct SceneGeneric<M> {
    config: SceneConfig,
    camera: Camera,
    #[serde(skip)]
    primitives: Vec<Box<dyn Raytraceable>>,
    materials: Vec<M>,

    #[serde(skip)]
    workgroup_count: UVec2,
    #[serde(skip)]
    workgroups: Vec<WorkGroup>,
}

static mut GLOBAL_SCENE_PTR: *const Scene = std::ptr::null();

impl SceneUninit {
    pub fn init(mut self) -> Scene {
        let materials = self.0.materials.drain(..).map(|mat| mat.init()).collect();

        Scene {
            config: self.0.config,
            camera: self.0.camera,
            primitives: self.0.primitives,
            materials,
            workgroup_count: self.0.workgroup_count,
            workgroups: self.0.workgroups,
        }
    }
}

impl Scene {
    pub fn from_json(json: &str) -> Scene {
        let uninit: SceneUninit = serde_json::de::from_str(json).unwrap();
        let mut init = uninit.init();
        // @TODO: Move to SceneUninit::init.
        init.divide_to_workgroups();
        init
    }

    pub fn add_primitive(&mut self, primitive: Box<dyn Raytraceable>) {
        self.primitives.push(primitive);
    }

    fn divide_to_workgroups(&mut self) {
        self.workgroups = Vec::new();

        // Number of full-widthed and full-heighted workgroups
        self.workgroup_count = self.camera.resolution / self.config.workgroup_size;
        let remainder = self.camera.resolution - self.workgroup_count * self.config.workgroup_size;
        if remainder.x != 0 {
            self.workgroup_count.x += 1;
        }
        if remainder.y != 0 {
            self.workgroup_count.y += 1;
        }

        self.workgroups
            .reserve(self.workgroup_count.x * self.workgroup_count.y);

        for row_id in 0..self.workgroup_count.y {
            let mut row_height = self.config.workgroup_size.y;
            // Last row can be not full-heighted
            if row_id == self.workgroup_count.y && remainder.y != 0 {
                row_height = remainder.y;
            }

            for column_id in 0..self.workgroup_count.x {
                let mut column_width = self.config.workgroup_size.x;
                // Last column can be not full-widthed
                if column_id == self.workgroup_count.x && remainder.x != 0 {
                    column_width = remainder.x;
                }

                let workgroup = WorkGroup::new(
                    column_id * self.config.workgroup_size.x,
                    row_id * self.config.workgroup_size.y,
                    column_width,
                    row_height,
                );

                self.workgroups.push(workgroup);
            }
        }
    }

    pub fn init(&mut self) {
        // @TODO: Remove this.
        unsafe {
            GLOBAL_SCENE_PTR = self;
        }
    }

    pub fn iterations(&mut self, num_iterations: usize) {
        unsafe {
            if GLOBAL_SCENE_PTR.is_null() {
                panic!("scene is not initialized!");
            }
        }

        let (tx, rx): (
            std::sync::mpsc::Sender<(WorkGroup, usize)>,
            std::sync::mpsc::Receiver<(WorkGroup, usize)>,
        ) = channel();
        let pool = ThreadPool::new(self.config.num_threads);

        for _ in 0..num_iterations {
            let workgroup_count = self.workgroup_count.x * self.workgroup_count.y;

            let mut workgroups_received: Vec<Option<WorkGroup>> = Vec::new();
            for _ in 0..workgroup_count {
                workgroups_received.push(None);
            }

            for i in 0..workgroup_count {
                let tx_ = tx.clone();
                let mut workgroup = self.workgroups.remove(0);
                let trace_depth = self.config.trace_depth;
                pool.execute(move || {
                    unsafe {
                        workgroup.iteration(GLOBAL_SCENE_PTR.as_ref().unwrap(), trace_depth);
                    }
                    tx_.send((workgroup, i)).unwrap();
                });
            }
            for _ in 0..workgroup_count {
                let (workgroup, id) = rx.recv().unwrap();
                workgroups_received[id] = Some(workgroup);
            }
            for workgroup in workgroups_received {
                self.workgroups.push(workgroup.ok_or("").unwrap());
            }
        }
    }

    // FIXME: Code duplication.
    pub fn save_output(&self, path: &std::path::Path) {
        let mut buffer: Vec<u8> =
            vec![0u8; self.camera.resolution.x * self.camera.resolution.y * 3];

        for x in 0..self.workgroup_count.x {
            for y in 0..self.workgroup_count.y {
                let workgroup_buffer =
                    self.workgroups[x + y * self.workgroup_count.x].get_raw_image_data();

                for buf_x in 0..workgroup_buffer.len() {
                    for buf_y in 0..workgroup_buffer[0].len() {
                        let buf_pixel = workgroup_buffer[buf_x][buf_y];
                        let glob_x = x * self.config.workgroup_size.x + buf_x;
                        let mut glob_y = y * self.config.workgroup_size.y + buf_y;
                        glob_y = self.camera.resolution.y - glob_y - 1;
                        let glob_adress = glob_x + glob_y * self.camera.resolution.x;

                        let buf_color = Color24bpprgb::from_hdr_tone_mapped(buf_pixel);

                        buffer[glob_adress * 3] = buf_color.r;
                        buffer[glob_adress * 3 + 1] = buf_color.g;
                        buffer[glob_adress * 3 + 2] = buf_color.b;
                    }
                }
            }
        }

        image::save_buffer_with_format(
            path,
            &buffer,
            self.camera.resolution.x as u32,
            self.camera.resolution.y as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Bmp,
        )
        .unwrap();
    }

    pub fn get_raw_image(&self) -> Vec<u32> {
        let mut buffer: Vec<u32> = vec![0u32; self.camera.resolution.x * self.camera.resolution.y];

        for x in 0..self.workgroup_count.x {
            for y in 0..self.workgroup_count.y {
                let workgroup_buffer =
                    self.workgroups[x + y * self.workgroup_count.x].get_raw_image_data();

                for buf_x in 0..workgroup_buffer.len() {
                    for buf_y in 0..workgroup_buffer[0].len() {
                        let buf_pixel = workgroup_buffer[buf_x][buf_y];
                        let glob_x = x * self.config.workgroup_size.x + buf_x;
                        let glob_y = y * self.config.workgroup_size.y + buf_y;
                        let glob_adress = glob_x + glob_y * self.camera.resolution.x;

                        let buf_color = Color24bpprgb::from_hdr_tone_mapped(buf_pixel);

                        buffer[glob_adress] = (buf_color.r as u32)
                            + 256 * (buf_color.g as u32)
                            + 256 * 256 * (buf_color.b as u32);
                    }
                }
            }
        }

        buffer
    }
}
