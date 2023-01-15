use std::{
    sync::{Arc, Mutex},
    thread,
};

mod material;
pub mod obj_loader;
mod ray;
mod raytraceable;
mod renderer;
mod scene;

use renderer::Renderer;
use scene::Scene;

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

    let framebuffer = Arc::new(Framebuffer::new());
    let framebuffer_clone = framebuffer.clone();

    let renderer = Renderer::new(screen_width, screen_height);

    thread::spawn(move || {
        let mut scene = Scene::from_json(&std::fs::read_to_string("./sample_scene.json").unwrap());

        let mut iterations = 0;
        loop {
            scene.iterations(1);
            println!("iteration {}", iterations);
            iterations += 1;

            let image = scene.get_raw_image();
            framebuffer_clone.set_image(image);
        }
    });

    renderer.enter_rendering_loop(framebuffer);
}
