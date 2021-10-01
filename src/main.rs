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

use std::{f64::consts::PI, time::Instant};

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
            pitch: -0.35,
            yaw: -PI / 4.,
            ..Default::default()
        },
        skybox: Box::new(skybox::Cubemap::new(skybox_tex)),
        ..Default::default()
    };

    // add a plane
    scene.objects.push(Box::new(object::Plane {
        origin: Vector3::new(0., -1., 0.),
        normal: Vector3::up(),
        material: Material {
            texture: Texture::Checkerboard(Color::white(), Color::new(40, 40, 40)), //Texture::Solid(Color::new(180, 180, 180)),
            reflectiveness: 0.,
        },
        uv_wrap: 2.,
    }));

    // add the obj in the middle
    let texture_name = "assets/raytraced-isaac.png";
    let obj_name = "assets/raytraced-isaac.obj";

    let tex = image::open(texture_name).unwrap().to_rgb8();

    let mut obj = object::Mesh::from_obj(
        obj_name.into(),
        Material {
            texture: Texture::Image(tex),
            reflectiveness: 0.,
        },
    );
    obj.scale(0.8);
    obj.shift(Vector3::new(0.6, -1., 0.));
    obj.recalculate();
    scene.objects.push(Box::new(obj));

    // add some reflective spheres around the center
    for n in 0..8 {
        let inner = n as f64 / 8. * PI * 2.;
        let cos = inner.cos();

        let color = Color::hsv(n as f32 / 8. * 360., 255, 255);
        let sin = inner.sin();

        let light = lighting::Point {
            color,
            intensity: 4.,
            position: Vector3::new(cos * 5., 2., sin * 5.),
            ..Default::default()
        };

        let sphere = object::Sphere::new(
            Vector3::new(cos * 8., 1., sin * 8.),
            2.,
            Material {
                texture: Texture::Solid(color),
                reflectiveness: 0.9,
            },
        );

        scene.lights.push(Box::new(light));
        scene.objects.push(Box::new(sphere));
    }

    // render out to a list of colors
    println!("Rendering scene");
    scene.render_to("render.png", image::ImageFormat::Png);

    println!(
        "Operation complete in {}s",
        start_time.elapsed().as_secs() as f32
            + start_time.elapsed().subsec_nanos() as f32 / 1000000000.
    );
}
