extern crate kd_tree;
extern crate math;

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

use std::time::SystemTime;

fn main() {
    let camera = Camera {
        width: 1000,
        height: 1000,
        position: Vec3::new(0.0, 0.0, 10.0),
        view_distance: 7.1,
        viewport: Vec2::new(5.99, 5.99),
        rotation: Mat3::create_rotation(0.0, 0.0, 0.0)
    };
    let mut scene = Scene::new(camera);

    let mut materials: Vec<Box<dyn Material>> = Vec::new();

    // 0th mat
    let material = PBRMaterial::new(Vec3::new(1.0, 1.0, 1.0), 0.1, 0.5);
    materials.push(Box::new(material));
    // 1th mat
    let mut material = BaseMaterial::default();
    material.emissive = 1.0;
    materials.push(Box::new(material));
    // 2th mat
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.2, 0.4, 0.8);
    material.reflective = 0.0;
    materials.push(Box::new(material));

    scene.init_materials(materials);

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, -2.0, 0.0), 1.0, 0)));

    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, -3.0, 0.0),Vec3::new(0.0, 1.0, 0.0),2)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 3.0, 0.0),Vec3::new(0.0, -1.0, 0.0),1)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(-3.0, 0.0, 0.0),Vec3::new(1.0, 0.0, 0.0),2)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(3.0, 0.0, 0.0),Vec3::new(-1.0, 0.0, 0.0),2)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, -3.0),Vec3::new(0.0, 0.0, 1.0),2)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, 3.0),Vec3::new(0.0, 0.0, -1.0),2)));
    
    //let mut kd_tree = KDTree::new(2);
    //kd_tree.load(&"data/stanford-dragon.obj".to_string(), &"data/stanford-dragon.tree".to_string());
    //scene.add_primitive(Box::new(kd_tree));
    
    scene.set_trace_depth(16);
    let start_trace_time = SystemTime::now();
    scene.iterations(512);
    println!("traced for {} secs", start_trace_time.elapsed().unwrap().as_secs_f32());

    scene.save_output(&std::path::Path::new("output.bmp"));
}
