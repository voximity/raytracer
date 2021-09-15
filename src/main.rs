#![allow(dead_code)]

mod camera;
mod lighting;
mod material;
mod math;
mod object;
mod scene;

use std::{f64::consts::PI, ops::Range, time::Instant};

use camera::Camera;
use material::{Color, Material};
use math::Vector3;
use scene::Scene;

pub fn remap(t: f64, a: Range<f64>, b: Range<f64>) -> f64 {
    (t - a.start) * ((b.end - b.start) / (a.end - a.start)) + b.start
}

fn main() {
    println!("Initializing scene");
    let start_time = Instant::now();

    let mut scene = Scene {
        camera: Camera {
            vw: 800,
            vh: 600,
            origin: Vector3::new(0., 2.5, 0.),
            pitch: -PI / 8.,
            ..Default::default()
        },
        ..Default::default()
    };

    // add a good ol' sun
    scene.lights.push(Box::new(lighting::Sun {
        vector: Vector3::new(-0.4, -1., 0.2).normalize(),
        ..Default::default()
    }));

    // add a plane
    scene.objects.push(Box::new(object::Plane::new(
        Vector3::new(0., -2., 0.),
        Vector3::new(0., 1., 0.),
        Material {
            color: Color::new(10, 80, 20),
            reflectiveness: 0.4,
        },
    )));

    // add some spheres
    scene.objects.push(Box::new(object::Aabb::new(
        Vector3::new(-5., 0., -15.),
        Vector3::new(2., 2., 2.),
        Material {
            color: Color::new(180, 0, 0),
            reflectiveness: 0.4,
        },
    )));

    scene.objects.push(Box::new(object::Aabb::new(
        Vector3::new(0., 0., -15.),
        Vector3::new(2., 2., 2.),
        Material {
            color: Color::new(0, 180, 0),
            reflectiveness: 0.4,
        },
    )));

    scene.objects.push(Box::new(object::Aabb::new(
        Vector3::new(5., 0., -15.),
        Vector3::new(2., 2., 2.),
        Material {
            color: Color::new(0, 0, 180),
            reflectiveness: 0.4,
        },
    )));

    // render out to a list of colors
    println!("Rendering scene");
    let rendered = scene.render();

    // spit out an image
    println!("Saving to image");
    let mut imgbuf: image::RgbImage =
        image::ImageBuffer::new(scene.camera.vw as u32, scene.camera.vh as u32);

    for (i, color) in rendered.into_iter().enumerate() {
        imgbuf.put_pixel(
            i as u32 % scene.camera.vw as u32,
            i as u32 / scene.camera.vw as u32,
            image::Rgb([color.r, color.g, color.b]),
        );
    }

    imgbuf
        .save_with_format("render.png", image::ImageFormat::Png)
        .unwrap();

    println!(
        "Operation complete in {}s",
        start_time.elapsed().as_secs() as f32 + start_time.elapsed().subsec_millis() as f32 / 1000.
    );
}
