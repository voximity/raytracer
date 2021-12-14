use std::collections::HashMap;

use crate::{
    acceleration,
    material::Material,
    math::{Matrix, Ray, Vector3, VECTOR_MAX, VECTOR_MIN},
    scene::EPSILON,
};

use super::{Hit, Intersect, SceneObject};

struct TriIntersect {
    p: Vector3,
    t: f64,
    u: f32,
    v: f32,
}

fn triangle_normal(v0: Vector3, v1: Vector3, v2: Vector3) -> Vector3 {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    edge1.cross(edge2).normalize()
}

fn triangle_intersect(v0: Vector3, v1: Vector3, v2: Vector3, ray: &Ray) -> Option<TriIntersect> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;

    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);
    if a > -EPSILON && a < EPSILON {
        return None;
    }

    let f = 1. / a;
    let s = ray.origin - v0;
    let u = f * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);
    if t > EPSILON {
        Some(TriIntersect {
            p: ray.along(t),
            t,
            u: u as f32,
            v: v as f32,
        })
    } else {
        None
    }
}

fn triangle_intersect_uvs(
    v0: usize,
    v1: usize,
    v2: usize,
    tc: &[(f32, f32)],
    i: &TriIntersect,
) -> (f32, f32) {
    if tc.len() == 0 {
        return (0., 0.);
    }

    let (a, b, c) = (&tc[v0], &tc[v1], &tc[v2]);
    let (iu, iv, iw) = (i.u, i.v, 1. - i.u - i.v);
    let u = a.0 * iw + b.0 * iu + c.0 * iv;
    let v = a.1 * iw + b.1 * iu + c.1 * iv;
    (u.rem_euclid(1.), (1. - v).rem_euclid(1.))
}

fn triangle_intersect_normal(
    v0: usize,
    v1: usize,
    v2: usize,
    ns: &[Vector3],
    i: &TriIntersect,
) -> Vector3 {
    let (a, b, c) = (&ns[v0], &ns[v1], &ns[v2]);
    let (u, v, w) = (1. - i.u - i.v, i.u, i.v);
    *a * u as f64 + *b * v as f64 + *c * w as f64
}

pub struct Mesh {
    /// A list of unique vertices to use in the mesh.
    pub verts: Vec<Vector3>,

    /// A list of triangles; each triangle stores an index into the `verts` `Vec`.
    pub tris: Vec<[usize; 3]>,

    /// A list of unique normals to use in the mesh.
    pub normals: Vec<Vector3>,

    /// A list of triangle normals; each triangle stores an index into the `normals` `Vec`.
    ///
    /// **Indices are shared with `tris`.**
    pub tri_normals: Vec<[usize; 3]>,

    /// A list of each vertex's texture coordinates.
    pub texcoords: Vec<(f32, f32)>,

    /// A list of each triangle vertex's texture coordinates, pointing to an index in `texcoords`.
    ///
    /// **Indices are shared with `tris`.**
    pub tri_texcoords: Vec<[usize; 3]>,

    /// The material of this object.
    pub material: Material,

    /// The SBVH acceleration structure of this mesh.
    pub sbvh: Option<acceleration::TreeNode>,
}

impl Mesh {
    pub fn new(material: Material) -> Self {
        Self {
            verts: Vec::new(),
            tris: Vec::new(),
            normals: Vec::new(),
            tri_normals: Vec::new(),
            texcoords: Vec::new(),
            tri_texcoords: Vec::new(),
            material,
            sbvh: None,
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

        // Gather all vertices
        let mut verts = Vec::new();
        let mut verts_iter = model.mesh.positions.into_iter().peekable();
        while verts_iter.peek().is_some() {
            let v = verts_iter.by_ref().take(3).collect::<Vec<_>>();
            verts.push(Vector3::new(v[0] as f64, v[1] as f64, v[2] as f64));
        }

        // Gather all texcoords
        let mut texcoords = Vec::new();
        let mut texcoords_iter = model.mesh.texcoords.into_iter().peekable();
        while texcoords_iter.peek().is_some() {
            let tc = texcoords_iter.by_ref().take(2).collect::<Vec<_>>();
            texcoords.push((tc[0], tc[1]))
        }

        // Gather all normals
        let mut normals = Vec::new();
        let mut normals_iter = model.mesh.normals.into_iter().peekable();
        while normals_iter.peek().is_some() {
            let ns = normals_iter.by_ref().take(3).collect::<Vec<_>>();
            normals.push(Vector3::new(ns[0] as f64, ns[1] as f64, ns[2] as f64));
        }

        // Gather all vertex indices (into triangles)
        let mut tris = Vec::new();
        let mut tris_iter = model.mesh.indices.into_iter().peekable();
        while tris_iter.peek().is_some() {
            let v = tris_iter.by_ref().take(3).collect::<Vec<_>>();
            tris.push([v[0] as usize, v[1] as usize, v[2] as usize]);
        }

        // Gather all texcoord indices
        let mut texcoord_indices = Vec::new();
        let mut texcoord_indices_iter = model.mesh.texcoord_indices.into_iter().peekable();
        while texcoord_indices_iter.peek().is_some() {
            let v = texcoord_indices_iter.by_ref().take(3).collect::<Vec<_>>();
            texcoord_indices.push([v[0] as usize, v[1] as usize, v[2] as usize]);
        }

        // Gather all normal indices
        let mut normal_indices = Vec::new();
        let mut normal_indices_iter = model.mesh.normal_indices.into_iter().peekable();
        while normal_indices_iter.peek().is_some() {
            let v = normal_indices_iter.by_ref().take(3).collect::<Vec<_>>();
            normal_indices.push([v[0] as usize, v[1] as usize, v[2] as usize]);
        }

        Self {
            verts,
            tris,
            normals,
            tri_normals: normal_indices,
            texcoords,
            tri_texcoords: texcoord_indices,
            material,
            sbvh: None,
        }
    }

    /// Recalculate the mesh's normals.
    pub fn recalculate_normals(&mut self) {
        // Empty the normals and vert_normals vecs
        self.normals = Vec::new();
        self.tri_normals = Vec::new();

        // Populate the vertex map
        let mut vert_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, tri) in self.tris.iter().enumerate() {
            for &c in tri {
                vert_map
                    .entry(c)
                    .and_modify(|e| e.push(idx))
                    .or_insert_with(|| vec![idx]);
            }

            self.tri_normals.push([0, 0, 0]);
        }

        // For each vertex, compute all of its adjoined triangles' normals
        for (vert_idx, tris) in vert_map.into_iter() {
            let normals = tris
                .iter()
                .map(|idx| &self.tris[*idx])
                .map(|tri| {
                    triangle_normal(self.verts[tri[0]], self.verts[tri[1]], self.verts[tri[2]])
                })
                .collect::<Vec<_>>();

            // Sum up all of the normals
            let mut agg_norm = Vector3::default();
            for normal in normals.iter() {
                agg_norm += *normal;
            }

            // Insert the calculated normal into the normals Vec
            self.normals
                .push((agg_norm / normals.len() as f64).normalize());

            for tri in tris {
                for n in 0..3 {
                    if self.tris[tri][n] == vert_idx {
                        self.tri_normals[tri][n] = self.normals.len() - 1;
                    }
                }
            }
        }
    }

    /// (Re)generate this mesh's SBVH.
    pub fn generate_sbvh(&mut self) {
        // Bake the mesh's triangles into a list of SBVH tris.
        let tris = self
            .tris
            .iter()
            .map(|tri| {
                acceleration::Triangle::new(
                    self.verts[tri[0]],
                    self.verts[tri[1]],
                    self.verts[tri[2]],
                )
            })
            .collect::<Vec<_>>();

        self.sbvh = Some(acceleration::Sbvh::new(&tris).into());
    }

    /// Shift all vertices by some vector.
    pub fn shift(&mut self, delta: Vector3) {
        self.verts.iter_mut().for_each(|v| *v += delta);
    }

    pub fn center(&mut self) {
        let mut min = VECTOR_MAX;
        let mut max = VECTOR_MIN;

        for v in self.verts.iter() {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        self.shift((min + max) * -0.5);
    }

    /// Scale all vertices by some vector.
    pub fn scale(&mut self, delta: f64) {
        self.verts.iter_mut().for_each(|v| *v *= delta);
    }

    /// Rotate the mesh in XYZ order.
    pub fn rotate_xyz(&mut self, rot: Vector3) {
        let rot = Matrix::from_euler_xyz(-rot.x, -rot.y, -rot.z);

        for vert in self.verts.iter_mut() {
            *vert = (rot * Matrix::from(*vert)).pos();
        }

        for norm in self.normals.iter_mut() {
            *norm = (rot * Matrix::from(*norm)).pos();
        }
    }

    /// Rotate the mesh in ZYX order.
    pub fn rotate_zyx(&mut self, rot: Vector3) {
        let rot = Matrix::from_euler_zyx(-rot.x, -rot.y, -rot.z);

        for vert in self.verts.iter_mut() {
            *vert = (rot * Matrix::from(*vert)).pos();
        }

        for norm in self.normals.iter_mut() {
            *norm = (rot * Matrix::from(*norm)).pos();
        }
    }

    fn sbvh_intersection(&self, node: &acceleration::TreeNode, ray: &Ray) -> Option<Vec<usize>> {
        if !node.bounding().intersect(ray) {
            return None;
        }

        match node {
            acceleration::TreeNode::Branch { a, b, .. } => {
                match (
                    self.sbvh_intersection(a, ray),
                    self.sbvh_intersection(b, ray),
                ) {
                    (Some(av), Some(bv)) => Some(av.into_iter().chain(bv).collect()),
                    (Some(av), None) => Some(av),
                    (None, Some(bv)) => Some(bv),
                    (None, None) => None,
                }
            }
            acceleration::TreeNode::Leaf { indices, .. } => Some(indices.clone()),
        }
    }
}

impl Intersect for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        assert!(self.sbvh.is_some());

        let tris = match self.sbvh_intersection(self.sbvh.as_ref().unwrap(), ray) {
            Some(v) => v,
            None => return None,
        }
        .into_iter()
        .map(|i| (i, &self.tris[i]))
        .collect::<Vec<_>>();

        if tris.is_empty() {
            return None;
        }

        // find the triangles that intersect our ray
        let mut intersected_tris = tris
            .iter()
            .filter_map(|(i, t)| {
                triangle_intersect(self.verts[t[0]], self.verts[t[1]], self.verts[t[2]], ray)
                    .map(|h| (i, t, h))
            })
            .collect::<Vec<_>>();

        // then sort them by nearness
        intersected_tris.sort_by(|(_, _, ta), (_, _, tb)| {
            ta.t.partial_cmp(&tb.t).unwrap_or(std::cmp::Ordering::Equal)
        });

        // return based on how many triangles we have
        match intersected_tris.len() {
            // no hits: return no hit
            0 => None,

            // one hit: return the only hit, where t_far is also t_near
            1 => {
                let t = &intersected_tris[0];
                Some(Hit::new(
                    triangle_intersect_normal(
                        self.tri_normals[*t.0][0],
                        self.tri_normals[*t.0][1],
                        self.tri_normals[*t.0][2],
                        &self.normals,
                        &t.2,
                    ),
                    (t.2.t, t.2.p),
                    (t.2.t, t.2.p),
                    if self.tri_texcoords.len() > 0 {
                        triangle_intersect_uvs(
                            self.tri_texcoords[*t.0][0],
                            self.tri_texcoords[*t.0][1],
                            self.tri_texcoords[*t.0][2],
                            &self.texcoords,
                            &t.2,
                        )
                    } else {
                        (0., 0.)
                    },
                ))
            }

            // 2+ hits: return the first hit, but with a proper t_far
            _ => {
                let t = &intersected_tris[0];
                let t1 = &intersected_tris[1];
                Some(Hit::new(
                    triangle_intersect_normal(
                        self.tri_normals[*t.0][0],
                        self.tri_normals[*t.0][1],
                        self.tri_normals[*t.0][2],
                        &self.normals,
                        &t.2,
                    ),
                    (t.2.t, t.2.p),
                    (t1.2.t, t1.2.p),
                    if self.tri_texcoords.len() > 0 {
                        triangle_intersect_uvs(
                            self.tri_texcoords[*t.0][0],
                            self.tri_texcoords[*t.0][1],
                            self.tri_texcoords[*t.0][2],
                            &self.texcoords,
                            &t.2,
                        )
                    } else {
                        (0., 0.)
                    },
                ))
            }
        }
    }
}

impl SceneObject for Mesh {
    fn material(&self) -> &Material {
        &self.material
    }
}
