use crate::{
    material::Material,
    math::{Ray, Vector3},
    scene::EPSILON,
};

use super::{Aabb, Hit, Intersect, SceneObject};

#[derive(Clone, Debug)]
pub struct Triangle {
    /// The first vertex of the triangle.
    v0: Vector3,

    /// The second vertex of the triangle.
    v1: Vector3,

    /// The third vertex of the triangle.
    v2: Vector3,

    /// The texcoords of each vertex.
    texcoords: Option<(u32, u32, u32)>,

    /// The precomputed edge 0.
    e0: Vector3,

    /// The precomputed edge 1.
    e1: Vector3,

    /// The precomputed edge 2.
    e2: Vector3,

    /// The precomputed normal.
    normal: Vector3,
}

#[derive(Debug, Clone)]
struct TriIntersect {
    p: Vector3,
    t: f64,
    u: f64,
    v: f64,
}

impl TriIntersect {
    fn u(&self) -> f64 {
        self.u
    }

    fn v(&self) -> f64 {
        self.v
    }

    fn w(&self) -> f64 {
        1. - self.u - self.v
    }
}

impl Triangle {
    pub fn new(
        (v0, v1, v2): (Vector3, Vector3, Vector3),
        texcoords: Option<(u32, u32, u32)>,
    ) -> Self {
        let e0 = v1 - v0;
        let e1 = v2 - v1;
        let e2 = v0 - v2;
        let n = e0.cross(-e2).normalize();

        Self {
            v0,
            v1,
            v2,
            texcoords,
            e0,
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

    fn intersect(&self, ray: &Ray) -> Option<TriIntersect> {
        let denom = self.normal.dot(self.normal);

        // find P
        let ndrd = self.normal.dot(ray.direction);
        if ndrd.abs() < EPSILON {
            return None;
        }

        // compute d parameter
        let d = self.normal.dot(self.v0);

        // compute t
        let t = (self.normal.dot(ray.origin) + d) / ndrd;
        if t < 0. {
            return None;
        }

        // compute the intersection
        let p = ray.origin + ray.direction * t;
        let mut c;

        // edge 0
        let vp0 = p - self.v0;
        c = self.e0.cross(vp0);
        if self.normal.dot(c) < 0. {
            return None;
        }

        // edge 1
        let vp1 = p - self.v1;
        c = self.e1.cross(vp1);
        let mut u = self.normal.dot(c);
        if u < 0. {
            return None;
        }

        // edge 2
        let vp2 = p - self.v2;
        c = self.e2.cross(vp2);
        let mut v = self.normal.dot(c);
        if v < 0. {
            return None;
        }

        u /= denom;
        v /= denom;

        Some(TriIntersect { p, t, u, v })
    }

    fn uvs(&self, i: &TriIntersect, tc: &[(f32, f32)]) -> Option<(f32, f32)> {
        match self.texcoords {
            None => None,
            Some((a, b, c)) => {
                let (a, b, c) = (&tc[a as usize], &tc[b as usize], &tc[c as usize]);
                let iw = i.w();
                let u = a.0 * i.u as f32 + b.0 * i.v as f32 + c.0 * iw as f32;
                let v = a.1 * i.u as f32 + b.1 * i.v as f32 + c.1 * iw as f32;
                Some((u, v))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub bounding_box: Aabb,
    pub material: Material,
    pub texcoords: Vec<(f32, f32)>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Material) -> Self {
        Self {
            triangles,
            bounding_box: Default::default(),
            material,
            texcoords: Vec::new(),
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
        let mut texcoords_iter = model.mesh.texcoords.into_iter().peekable();
        let mut texcoords = vec![];

        while texcoords_iter.peek().is_some() {
            let (x, y) = (
                texcoords_iter.next().unwrap(),
                texcoords_iter.next().unwrap(),
            );
            texcoords.push((x, y));
        }

        let mut iter = model.mesh.indices.into_iter().peekable();

        let mut triangles = vec![];
        while iter.peek().is_some() {
            let v0i = iter.next().unwrap();
            let v1i = iter.next().unwrap();
            let v2i = iter.next().unwrap();

            triangles.push(Triangle::new(
                (
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
                ),
                if texcoords.is_empty() {
                    None
                } else {
                    Some((
                        model.mesh.texcoord_indices[v0i as usize],
                        model.mesh.texcoord_indices[v1i as usize],
                        model.mesh.texcoord_indices[v2i as usize],
                    ))
                },
            ));
        }

        Self {
            triangles,
            material,
            bounding_box: Aabb::default(),
            texcoords,
        }
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

        let min_x = vecs.iter().map(|v| v.x).fold(f64::NAN, f64::min);
        let max_x = vecs.iter().map(|v| v.x).fold(f64::NAN, f64::max);
        let min_y = vecs.iter().map(|v| v.y).fold(f64::NAN, f64::min);
        let max_y = vecs.iter().map(|v| v.y).fold(f64::NAN, f64::max);
        let min_z = vecs.iter().map(|v| v.z).fold(f64::NAN, f64::min);
        let max_z = vecs.iter().map(|v| v.z).fold(f64::NAN, f64::max);

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
        self.bounding_box.intersect(ray)?;

        // find all triangles that intersect our ray
        let mut intersected_tris = self
            .triangles
            .iter()
            .filter_map(|t| t.intersect(ray).map(|h| (t, h)))
            .collect::<Vec<_>>();

        // and sort them by nearness
        intersected_tris.sort_by(|(_, ta), (_, tb)| {
            ta.t.partial_cmp(&tb.t).unwrap_or(std::cmp::Ordering::Equal)
        });

        // return based on how many triangles we have
        match intersected_tris.len() {
            // no hits: return no hit
            0 => None,

            // one hit: return the only hit, where t_far is also t_near
            1 => Some(Hit::new(
                intersected_tris[0].0.normal,
                (intersected_tris[0].1.t, intersected_tris[0].1.p),
                (intersected_tris[0].1.t, intersected_tris[0].1.p),
                intersected_tris[0]
                    .0
                    .uvs(&intersected_tris[0].1, &self.texcoords)
                    .unwrap_or_default(),
            )),

            // two hits: return the first hit, but t_far is the t_near of the second hit (assuming we leave the mesh at this point)
            _ => Some(Hit::new(
                intersected_tris[0].0.normal,
                (intersected_tris[0].1.t, intersected_tris[0].1.p),
                (intersected_tris[1].1.t, intersected_tris[1].1.p),
                intersected_tris[0]
                    .0
                    .uvs(&intersected_tris[0].1, &self.texcoords)
                    .unwrap_or_default(),
            )),
        }
    }
}

impl SceneObject for Mesh {
    fn material(&self) -> &Material {
        &self.material
    }
}
