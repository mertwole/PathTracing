mod camera;
mod math;
mod primitives;
mod ray;
mod scene;
use crate::camera::*;
use crate::scene::*;
use crate::math::*;
use crate::primitives::*;

fn main() {
    let camera = Camera{ 
        width : 1920, 
        height : 1080, 
        position : Vec3::new(0.0, 0.0, 10.0),
        view_distance : 5f32,
        viewport : Vec2::new(1.6, 0.9),
        rotation : Mat3::create_rotation(0.0, 0.0, 0.0)
    };
    let mut scene = Scene::new(camera);

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0)));

    scene.iteration();

    scene.save_output(&std::path::Path::new("output.bmp"));
}
