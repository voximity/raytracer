#![allow(dead_code)]
#![allow(clippy::many_single_char_names)]
#![feature(new_uninit)]

mod acceleration;
mod camera;
mod lighting;
mod material;
mod math;
mod object;
mod scene;
mod skybox;

use std::time::Instant;

use camera::Camera;
use material::{Color, Material, Texture};
use math::Vector3;
use scene::Scene;

fn main() {
    println!("Initializing scene");
    let start_time = Instant::now();

    let skybox_tex = image::open("assets/skybox.jpg").unwrap().to_rgb8();

    let mut scene = Scene {
        camera: Camera {
            vw: 1920,
            vh: 1080,
            origin: Vector3::new(4., 2., 4.),
            pitch: -0.2,
            yaw: -0.5,
            ..Default::default()
        },
        skybox: Box::new(skybox::Cubemap::new(skybox_tex)),
        ..Default::default()
    };

    // add the sun
    scene.lights.push(Box::new(lighting::Sun {
        vector: Vector3::new(-0.4, -1., 0.2).normalize(),
        ..Default::default()
    }));

    // add a plane
    scene.objects.push(Box::new(object::Plane {
        origin: Vector3::new(0., -1., 0.),
        normal: Vector3::up(),
        material: Material {
            texture: Texture::Checkerboard(Color::white(), Color::new(40, 40, 40)), //Texture::Solid(Color::new(180, 180, 180)),
            ..Default::default()
        },
        uv_wrap: 2.,
    }));

    // add a marble
    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(0., 0., 0.),
        1.,
        Material {
            texture: Texture::Solid(Color::blue()),
            reflectiveness: 0.5,
            ior: 1.333,
            transparency: 0.7,
        },
    )));

    // add the fedora behind it
    let texture_name = "assets/Handle1Tex.png";
    let obj_name = "assets/fedora.obj";

    let tex = image::open(texture_name).unwrap().to_rgb8();

    let mut obj = object::Mesh::from_obj(
        obj_name.into(),
        Material {
            texture: Texture::Image(tex),
            ..Default::default()
        },
    );
    obj.scale(2.0);
    obj.shift(Vector3::new(1.4, -3., -12.));
    obj.recalculate();
    scene.objects.push(Box::new(obj));

    // maybe some spheres behind it
    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(1.8, 0., -8.),
        1.,
        Material {
            texture: Texture::Solid(Color::red()),
            ..Default::default()
        },
    )));

    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(-1.8, 0., -8.),
        1.,
        Material {
            texture: Texture::Solid(Color::green()),
            ..Default::default()
        },
    )));

    // render out to a list of colors
    println!("Rendering scene");
    scene.render_to("render.png", image::ImageFormat::Png);

    println!(
        "Operation complete in {}s",
        start_time.elapsed().as_secs() as f32
            + start_time.elapsed().subsec_nanos() as f32 / 1000000000.
    );
}
