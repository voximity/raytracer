use std::cmp;

use crate::{
    material::{Color, Material},
    math::{Ray, Vector3},
    object::Hit,
    remap,
    scene::{Scene, EPSILON},
};

use super::{Light, LightShading};

pub struct Sun {
    pub color: Color,
    pub intensity: f64,
    pub specular_power: i32,
    pub specular_strength: f64,
    pub vector: Vector3,
    pub shadow_coefficient: f64,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            color: Color::new(255, 255, 255),
            intensity: 1.,
            specular_power: 32,
            specular_strength: 0.5,
            vector: Vector3::new(0., -1., 0.),
            shadow_coefficient: 0.2,
        }
    }
}

impl Light for Sun {
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
        let lvec = -self.vector;

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
        let hit_pos = ray.along(hit.near);
        let shadow_ray = Ray::new(hit_pos + hit.normal * EPSILON, lvec);
        if let Some(_shadow_hit) = scene.cast_ray_once(&shadow_ray) {
            // TODO: deal with transparency
            diffuse *= self.shadow_coefficient;
            specular *= self.shadow_coefficient;
        }

        LightShading::new(diffuse, specular, self.intensity)
    }
}
