use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use image::RgbaImage;
use threadpool::ThreadPool;

use crate::{api::render_task::RenderTask, ray::Ray, render_store::RenderStore, scene::Scene};
use math::{Color24bpprgb, UVec2, Vec2, Vec3};

use super::Renderer;

mod image_buffer;
mod work_group;
use work_group::WorkGroup;

pub struct CPURenderer {
    scene: Arc<Scene>,

    workgroup_count: UVec2,
    workgroup_size: UVec2,
    workgroups: Vec<WorkGroup>,
    thread_pool: ThreadPool,
}

#[async_trait::async_trait]
impl Renderer for CPURenderer {
    fn init(scene: Arc<Scene>) -> CPURenderer {
        CPURenderer {
            scene,
            workgroup_count: UVec2::default(),
            workgroup_size: UVec2::new(32, 32),
            workgroups: vec![],
            thread_pool: ThreadPool::new(num_cpus::get()),
        }
    }

    async fn render(&mut self, render_task: Arc<RenderTask>, render_store: &RenderStore) {
        (self.workgroup_count, self.workgroups) = self.divide_to_workgroups(&*render_task);
        self.iterations(render_task.clone());
        let image = self.get_image(&*render_task);
        render_store.save_render(&render_task, image).await;
    }
}

pub struct RayTraceResult {
    pub hit: bool,
    pub hit_inside: bool,

    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub t: f32,

    pub material_id: usize,
}

impl RayTraceResult {
    pub fn void() -> RayTraceResult {
        RayTraceResult {
            hit: false,
            point: Vec3::default(),
            normal: Vec3::default(),
            uv: Vec2::default(),
            t: 0.0,
            material_id: 0,
            hit_inside: false,
        }
    }
}

pub trait SceneNode: Send + Sync {
    fn trace_ray(&self, ray: &Ray) -> RayTraceResult;
}

pub enum GetColorResult {
    Color(Vec3),
    NextRayColorMultiplierAndDirection(Vec3, Vec3),
}

pub trait Material: Send + Sync {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult;
}

impl CPURenderer {
    pub fn iterations(&mut self, render_task: Arc<RenderTask>) {
        let (tx, rx): (Sender<(WorkGroup, usize)>, Receiver<(WorkGroup, usize)>) = channel();

        for _ in 0..render_task.config.iterations {
            let workgroup_count = self.workgroup_count.x * self.workgroup_count.y;

            let mut workgroups_received = vec![];
            for _ in 0..workgroup_count {
                workgroups_received.push(None);
            }

            for i in 0..workgroup_count {
                let tx_ = tx.clone();
                let mut workgroup = self.workgroups.remove(0);
                let scene = self.scene.clone();
                let render_task = render_task.clone();
                self.thread_pool.execute(move || {
                    workgroup.iteration(scene.clone(), render_task);
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

    pub fn trace_ray(&self, ray: &Ray) -> RayTraceResult {
        self.scene.hierarchy.trace_ray(ray)
    }

    fn divide_to_workgroups(&self, render_task: &RenderTask) -> (UVec2, Vec<WorkGroup>) {
        let mut workgroups = Vec::new();

        // Number of full-widthed and full-heighted workgroups
        let mut workgroup_count = render_task.camera.resolution / self.workgroup_size;
        let remainder = render_task.camera.resolution - workgroup_count * self.workgroup_size;
        if remainder.x != 0 {
            workgroup_count.x += 1;
        }
        if remainder.y != 0 {
            workgroup_count.y += 1;
        }

        workgroups.reserve(workgroup_count.x * workgroup_count.y);

        for row_id in 0..workgroup_count.y {
            let mut row_height = self.workgroup_size.y;
            // Last row can be not full-heighted
            if row_id == workgroup_count.y && remainder.y != 0 {
                row_height = remainder.y;
            }

            for column_id in 0..workgroup_count.x {
                let mut column_width = self.workgroup_size.x;
                // Last column can be not full-widthed
                if column_id == workgroup_count.x && remainder.x != 0 {
                    column_width = remainder.x;
                }

                let workgroup = WorkGroup::new(
                    column_id * self.workgroup_size.x,
                    row_id * self.workgroup_size.y,
                    column_width,
                    row_height,
                );

                workgroups.push(workgroup);
            }
        }

        (workgroup_count, workgroups)
    }

    pub fn get_image(&self, render_task: &RenderTask) -> RgbaImage {
        let mut buffer: Vec<u8> =
            vec![0u8; render_task.camera.resolution.x * render_task.camera.resolution.y * 4];

        for x in 0..self.workgroup_count.x {
            for y in 0..self.workgroup_count.y {
                let workgroup_buffer =
                    self.workgroups[x + y * self.workgroup_count.x].get_raw_image_data();

                for buf_x in 0..workgroup_buffer.len() {
                    for buf_y in 0..workgroup_buffer[0].len() {
                        let buf_pixel = workgroup_buffer[buf_x][buf_y];
                        let glob_x = x * self.workgroup_size.x + buf_x;
                        let glob_y = y * self.workgroup_size.y + buf_y;
                        let glob_adress = glob_x + glob_y * render_task.camera.resolution.x;

                        let buf_color = Color24bpprgb::from_hdr_tone_mapped(buf_pixel);

                        buffer[glob_adress * 4] = buf_color.r;
                        buffer[glob_adress * 4 + 1] = buf_color.g;
                        buffer[glob_adress * 4 + 2] = buf_color.b;
                        buffer[glob_adress * 4 + 3] = 255;
                    }
                }
            }
        }

        RgbaImage::from_raw(
            render_task.camera.resolution.x as u32,
            render_task.camera.resolution.y as u32,
            buffer,
        )
        .unwrap()
    }
}
