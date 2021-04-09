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

fn init_materials(scene : &mut Scene) {
    let mut materials: Vec<Box<dyn Material>> = Vec::new();

    // 0th mat (emissive)
    let mut material = BaseMaterial::default();
    material.emissive = 1.0;
    materials.push(Box::new(material));
    // 1th mat (red)
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(1.0, 0.0, 0.0);
    materials.push(Box::new(material));
    // 2th mat (green)
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.0, 1.0, 0.0);
    materials.push(Box::new(material));
    // 3th mat (grey)
    let mut material = BaseMaterial::default();
    material.color = &Vec3::new(1.0, 1.0, 1.0) * 0.9;
    materials.push(Box::new(material));
    // 4th mat (reflective)
    let mut material = BaseMaterial::default();
    material.color = Vec3::new(0.8, 0.4, 0.2);
    material.reflective = 1.0;
    materials.push(Box::new(material));
    // 5th mat (refractive)
    let mut material = BaseMaterial::default();
    material.refractive = 1.0;
    material.reflective = 0.0;
    material.refraction = 1.01;
    materials.push(Box::new(material));
    // 6th mat (emissive)
    let mut material = BaseMaterial::default();
    material.emission = Vec3::new(1.0, 0.0, 0.0);
    material.emissive = 1.0;
    materials.push(Box::new(material));
    // 7th mat (emissive)
    let mut material = BaseMaterial::default();
    material.emission = Vec3::new(0.0, 1.0, 0.0);
    material.emissive = 1.0;
    materials.push(Box::new(material));
    // 8th mat (emissive)
    let mut material = BaseMaterial::default();
    material.emission = Vec3::new(0.0, 0.0, 1.0);
    material.emissive = 1.0;
    materials.push(Box::new(material));

    scene.init_materials(materials);
}

fn main() {
    let camera = Camera {
        resolution : UVec2::new(1024, 1024),
        rotation: Mat3::create_rotation(0.0, 0.0, 0.0),
        position: Vec3::new(0.0, 0.0, 1.0),

        fov : 80.0 / 180.0 * math::PI,
        near_plane : 0.0,
        focal_length : 5.5,

        bokeh_shape : BokehShape::Circle,
        bokeh_size : 0.3
    };

    let mut scene = Scene::new(camera);
    scene.set_trace_depth(16);
    scene.init();

    init_materials(&mut scene);

    // box
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, -3.0, 0.0),Vec3::new(0.0, 1.0, 0.0),3)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 3.0, 0.0),Vec3::new(0.0, -1.0, 0.0),0)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(-3.0, 0.0, 0.0),Vec3::new(1.0, 0.0, 0.0),3)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(3.0, 0.0, 0.0),Vec3::new(-1.0, 0.0, 0.0),3)));
    scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, -3.0),Vec3::new(0.0, 0.0, 1.0),3)));
    //scene.add_primitive(Box::new(Plane::new(Vec3::new(0.0, 0.0, 3.0),Vec3::new(0.0, 0.0, -1.0),3)));

    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-2.5, -2.5, -2.5), 0.5, 3)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-1.5, -2.5, -1.5), 0.5, 6)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(-0.5, -2.5, -0.5), 0.5, 3)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(0.5, -2.5, 0.5), 0.5, 4)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(1.5, -2.5, 1.5), 0.5, 3)));
    scene.add_primitive(Box::new(Sphere::new(Vec3::new(2.5, -2.5, 2.5), 0.5, 3)));

    //scene.add_primitive(Box::new(Sphere::new(Vec3::new(2.5, -2.5, -2.5), 1.0, 6)));
    //scene.add_primitive(Box::new(Sphere::new(Vec3::new(-1.5, -2.5, 1.5), 0.5, 7)));

    //let mut kd_tree = KDTree::new(0);
    //kd_tree.load(&"data/stanford-dragon.obj".to_string(), &"data/stanford-dragon.tree".to_string());
    //scene.add_primitive(Box::new(kd_tree));

    let screen_width = 1024;
    let screen_height = 1024;

    let mut window = Window::open(WindowParameters { width : screen_width, height : screen_height, title : String::from("title")});
    let mut renderer = Renderer::new(screen_width, screen_height);

    let mut iteration = 0;

    loop {
        window.process_events();
        if window.should_close() { break; }

        if iteration <= 20000 { 
            iteration += 1;
            scene.iterations(1);
            println!("iteration {}", iteration);
        }

        renderer.render_from_raw(&mut window, screen_width, screen_height, scene.get_raw_image().as_ptr());
    }

    scene.save_output(&std::path::Path::new("output.bmp"));
}