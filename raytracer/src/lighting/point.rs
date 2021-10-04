use crate::{
    material::Color,
    math::{Ray, Vector3},
    object::Hit,
    scene::{Scene, EPSILON},
};

use super::{Light, LightShading, METER};

/// A point light, which is a light that emits in all directions from a specified position.
#[derive(Clone, Debug)]
pub struct Point {
    /// The color of this light.
    pub color: Color,

    /// The intensity of this light. Not totally sure what real-world unit
    /// to relate this value to...
    pub intensity: f64,

    /// The power at which specular lighting will be raised to. Generally speaking,
    /// 16, 32, and 64 are good values.
    pub specular_power: i32,

    /// The strength at which specular lighting will be applied.
    pub specular_strength: f64,

    /// The position in space of this light.
    pub position: Vector3,

    /// The maximum distance at which this light can influence a hit point. It
    /// will not be considered if the distance from the hit point to the light is
    /// greater than this value.
    pub max_distance: f64,
}

impl Default for Point {
    fn default() -> Self {
        Self {
            color: Color::new(255, 255, 255),
            intensity: 6.,
            specular_power: 32,
            specular_strength: 0.7,
            position: Vector3::new(0., 0., 0.),
            max_distance: 50.,
        }
    }
}

impl Light for Point {
    fn color(&self) -> &Color {
        &self.color
    }

    fn intensity(&self) -> f64 {
        self.intensity
    }

    fn specular_power(&self) -> i32 {
        self.specular_power
    }

    fn specular_strength(&self) -> f64 {
        self.specular_strength
    }

    fn shading(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> LightShading {
        // vector pointing from hit to light pos
        let lvec = self.position - hit.vnear;

        // calculate distance and normalize, all at once
        let dist = lvec.magnitude();
        if dist > self.max_distance {
            return LightShading::default();
        }

        let lvec = lvec / dist;

        // calculate diffuse
        let mut diffuse = hit.normal.dot(lvec).clamp(0., f64::MAX);

        // calculate specular
        let halfway_dir = (lvec - ray.direction).normalize();
        let mut specular = hit
            .normal
            .dot(halfway_dir)
            .clamp(0., f64::MAX)
            .powi(self.specular_power);

        // apply shadowing
        let shadow_ray = Ray::new(hit.vnear + hit.normal * EPSILON, lvec);
        if let Some(shadow_hit) = scene.cast_ray_once(&shadow_ray) {
            if shadow_hit.1.near <= dist {
                // TODO: deal with transparency

                // do we want a shadow_coefficient for point lights? probably not
                diffuse *= 0.;
                specular *= 0.;
            }
        }

        // calculate intensity
        let lint = self.intensity / (dist / METER).powi(2);

        LightShading::new(diffuse, specular, lint)
    }
}
