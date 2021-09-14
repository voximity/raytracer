mod camera;
mod material;
mod math;
mod object;
mod scene;

use material::{Color, Material};
use math::Vector3;
use scene::Scene;

fn main() {
    let mut scene = Scene::default();
    scene.camera.yaw = 0.2;

    // add a red sphere as a test
    scene.objects.push(Box::new(object::Sphere::new(
        Vector3::new(0., 0., -5.),
        2.,
        Material {
            color: Color::new(180, 0, 0),
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
