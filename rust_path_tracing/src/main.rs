use std::{
    sync::{Arc, Mutex},
    thread,
};

mod material;
mod ray;
mod raytraceable;
mod renderer;
mod scene;

use math::Vec3;
use raytraceable::{plane::Plane, sphere::Sphere};
use renderer::Renderer;
use scene::Scene;

fn init_primitives(scene: &mut Scene) {
    // floor
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, -3.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        3,
    )));
    // ceiling
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, 3.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        0,
    )));
    // walls
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, -3.0),
        Vec3::new(0.0, 0.0, 1.0),
        3,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        1,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        2,
    )));

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-1.0, -2.0, 0.0), 1.0, 4)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.5, 0)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(1.0, -2.0, 0.0), 1.0, 4)));

    // let mut kd_tree = KDTree::new(1);
    // kd_tree.load(&"data/stanford-dragon.obj".to_string(), &"data/stanford-dragon.tree".to_string());
    // kd_tree.load(&"data/cube.obj".to_string(), &"data/cube.tree".to_string());
    // scene.add_primitive(Box::new(kd_tree));
}

pub struct Framebuffer {
    data: Mutex<Vec<u32>>,
}

impl Framebuffer {
    pub fn new() -> Framebuffer {
        Framebuffer {
            data: Mutex::new(vec![]),
        }
    }

    pub fn set_image(&self, data: Vec<u32>) {
        let mut guard = self.data.lock().unwrap();
        *guard = data;
    }

    pub fn get_image_pointer(&self) -> Option<*const u32> {
        let data = self.data.lock().unwrap();
        if data.len() != 0 {
            Some((*data).as_ptr() as *const u32)
        } else {
            None
        }
    }
}

fn main() {
    let screen_width = 1024u32;
    let screen_height = 1024u32;

    let mut scene = Scene::from_json(&std::fs::read_to_string("./sample_scene.json").unwrap());
    scene.init();

    init_primitives(&mut scene);

    let framebuffer = Arc::new(Framebuffer::new());
    let framebuffer_clone = framebuffer.clone();

    let renderer = Renderer::new(screen_width, screen_height);

    let mut reset_render = true;

    thread::spawn(move || {
        let mut iterations = 0;
        loop {
            if reset_render {
                scene.init();
                reset_render = false;
            }

            scene.iterations(1);
            println!("iteration {}", iterations);
            iterations += 1;

            let image = scene.get_raw_image();
            framebuffer_clone.set_image(image);
        }
    });

    renderer.enter_rendering_loop(framebuffer);

    //scene.save_output(&std::path::Path::new("output.bmp"));
}
