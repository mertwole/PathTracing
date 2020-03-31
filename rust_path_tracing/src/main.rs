extern crate kd_tree;
extern crate math;

mod camera;
mod material;
mod raytraceable;
mod ray;
mod scene;
use crate::camera::*;
use crate::material::Material;
use crate::raytraceable::*;
use crate::scene::*;
use crate::math::*;

fn main() {
    let camera = Camera {
        width: 1000,
        height: 1000,
        position: Vec3::new(0.0, 0.0, 10.0),
        view_distance: 7.1,
        viewport: Vec2::new(5.99, 5.99),
        rotation: Mat3::create_rotation(0.0, 3.14, 0.0),
    };
    let mut scene = Scene::new(camera);

    let mut materials: Vec<Material> = Vec::new();
    materials.push(Material {
        color: Vec3::new(1.0, 0.5, 0.25),
        emission: Vec3::new(1.0, 1.0, 1.0),
        refraction: 1.0,
        reflective: 0.0,
        emissive: 0.0,
        refractive: 0.0,
    });
    materials.push(Material {
        color: Vec3::new(1.0, 0.5, 0.25),
        emission: Vec3::new(1.0, 1.0, 1.0),
        refraction: 1.0,
        reflective: 0.0,
        emissive: 1.0,
        refractive: 0.0,
    });
    materials.push(Material {
        color: Vec3::new(1.0, 1.0, 1.0),
        emission: Vec3::new(1.0, 1.0, 1.0),
        refraction: 1.0,
        reflective: 0.3,
        emissive: 0.0,
        refractive: 0.0,
    });
    materials.push(Material {
        color: Vec3::new(0.2, 0.4, 0.8),
        emission: Vec3::new(1.0, 1.0, 1.0),
        refraction: 1.0,
        reflective: 0.3,
        emissive: 0.0,
        refractive: 0.0,
    });

    scene.init_materials(materials);

    //scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, -2.0, 0.0), 1.0, 2)));

    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, -3.0, 0.0),Vec3::new(0.0, 1.0, 0.0),0)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 3.0, 0.0),Vec3::new(0.0, -1.0, 0.0),1)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(-3.0, 0.0, 0.0),Vec3::new(1.0, 0.0, 0.0),0)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(3.0, 0.0, 0.0),Vec3::new(-1.0, 0.0, 0.0),2)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, -3.0),Vec3::new(0.0, 0.0, 1.0),0)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, 3.0),Vec3::new(0.0, 0.0, -1.0),0)));
    
    let mut kd_tree = KDTree::new(3);
    kd_tree.load(&"data/stanford-dragon.obj".to_string(), &"data/stanford-dragon.tree".to_string());
    scene.add_primitive(Box::new(kd_tree));

    for _i in 0..10 {
        scene.iteration();
    }

    scene.save_output(&std::path::Path::new("output.bmp"));
}
