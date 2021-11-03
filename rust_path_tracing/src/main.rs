extern crate kd_tree;
extern crate math;
extern crate rand;

mod renderer;
mod camera;
mod material;
mod raytraceable;
mod ray;
mod scene;
use crate::camera::*;
use crate::material::*;
use crate::raytraceable::*;
use crate::scene::*;
use crate::math::*;
use crate::renderer::*;

use std::thread;

static mut FRAMEBUFFER_PTR : *const u32 = std::ptr::null();

fn init_materials(scene : &mut Scene) {
    let mut materials: Vec<Box<dyn Material>> = Vec::new();

    // 0th mat (emissive)
    let mut material = BaseMaterial::default();
    material.emissive = 1.0;
    material.emission = Vec3::new_xyz(5.0);
    materials.push(Box::new(material));
    // 1th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.8, 0.2, 0.1);
    material.reflective = 0.01;
    materials.push(Box::new(material));
    // 2th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.1, 0.2, 0.8);
    material.reflective = 0.01;
    materials.push(Box::new(material));
    // 3th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(1.0, 1.0, 1.0);
    material.reflective = 0.0;
    materials.push(Box::new(material));
    // 4th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(1.0, 0.0, 0.0);
    material.reflective = 0.01;
    materials.push(Box::new(material));
    // 5th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.7, 0.7, 0.7);
    material.reflective = 0.0;
    materials.push(Box::new(material));
    // 6th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(1.0, 1.0, 1.0);
    material.reflective = 0.9;
    materials.push(Box::new(material));
    // 7th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.11, 0.0);
    materials.push(Box::new(material));
    // 8th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.11, 0.3);
    materials.push(Box::new(material));
    // 9th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.11, 0.5);
    materials.push(Box::new(material));
    // 10th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.11, 0.7);
    materials.push(Box::new(material));
    // 11th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.11, 1.0);
    materials.push(Box::new(material));

    scene.init_materials(materials);
}

fn init_primitives(scene : &mut Scene) {
    // floor
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 5)));
    // ceiling
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0)));
    //scene.add_primitive(Box::new(Plane::new(Vec3::new(-30.0, 30.0, 0.0), Vec3::new(1.0, -1.0, 0.0).normalized(), 0)));
    // walls
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0), 3)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 1)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), 2)));

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-1.9, -2.0, 0.0), 1.0, 7)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(1.9, -2.0, 0.0), 1.0, 8)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, -0.5, 0.0), 1.0, 9)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-1.9, 1.0, 0.0), 1.0, 10)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(1.9, 1.0, 0.0), 1.0, 11)));

    // let mut kd_tree = KDTree::new(1);
    // kd_tree.load(&"data/stanford-dragon.obj".to_string(), &"data/stanford-dragon.tree".to_string());
    // kd_tree.load(&"data/cube.obj".to_string(), &"data/cube.tree".to_string());
    // scene.add_primitive(Box::new(kd_tree));
}

fn main() {
    let screen_width = 1024u32;
    let screen_height = 1024u32;

    let camera = Camera {
        resolution : UVec2::new(screen_width as usize, screen_height as usize),
        rotation: Mat3::create_rotation_x(0.0 / 180.0 * math::PI),
        position: Vec3::new(0.0, 0.0, 15.0),

        fov : 30.0 / 180.0 * math::PI,
        near_plane : 0.0,
        focal_length : 15.0,

        bokeh_shape : BokehShape::Square,
        bokeh_size : 0.0
    };

    let mut scene = Scene::new(camera);
    scene.set_trace_depth(8);
    scene.set_workgroup_size(16, 16);
    scene.init();

    init_materials(&mut scene);
    init_primitives(&mut scene);

    let mut window = Window::open(WindowParameters { width : screen_width, height : screen_height, title : String::from("title")});
    let mut renderer = Renderer::new(screen_width, screen_height);

    let mut reset_render = true;

    thread::spawn(move || {
        let mut iterations = 0;
        let mut raw_img = Vec::new();

        loop {
            if reset_render { scene.init(); reset_render = false; }

            scene.iterations(16);
            println!("iteration {}", iterations);
            iterations += 1;
            unsafe { FRAMEBUFFER_PTR = std::ptr::null(); }
            raw_img = scene.get_raw_image();
            unsafe { FRAMEBUFFER_PTR = raw_img.as_ptr(); }
        }
    });

    loop {
        let events = window.process_events();
        if window.should_close() { break; }

        unsafe {
            if FRAMEBUFFER_PTR != std::ptr::null() {
                renderer.render_from_raw(&mut window, screen_width, screen_height, FRAMEBUFFER_PTR);
            }
        }
    }

    //scene.save_output(&std::path::Path::new("output.bmp"));
}