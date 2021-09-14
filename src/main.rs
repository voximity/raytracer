mod camera;
mod lighting;
mod material;
mod math;
mod object;
mod scene;

use std::ops::Range;

use camera::Camera;
use material::{Color, Material};
use math::Vector3;
use scene::Scene;

pub fn remap(t: f64, a: Range<f64>, b: Range<f64>) -> f64 {
    (t - a.start) * ((b.end - b.start) / (a.end - a.start)) + b.start
}

fn main() {
    let mut scene = Scene {
        camera: Camera {
            vw: 1920,
            vh: 1080,
            ..Default::default()
        },
        ..Default::default()
    };

    // add a good ol' sun
    scene.lights.push(Box::new(lighting::Sun {
        vector: Vector3::new(-0.4, -1., 0.2).normalize(),
        ..Default::default()
    }));

    // add a red sphere as a test
    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(-5., 0., -10.),
        2.,
        Material {
            color: Color::new(180, 0, 0),
        },
    )));

    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(0., 0., -10.),
        2.,
        Material {
            color: Color::new(0, 180, 0),
        },
    )));

    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(5., 0., -10.),
        2.,
        Material {
            color: Color::new(0, 0, 180),
        },
    )));

    // render out to a list of colors
    let rendered = scene.render();

    // spit out an image
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
}
