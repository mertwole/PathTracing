use crate::camera::*;
use crate::material::*;
use crate::math::*;
use crate::raytraceable::*;

mod work_group;
use work_group::*;
mod image_buffer;

extern crate num_cpus;
extern crate threadpool;

use std::sync::mpsc::channel;
use threadpool::ThreadPool;

pub struct Scene {
    camera: Camera,
    num_threads: usize,
    trace_depth: usize,
    workgroup_size: UVec2,
    workgroup_count: UVec2,

    primitives: Vec<Box<dyn Raytraceable>>,
    materials: Vec<Box<dyn Material>>,

    workgroups: Vec<WorkGroup>,
}

static mut GLOBAL_SCENE_PTR: *const Scene = std::ptr::null();

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            camera,
            num_threads: num_cpus::get(),
            trace_depth: 8,
            workgroup_count: UVec2::new(0, 0),
            workgroup_size: UVec2::new(32, 32),
            primitives: Vec::new(),
            materials: Vec::new(),
            workgroups: Vec::new(),
        }
    }

    // region getters and setters

    pub fn add_primitive(&mut self, primitive: Box<dyn Raytraceable>) {
        self.primitives.push(primitive);
    }

    pub fn init_materials(&mut self, materials: Vec<Box<dyn Material>>) {
        self.materials = materials;
    }

    pub fn set_workgroup_size(&mut self, width: usize, height: usize) {
        self.workgroup_size = UVec2::new(width, height);
    }

    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.num_threads = num_threads;
    }

    pub fn set_trace_depth(&mut self, trace_depth: usize) {
        self.trace_depth = trace_depth;
    }

    // endregion

    fn divide_to_workgroups(&mut self) {
        self.workgroups = Vec::new();

        // Number of full-widthed and full-heighted workgroups
        self.workgroup_count = self.camera.resolution / self.workgroup_size;
        let remainder = self.camera.resolution - self.workgroup_count * self.workgroup_size;
        if remainder.x != 0 {
            self.workgroup_count.x += 1;
        }
        if remainder.y != 0 {
            self.workgroup_count.y += 1;
        }

        self.workgroups
            .reserve(self.workgroup_count.x * self.workgroup_count.y);

        for row_id in 0..self.workgroup_count.y {
            let mut row_height = self.workgroup_size.y;
            // Last row can be not full-heighted
            if row_id == self.workgroup_count.y && remainder.y != 0 {
                row_height = remainder.y;
            }

            for column_id in 0..self.workgroup_count.x {
                let mut column_width = self.workgroup_size.x;
                // Last column can be not full-widthed
                if column_id == self.workgroup_count.x && remainder.x != 0 {
                    column_width = remainder.x;
                }

                let workgroup = WorkGroup::new(
                    column_id * self.workgroup_size.x,
                    row_id * self.workgroup_size.y,
                    column_width,
                    row_height,
                );

                self.workgroups.push(workgroup);
            }
        }
    }

    pub fn init(&mut self) {
        self.divide_to_workgroups();
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
        let pool = ThreadPool::new(self.num_threads);

        for _ in 0..num_iterations {
            let mut workgroups_received: Vec<Option<WorkGroup>> = Vec::new();
            for _ in 0..self.workgroup_count.x * self.workgroup_count.y {
                workgroups_received.push(None);
            }

            for i in 0..self.workgroup_count.x * self.workgroup_count.y {
                let tx_ = tx.clone();
                let mut workgroup = self.workgroups.remove(0);
                let trace_depth = self.trace_depth;
                pool.execute(move || {
                    unsafe {
                        workgroup.iteration(GLOBAL_SCENE_PTR.as_ref().unwrap(), trace_depth);
                    }
                    tx_.send((workgroup, i)).unwrap();
                });
            }
            for _ in 0..self.workgroup_count.x * self.workgroup_count.y {
                let (workgroup, id) = rx.recv().unwrap();
                workgroups_received[id] = Some(workgroup);
            }
            for workgroup in workgroups_received {
                self.workgroups.push(workgroup.ok_or("").unwrap());
            }
        }
    }

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
                        let glob_x = x * self.workgroup_size.x + buf_x;
                        let mut glob_y = y * self.workgroup_size.y + buf_y;
                        glob_y = self.camera.resolution.y - glob_y - 1;
                        let glob_adress = glob_x + glob_y * self.camera.resolution.x;

                        buffer[glob_adress * 3] = buf_pixel.r;
                        buffer[glob_adress * 3 + 1] = buf_pixel.g;
                        buffer[glob_adress * 3 + 2] = buf_pixel.b;
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
                        let glob_x = x * self.workgroup_size.x + buf_x;
                        let glob_y = y * self.workgroup_size.y + buf_y;
                        let glob_adress = glob_x + glob_y * self.camera.resolution.x;

                        buffer[glob_adress] = (buf_pixel.r as u32)
                            + 256 * (buf_pixel.g as u32)
                            + 256 * 256 * (buf_pixel.b as u32);
                    }
                }
            }
        }

        buffer
    }
}
