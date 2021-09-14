use crate::{
    camera::Camera,
    material::Color,
    math::Ray,
    object::{Hit, SceneObject},
};

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Box<dyn SceneObject>>,
    pub camera: Camera,
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

        let mut color = Color::from_normal(ray.direction);
        let objects = self.cast_ray(&ray);

        // TODO: this needs to be a lot more complex, but for now, we'll just take the first object if any
        if !objects.is_empty() {
            let (obj, hit) = &objects[0];
            color = obj.material().color;
        }

        color
    }

    /// Render the image out as a list of Colors.
    pub fn render(&self) -> Vec<Color> {
        let mut v = vec![];

        // TEMPORARY: naive approach, does not use multiple cores
        for y in 0..self.camera.vh {
            for x in 0..self.camera.vw {
                v.push(self.trace_pixel(x, y));
            }
        }

        v
    }
}
