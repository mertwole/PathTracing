use crate::camera::*;
use crate::material::*;
use crate::raytraceable::*;

mod work_group;
use work_group::*;
mod image_buffer;

extern crate image;
extern crate num_cpus;

pub struct Scene {
    camera: Camera,
    num_threads : usize,
    trace_depth : usize,
    workgroup_width : usize,
    workgroup_height : usize,


    iteration: usize,

    primitives: Vec<Box<dyn Raytraceable>>,
    materials: Vec<Box<dyn Material>>,

    workgroups : Vec<Vec<WorkGroup>>
}

impl Scene {
    pub fn new(camera: Camera) -> Scene {
        Scene {
            camera,
            num_threads : num_cpus::get_physical(),
            trace_depth : 8,
            workgroup_width : 32,
            workgroup_height : 32,
            iteration: 0usize,
            primitives: Vec::new(),
            materials: Vec::new(),
            workgroups : Vec::new()
        }
    }

    // region getters and setters

    pub fn add_primitive(&mut self, primitive: Box<dyn Raytraceable>) {
        self.primitives.push(primitive);
    }

    pub fn init_materials(&mut self, materials: Vec<Box<dyn Material>>) {
        self.materials = materials;
    }

    pub fn set_workgroup_size(&mut self, workgroup_width : usize, workgroup_height : usize){
        self.workgroup_width = workgroup_width;
        self.workgroup_height = workgroup_height;
    }

    pub fn set_num_threads(&mut self, num_threads : usize) { 
        self.num_threads = num_threads; 
    }

    pub fn set_trace_depth(&mut self, trace_depth : usize) { 
        self.trace_depth = trace_depth; 
    }

    // endregion

    fn bring_to_workgroups(&mut self) -> Vec<Vec<WorkGroup>>{
        let mut workgroups : Vec<Vec<WorkGroup>> = Vec::new();

        let num_workgroups_x = self.camera.width / self.workgroup_width;
        let num_workgroups_y = self.camera.height / self.workgroup_height;
        let remainder_width = self.camera.width - num_workgroups_x * self.workgroup_width;
        let remainder_height = self.camera.height - num_workgroups_y * self.workgroup_height;

        workgroups.reserve(num_workgroups_y);

        for column_id in 0..num_workgroups_x + 1{ 
            let mut column_width = self.workgroup_width;
            if column_id == num_workgroups_x {
                if remainder_width == 0 { break; } else { column_width = remainder_width; }
            }

            let mut workgroup_column : Vec<WorkGroup> = Vec::new();
            workgroup_column.reserve(num_workgroups_x);

            for row_id in 0..num_workgroups_y + 1{
                let mut row_height = self.workgroup_height;
                if row_id == num_workgroups_y {
                    if remainder_height == 0 { break; } else { row_height = remainder_height; }
                }

                let workgroup = WorkGroup::new(
                    column_id * self.workgroup_width, 
                    row_id * self.workgroup_height, 
                    column_width, 
                    row_height
                );
                workgroup_column.push(workgroup);
            }
            workgroups.push(workgroup_column);
        }
        
        workgroups
    }

    pub fn iterations(&mut self, num_iterations : usize) {
        let mut workgroups = self.bring_to_workgroups();      
        // TODO : multithreading
        for i in 0..num_iterations{
            let mut workgroup_id = 0;
            for workgroup_column in &mut workgroups{
                for workgroup in workgroup_column{
                    workgroup.iteration(self, self.trace_depth);
                    println!("iter : {} wg id : {}", i, workgroup_id);
                    workgroup_id += 1;
                }
            }
        }

        self.workgroups = workgroups;
    }

    pub fn save_output(&self, path: &std::path::Path) {
        let mut buffer: Vec<u8> = vec![0; self.camera.width * self.camera.height * 3];
        
        for x in 0..self.workgroups.len(){
            for y in 0..self.workgroups[0].len(){
                let offset_x = x * self.workgroup_width;
                let offset_y = y * self.workgroup_height;

                let workgroup = &self.workgroups[x][y];
                for img_x in 0..workgroup.buffer.width{
                    for img_y in 0..workgroup.buffer.height{
                        let glob_pixel_id = offset_x + img_x + (offset_y + img_y) * self.camera.width;
                        let pixel = workgroup.buffer.get_pixel(img_x, img_y);
                        buffer[glob_pixel_id * 3] =     (pixel.x.powf(1.0 / 2.2) * 255f32) as u8;
                        buffer[glob_pixel_id * 3 + 1] = (pixel.y.powf(1.0 / 2.2) * 255f32) as u8;
                        buffer[glob_pixel_id * 3 + 2] = (pixel.z.powf(1.0 / 2.2) * 255f32) as u8;
                    }
                }
            }
        }

        image::save_buffer_with_format(
            path,
            &buffer,
            self.camera.width as u32,
            self.camera.height as u32,
            image::ColorType::Rgb8,
            image::ImageFormat::Bmp,
        ).unwrap();
    }
}
