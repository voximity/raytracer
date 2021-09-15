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

/// A scene, which contains a list of objects, lights, and a camera to render from.
pub struct Scene {
    pub objects: Vec<Box<dyn SceneObject>>,
    pub lights: Vec<Box<dyn Light>>,
    pub camera: Camera,
    pub ambient: Color,
    pub max_reflection_depth: u32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            camera: Camera::default(),
            ambient: Color::new(40, 40, 40),
            max_reflection_depth: 4,
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

    /// Trace out a ray, getting its color.
    pub fn trace_ray(&self, ray: Ray, depth: u32) -> Color {
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

        let reflectiveness = object.material().reflectiveness;
        if reflectiveness > EPSILON && depth < self.max_reflection_depth {
            // reflect off this object, and mix in the final color
            // we do this just slightly off the surface of the
            // hit object so as not to cause any weird overlap

            // TODO: should I incorporate some rendering techniques like fresnel to fade or amplify the edges?
            let reflected = self.trace_ray(
                ray.reflect(ray.along(hit.near) + hit.normal * EPSILON, hit.normal),
                depth + 1,
            );

            color = color.lerp(reflected, reflectiveness);
        }

        // todo: fog

        color
    }

    /// Trace out a pixel, where top-left of the image is (0, 0).
    /// This function is run many times in parallel.
    pub fn trace_pixel(&self, x: i32, y: i32) -> Color {
        let ray = Ray::new(
            self.camera.origin,
            self.camera.direction_at(x as f64, y as f64),
        );

        self.trace_ray(ray, 0)
    }

    /// Render the image out as a list of Colors.
    pub fn render(&self) -> Vec<Color> {
        let (vw, vh) = (self.camera.vw, self.camera.vh);

        // Thanks to Rayon, parallelizing the raytracer is
        // outrageously simple. Rayon provides "parallel iterators",
        // which largely reflect the Rust trait `Iterator`, except
        // they are handled by Rayon's global thread scheduler,
        // which means they intelligently are scheduled to be
        // run by different CPU cores, all on a balanced load.
        // Initially, I'd written some code to chunk off pixels
        // in the image by some arbitrary configurable number and
        // use a thread scheduler to eat away these entire chunks,
        // aggregate the results into a map and write it back to
        // the main core, which worked, but this solution is MUCH
        // cleaner because you have the beauty of a well-maintained
        // and researched Rust library developed by very smart people
        // who have optimized for this specific case.
        (0..(vw * vh))
            .into_par_iter() // Look at that! Just create a range and parallelize it instantly. Beautiful!
            .map(|i| self.trace_pixel(i % vw, i / vw))
            .collect::<Vec<_>>()
    }
}
