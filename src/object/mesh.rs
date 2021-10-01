use crate::{
    acceleration,
    material::Material,
    math::{Ray, Vector3},
    scene::EPSILON,
};

use super::{AabbIntersector, Hit, Intersect, SceneObject};

#[derive(Clone, Debug)]
pub struct Triangle {
    /// The vertices of the triangle.
    v: [Vector3; 3],

    /// The texcoords of each vertex.
    texcoords: Option<(u32, u32, u32)>,

    /// The normals of each vertex. If `None`, the normal of the entire triangle will be used.
    normals: Option<(u32, u32, u32)>,

    /// The precomputed edge 1.
    edge1: Vector3,

    /// The precomputed edge 2.
    edge2: Vector3,

    /// The precomputed normal.
    normal: Vector3,

    /// The bounding box.
    bounding_box: acceleration::Aabb,
}

#[derive(Debug, Clone)]
struct TriIntersect {
    p: Vector3,
    t: f64,
    u: f64,
    v: f64,
}

impl Triangle {
    /// Create a new triangle. It must be `recalculate`d at some point before its usage.
    pub fn new(
        (v0, v1, v2): (Vector3, Vector3, Vector3),
        texcoords: Option<(u32, u32, u32)>,
        normals: Option<(u32, u32, u32)>,
    ) -> Self {
        let v = [v0, v1, v2];
        let bounding_box = acceleration::Aabb::from_vecs(&v);
        Self {
            v,
            texcoords,
            normals,
            edge1: Vector3::default(),
            edge2: Vector3::default(),
            normal: Vector3::default(),
            bounding_box,
        }
    }

    fn recalculate(&mut self) {
        self.edge1 = self.v[1] - self.v[0];
        self.edge2 = self.v[2] - self.v[0];
        self.normal = self.edge1.cross(self.edge2).normalize();
        self.bounding_box = acceleration::Aabb::from_vecs(&self.v);
    }

    // Muller-Trombore ray-triangle intersection algorithm
    fn intersect(&self, ray: &Ray) -> Option<TriIntersect> {
        let h = ray.direction.cross(self.edge2);
        let a = self.edge1.dot(h);
        if a > -EPSILON && a < EPSILON {
            return None;
        }

        let f = 1. / a;
        let s = ray.origin - self.v[0];
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(self.edge1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * self.edge2.dot(q);
        if t > EPSILON {
            Some(TriIntersect {
                p: ray.along(t),
                t,
                u,
                v,
            })
        } else {
            None
        }
    }

    fn uvs(&self, i: &TriIntersect, tc: &[(f32, f32)]) -> Option<(f32, f32)> {
        match self.texcoords {
            None => None,
            Some((a, b, c)) => {
                let (a, b, c) = (&tc[a as usize], &tc[b as usize], &tc[c as usize]);
                let (iu, iv, iw) = (i.u as f32, i.v as f32, 1. - i.u as f32 - i.v as f32);
                let u = a.0 * iw + b.0 * iu + c.0 * iv as f32;
                let v = a.1 * iw + b.1 * iu + c.1 * iv as f32;
                Some((u, 1. - v))
            }
        }
    }

    fn normal(&self, i: &TriIntersect, ns: &[Vector3]) -> Vector3 {
        match self.normals {
            None => self.normal,
            Some((a, b, c)) => {
                let (a, b, c) = (&ns[a as usize], &ns[b as usize], &ns[c as usize]);
                let (u, v, w) = (1. - i.u - i.v, i.u, i.v);
                *a * u + *b * v + *c * w
            }
        }
    }
}

impl acceleration::Primitive for Triangle {
    fn points(&self) -> &[Vector3] {
        &self.v
    }

    fn split(&self, _split: acceleration::Split) -> (Self, Option<Self>) {
        (self.clone(), None) // TODO: this eventually
    }

    fn bounding_box(&self) -> &acceleration::Aabb {
        &self.bounding_box
    }
}

pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub sbvh: Option<acceleration::TreeNode>,
    pub bounding_box: AabbIntersector,
    pub material: Material,
    pub texcoords: Vec<(f32, f32)>,
    pub normals: Vec<Vector3>,
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            triangles: self.triangles.clone(),
            sbvh: None,
            bounding_box: self.bounding_box.clone(),
            material: self.material.clone(),
            texcoords: self.texcoords.clone(),
            normals: self.normals.clone(),
        }
    }
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Material) -> Self {
        Self {
            triangles,
            sbvh: None,
            bounding_box: Default::default(),
            material,
            texcoords: Vec::new(),
            normals: Vec::new(),
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

        let texcoords_count = model.mesh.texcoords.len() / 2;
        let mut texcoords_iter = model.mesh.texcoords.into_iter();
        let mut texcoords = vec![];

        while texcoords.len() < texcoords_count {
            texcoords.push((
                texcoords_iter.next().unwrap(),
                texcoords_iter.next().unwrap(),
            ));
        }

        let normals_count = model.mesh.normals.len() / 3;
        let mut normals_iter = model.mesh.normals.into_iter();
        let mut normals = vec![];

        while normals.len() < normals_count {
            normals.push(Vector3::new(
                normals_iter.next().unwrap() as f64,
                normals_iter.next().unwrap() as f64,
                normals_iter.next().unwrap() as f64,
            ))
        }

        let tri_count = model.mesh.indices.len() / 3;
        let mut iter = model.mesh.indices.into_iter();

        let mut texc_iter = model.mesh.texcoord_indices.into_iter();
        let mut norm_iter = model.mesh.normal_indices.into_iter();

        let mut triangles = vec![];
        while triangles.len() < tri_count {
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
                        texc_iter.next().unwrap(),
                        texc_iter.next().unwrap(),
                        texc_iter.next().unwrap(),
                    ))
                },
                if normals.is_empty() {
                    None
                } else {
                    Some((
                        norm_iter.next().unwrap(),
                        norm_iter.next().unwrap(),
                        norm_iter.next().unwrap(),
                    ))
                },
            ));
        }
        Self {
            triangles,
            sbvh: None,
            material,
            bounding_box: AabbIntersector::default(),
            texcoords,
            normals,
        }
    }

    pub fn shift(&mut self, delta: Vector3) {
        for tri in self.triangles.iter_mut() {
            tri.v[0] += delta;
            tri.v[1] += delta;
            tri.v[2] += delta;
        }
    }

    pub fn scale(&mut self, delta: f64) {
        for tri in self.triangles.iter_mut() {
            tri.v[0] *= delta;
            tri.v[1] *= delta;
            tri.v[2] *= delta;
        }
    }

    pub fn recalculate(&mut self) {
        let vecs = self
            .triangles
            .iter_mut()
            .map(|v| {
                v.recalculate();
                &v.v
            })
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

        self.bounding_box = AabbIntersector {
            pos: center,
            size: max - center,
        };

        self.sbvh = Some(acceleration::Sbvh::new(&self.triangles).into());
    }

    fn sbvh_intersection(&self, node: &acceleration::TreeNode, ray: &Ray) -> Vec<usize> {
        assert!(self.sbvh.is_some());

        let bounding = match node {
            acceleration::TreeNode::Leaf { bounding, .. } => bounding,
            acceleration::TreeNode::Branch { bounding, .. } => bounding,
        };

        if !bounding.intersect(ray) {
            return vec![];
        }

        match node {
            acceleration::TreeNode::Branch { a, b, .. } => self
                .sbvh_intersection(a, ray)
                .into_iter()
                .chain(self.sbvh_intersection(b, ray).into_iter())
                .collect(),
            acceleration::TreeNode::Leaf { indices, .. } => indices.clone(),
        }
    }
}

impl Intersect for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        assert!(self.sbvh.is_some());

        let tris = self
            .sbvh_intersection(self.sbvh.as_ref().unwrap(), ray)
            .into_iter()
            .map(|i| &self.triangles[i])
            .collect::<Vec<_>>();

        if tris.is_empty() {
            return None;
        }

        // find all triangles that intersect our ray
        let mut intersected_tris = tris
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
                intersected_tris[0]
                    .0
                    .normal(&intersected_tris[0].1, &self.normals),
                (intersected_tris[0].1.t, intersected_tris[0].1.p),
                (intersected_tris[0].1.t, intersected_tris[0].1.p),
                intersected_tris[0]
                    .0
                    .uvs(&intersected_tris[0].1, &self.texcoords)
                    .unwrap_or_default(),
            )),

            // two hits: return the first hit, but t_far is the t_near of the second hit (assuming we leave the mesh at this point)
            _ => Some(Hit::new(
                intersected_tris[0]
                    .0
                    .normal(&intersected_tris[0].1, &self.normals),
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
