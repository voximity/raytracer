use crate::{
    material::Material,
    math::{Ray, Vector3},
    scene::EPSILON,
};

use super::{Aabb, Hit, Intersect, SceneObject};

#[derive(Clone, Debug)]
pub struct Triangle {
    v0: Vector3,
    v1: Vector3,
    v2: Vector3,
    e1: Vector3,
    e2: Vector3,
    normal: Vector3,
}

impl Triangle {
    pub fn new(v0: Vector3, v1: Vector3, v2: Vector3) -> Self {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let n = e1.cross(e2).normalize();

        Self {
            v0,
            v1,
            v2,
            e1,
            e2,
            normal: n,
        }
    }

    fn recalculate(&mut self) {
        self.e1 = self.v1 - self.v0;
        self.e2 = self.v2 - self.v0;
        self.normal = self.e1.cross(self.e2).normalize();
    }

    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let h = ray.direction.cross(self.e2);
        let a = self.e1.dot(h);
        if a > -EPSILON && a < EPSILON {
            return None;
        }

        let f = 1. / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);
        if u < 0. || u > 1. {
            return None;
        }

        let q = s.cross(self.e1);
        let v = f * ray.direction.dot(q);
        if v < 0. || u + v > 1. {
            return None;
        }

        let t = f * self.e2.dot(q);
        if t > EPSILON {
            Some(t)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub bounding_box: Aabb,
    pub material: Material,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Material) -> Self {
        Self {
            triangles,
            bounding_box: Default::default(),
            material,
        }
    }

    pub fn from_obj(file_name: String, material: Material) -> Self {
        let (models, _) = tobj::load_obj(
            file_name,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .expect("failed to parse obj");

        let model = models.into_iter().next().unwrap();
        let mut iter = model.mesh.indices.into_iter().peekable();

        let mut triangles = vec![];
        while iter.peek().is_some() {
            let v0i = iter.next().unwrap();
            let v1i = iter.next().unwrap();
            let v2i = iter.next().unwrap();

            triangles.push(Triangle::new(
                Vector3::new(
                    model.mesh.positions[v0i as usize * 3] as f64,
                    model.mesh.positions[v0i as usize * 3 + 1] as f64,
                    model.mesh.positions[v0i as usize * 3 + 2] as f64,
                ),
                Vector3::new(
                    model.mesh.positions[v1i as usize * 3] as f64,
                    model.mesh.positions[v1i as usize * 3 + 1] as f64,
                    model.mesh.positions[v1i as usize * 3 + 2] as f64,
                ),
                Vector3::new(
                    model.mesh.positions[v2i as usize * 3] as f64,
                    model.mesh.positions[v2i as usize * 3 + 1] as f64,
                    model.mesh.positions[v2i as usize * 3 + 2] as f64,
                ),
            ));
        }

        Self::new(triangles, material)
    }

    pub fn shift(&mut self, delta: Vector3) {
        for tri in self.triangles.iter_mut() {
            tri.v0 += delta;
            tri.v1 += delta;
            tri.v2 += delta;
        }
    }

    pub fn scale(&mut self, delta: f64) {
        for tri in self.triangles.iter_mut() {
            tri.v0 *= delta;
            tri.v1 *= delta;
            tri.v2 *= delta;
            tri.recalculate();
        }
    }

    pub fn recalculate(&mut self) {
        let vecs = self
            .triangles
            .iter()
            .map(|v| [&v.v0, &v.v1, &v.v2])
            .flatten()
            .collect::<Vec<_>>();

        let min_x = vecs.iter().map(|v| v.x).fold(0. / 0., f64::min);
        let max_x = vecs.iter().map(|v| v.x).fold(0. / 0., f64::max);
        let min_y = vecs.iter().map(|v| v.y).fold(0. / 0., f64::min);
        let max_y = vecs.iter().map(|v| v.y).fold(0. / 0., f64::max);
        let min_z = vecs.iter().map(|v| v.z).fold(0. / 0., f64::min);
        let max_z = vecs.iter().map(|v| v.z).fold(0. / 0., f64::max);

        let center = Vector3::new(
            (min_x + max_x) * 0.5,
            (min_y + max_y) * 0.5,
            (min_z + max_z) * 0.5,
        );
        let max = Vector3::new(max_x, max_y, max_z);

        self.bounding_box = Aabb::new(center, max - center, Material::default());
    }
}

impl Intersect for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // first, test if we strike the bounding box
        // if we don't, we can simply return
        //
        // this is a simple optimization method
        // to prevent naively testing against every
        // triangle in the mesh when we're nowhere near it
        if let None = self.bounding_box.intersect(ray) {
            return None;
        }

        // find all triangles that intersect our ray
        let mut intersected_tris = self
            .triangles
            .iter()
            .filter_map(|t| t.intersect(ray).map(|h| (t, h)))
            .collect::<Vec<_>>();

        // and sort them by nearness
        intersected_tris
            .sort_by(|(_, ta), (_, tb)| ta.partial_cmp(tb).unwrap_or(std::cmp::Ordering::Equal));

        // return based on how many triangles we have
        match intersected_tris.len() {
            // no hits: return no hit
            0 => None,

            // one hit: return the only hit, where t_far is also t_near
            1 => Some(Hit::new(
                intersected_tris[0].0.normal,
                intersected_tris[0].1,
                intersected_tris[0].1,
            )),

            // two hits: return the first hit, but t_far is the t_near of the second hit (assuming we leave the mesh at this point)
            _ => Some(Hit::new(
                intersected_tris[0].0.normal,
                intersected_tris[0].1,
                intersected_tris[1].1,
            )),
        }
    }
}

impl SceneObject for Mesh {
    fn material(&self) -> &Material {
        &self.material
    }
}
