use rand::Rng;

use crate::{
    material::Color,
    math::{blerp, Ray, Vector3},
    object::Hit,
    scene::{Scene, EPSILON},
};

use super::{Light, LightShading, METER};

/// A volume that an area light can take on.
#[derive(Debug, Clone)]
pub enum AreaSurface {
    /// A sphere with a center and radius.
    Sphere(Vector3, f64),

    /// A rectangle in space, given four vectors as corners.
    Rectangle([Vector3; 4]),
}

impl AreaSurface {
    /// Sample a point from this volume, given a random number generator that generates a
    /// random number from -1 to 1.
    pub fn sample<F>(&self, random: F) -> Vector3
    where
        F: Fn() -> f64,
    {
        match self {
            Self::Sphere(position, radius) => {
                // I know this isn't how you're supposed to implement this
                // but I'm doing it anyway
                let (mut x, mut y, mut z) = (random(), random(), random());
                let mag = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
                let d = (random() * 0.5 + 0.5) / mag;
                x *= d;
                y *= d;
                z *= d;
                Vector3::new(x, y, z) * *radius + *position
            }
            Self::Rectangle(corners) => blerp(
                random() * 0.5 + 0.5,
                random() * 0.5 + 0.5,
                corners[0],
                corners[1],
                corners[2],
                corners[3],
            ),
        }
    }
}

/// An area light, which is a light that emits in all directions from a specified position,
/// over a surface area in space, creating softer shadows.
#[derive(Clone, Debug)]
pub struct Area {
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

    /// The surface representing this area light.
    pub surface: AreaSurface,

    /// The number of iterations to calculate lighting on this light.
    pub iterations: u32,

    /// The maximum distance at which this light can influence a hit point. It
    /// will not be considered if the distance from the hit point to the light is
    /// greater than this value.
    pub max_distance: f64,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            color: Color::new(255, 255, 255),
            intensity: 6.,
            specular_power: 32,
            specular_strength: 0.7,
            surface: AreaSurface::Sphere(Vector3::new(0., 0., 0.), 0.),
            iterations: 4,
            max_distance: 50.,
        }
    }
}

impl Light for Area {
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
        let mut samples = vec![];

        for _ in 0..self.iterations {
            // vector pointing from hit to light pos
            let pos = self
                .surface
                .sample(|| rand::thread_rng().gen_range(-1. ..=1.));
            let lvec = pos - hit.vnear;

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

            samples.push(LightShading::new(diffuse, specular, lint));
        }

        LightShading::new(
            samples.iter().map(|s| s.diffuse).sum::<f64>() / samples.len() as f64,
            samples.iter().map(|s| s.specular).sum::<f64>() / samples.len() as f64,
            samples.iter().map(|s| s.intensity).sum::<f64>() / samples.len() as f64,
        )
    }
}
