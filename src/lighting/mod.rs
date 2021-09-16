mod point;
mod sun;

use crate::{material::Color, math::Ray, object::Hit, scene::Scene};

pub use point::*;
pub use sun::*;

/// Approximately how many raytracer units equates to a meter.
/// Used for lighting.
pub static METER: f64 = 1.;

/// The result of a light's influence on a ray intersection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightShading {
    pub diffuse: f64,
    pub specular: f64,
    pub intensity: f64,
}

impl LightShading {
    pub fn new(diffuse: f64, specular: f64, intensity: f64) -> Self {
        Self {
            diffuse,
            specular,
            intensity,
        }
    }
}

/// This trait represents any object that is a light.
/// It provides details about the light, as well as the result
/// of shading due to the light at a certain point.
pub trait Light: Send + Sync {
    fn color(&self) -> &Color;
    fn intensity(&self) -> f64;
    fn specular_power(&self) -> i32;
    fn specular_strength(&self) -> f64;

    fn shading(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> LightShading;
}
