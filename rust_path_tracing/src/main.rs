mod camera;
mod material;
mod math;
mod primitives;
mod ray;
mod scene;
use crate::camera::*;
use crate::material::Material;
use crate::math::*;
use crate::primitives::*;
use crate::scene::*;

fn main() {
    let camera = Camera {
        width: 1920,
        height: 1080,
        position: Vec3::new(0.0, -3.0 + 3.375 * 0.5, 10.0),
        view_distance: 7.1,
        viewport: Vec2::new(5.99, 3.375),
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

    scene.init_materials(materials);

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.0, -2.0, 0.0), 1.0, 2)));

    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, -3.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, 3.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        1,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        0,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        2,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, -3.0),
        Vec3::new(0.0, 0.0, 1.0),
        0,
    )));
    scene.add_primitive(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, 3.0),
        Vec3::new(0.0, 0.0, -1.0),
        0,
    )));

    for _i in 0..512 {
        scene.iteration();
    }

    scene.save_output(&std::path::Path::new("output.bmp"));
}
