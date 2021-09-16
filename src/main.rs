#![allow(dead_code)]
#![allow(clippy::many_single_char_names)]

mod camera;
mod lighting;
mod material;
mod math;
mod object;
mod scene;

use std::{f64::consts::PI, ops::Range, time::Instant};

use camera::Camera;
use material::{Color, Material, Texture};
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
            vw: 2560,
            vh: 1440,
            origin: Vector3::new(4., 1.6, 4.),
            pitch: -0.3,
            yaw: -PI / 4.,
            ..Default::default()
        },
        ..Default::default()
    };

    // add a sun light
    // scene.lights.push(Box::new(lighting::Sun::default()));

    // add a plane
    scene.objects.push(Box::new(object::Plane::new(
        Vector3::new(0., -1., 0.),
        Vector3::up(),
        Material {
            texture: Texture::Solid(Color::new(180, 180, 180)),
            reflectiveness: 0.,
        }
    )));

    // add the obj in the middle
    let texture_name = "assets/Handle1Tex.png";
    let obj_name = "assets/fedora.obj";

    let tex = image::open(texture_name).unwrap().to_rgb8();
    let mut obj = object::Mesh::from_obj(obj_name.into(), Material {
        texture: Texture::Image(tex),
        reflectiveness: 0.,
    });
    obj.scale(2.0);
    obj.shift(Vector3::new(0.6, -3., 0.));
    obj.recalculate();
    scene.objects.push(Box::new(obj));

    // add some reflective spheres around the center
    for n in 0..8 {
        let inner = n as f64 / 8. * PI * 2.;
        let cos = inner.cos();
        let sin = inner.sin();

        let light = lighting::Point {
            color: Color::hsv(n as f32 / 8. * 360., 255, 255),
            intensity: 4.,
            position: Vector3::new(cos * 5., 2., sin * 5.),
            ..Default::default()
        };

        let sphere = object::Sphere::new(
            Vector3::new(cos * 8., 1., sin * 8.),
            2.,
            Material {
                texture: Texture::Solid(Color::new(255, 255, 255)),
                reflectiveness: 1.,
            }
        );

        scene.lights.push(Box::new(light));
        scene.objects.push(Box::new(sphere));
    }

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
