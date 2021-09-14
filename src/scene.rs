

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    camera::Camera,
    lighting::Light,
    material::Color,
    math::{Ray, Vector3},
    object::{Hit, SceneObject},
};

/// A very small value, close to zero, to prevent weird overlapping.
pub const EPSILON: f64 = 0.000001;

pub struct Scene {
    pub objects: Vec<Box<dyn SceneObject>>,
    pub lights: Vec<Box<dyn Light>>,
    pub camera: Camera,
    pub ambient: Color,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            camera: Camera::default(),
            ambient: Color::new(40, 40, 40),
        }
    }
}

impl Scene {
    /// Develop a list of objects that are struck by a ray.
    pub fn cast_ray(&self, ray: &Ray) -> Vec<(&Box<dyn SceneObject>, Hit)> {
        let mut v = vec![];

        // iterate over every object in the scene and test for an intersection
        for object in self.objects.iter() {
            match object.intersect(ray) {
                Some(hit) => v.push((object, hit)),
                None => continue,
            }
        }

        // sort the struck objects by their nearness to the ray
        v.sort_by(|(_, ah), (_, bh)| {
            ah.near
                .partial_cmp(&bh.near)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        v
    }

    /// Cast a ray and return one optional object.
    pub fn cast_ray_once(&self, ray: &Ray) -> Option<(&Box<dyn SceneObject>, Hit)> {
        let hit = self.cast_ray(ray);
        hit.into_iter().next()
    }

    /// Trace out a pixel, where top-left of the image is (0, 0).
    /// This function is run many times in parallel.
    pub fn trace_pixel(&self, x: i32, y: i32) -> Color {
        // Things to study:
        // How can we optimize the object lookup process? There
        // are many methods documented online of how to accelerate
        // this. We can use a BVH, a chunked off octree (I have
        // already written an implementation of this for another
        // project so it wouldn't be too challenging to port it over
        // and repurpose it), or a number of other acceleration
        // structures. For now, we will just index through every
        // unique scene object for every ray. This is slow, but for
        // scenes of only a few objects, it's not really a problem.

        let ray = Ray::new(
            self.camera.origin,
            self.camera.direction_at(x as f64, y as f64),
        );

        // as a test, we take the normal color of the ray's direction for the skybox (just for now)
        let (object, hit) = match self.cast_ray_once(&ray) {
            Some(r) => r,
            None => return Color::from_normal(ray.direction),
        };

        let mut color = object.material().color;

        // Calculate light influences
        let mut sum_vecs: Vector3 = self.ambient.into();
        for light in self.lights.iter() {
            let lcol: Vector3 = (*light.color()).into();
            let shading = light.shading(&ray, &hit, self);

            // color from diffuse/specular
            let diffuse = lcol * shading.diffuse;
            let specular = lcol * (shading.specular * light.specular_strength());

            sum_vecs += (diffuse + specular) * shading.intensity;
        }

        color = (Into::<Vector3>::into(color) * sum_vecs).into();

        // todo: refraction

        // todo: reflection

        // todo: fog

        color
    }

    /// Render the image out as a list of Colors.
    pub fn render(&self) -> Vec<Color> {
        let (vw, vh) = (self.camera.vw, self.camera.vh);
        (0..(vw * vh))
            .into_par_iter()
            .map(|i| self.trace_pixel(i % vw, i / vw))
            .collect::<Vec<_>>()
    }
}
