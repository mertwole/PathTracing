mod camera;
mod math;
mod primitives;
mod ray;
mod scene;
use crate::camera::*;
use crate::scene::*;

fn main() {
    let camera = Camera::new(1920, 1080);
    let mut scene = Scene::new(camera);

    scene.iteration();

    scene.save_output(&std::path::Path::new("output.bmp"));
}
