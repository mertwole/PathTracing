mod camera;
mod math;
mod primitives;
mod ray;
mod scene;
mod material;
use crate::camera::*;
use crate::scene::*;
use crate::math::*;
use crate::primitives::*;
use crate::material::Material;

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

    let mut materials : Vec<Material> = Vec::new();
    materials.push(
        Material 
        { 
            color : Vec3::new(1.0, 0.5, 0.25),
            emission : Vec3::new(1.0, 1.0, 1.0), 
            refraction : 1.0,
            reflective : 0.4, 
            emissive : 0.0, 
            refractive : 0.0 
        } 
    );

    scene.init_materials(materials);

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0, 0)));

    scene.iteration();

    scene.save_output(&std::path::Path::new("output.bmp"));
}
