// This is an implementation of sbvh-rs that uses 64-bit floating precision,
// as well as integrates a bit better with my scene.
// I do not take credit for this technique or this code! I have simply
// rewritten it to work for my case.
//
// https://github.com/rytone/sbvh

use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{math::{Axis, Ray, VECTOR_MAX, VECTOR_MIN, Vector3}, object::AabbIntersector};

const OBJECT_BUCKETS: usize = 32;

/// An atomic arena object for quick read/writes between threads.
pub struct AtomicArena<T> {
    pub storage: UnsafeCell<Box<[MaybeUninit<T>]>>,
    pub len: AtomicUsize,
}

unsafe impl<T: Send> Send for AtomicArena<T> {}
unsafe impl<T: Sync> Sync for AtomicArena<T> {}

impl<T> AtomicArena<T> {
    pub fn new(size: usize) -> Self {
        Self {
            storage: UnsafeCell::new(Box::new_uninit_slice(size)),
            len: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, v: T) -> usize {
        let idx = self.len.fetch_add(1, Ordering::SeqCst);
        unsafe {
            (&mut *self.storage.get())[idx] = MaybeUninit::new(v);
        }
        idx
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i < self.len.load(Ordering::SeqCst) {
            unsafe { Some((&*self.storage.get())[i].assume_init_ref()) }
        } else {
            None
        }
    }
}

// TODO Figure out how to drop everything at once
impl<T> Drop for AtomicArena<T> {
    fn drop(&mut self) {
        for i in 0..self.len.load(Ordering::SeqCst) {
            unsafe { ptr::drop_in_place(self.storage.get_mut()[i].as_mut_ptr()) }
        }
    }
}

/// An axis-aligned bounding box, stored by its minimum and maximum corners.
#[derive(Debug, Clone)]
pub struct Aabb {
    pub centroid: Vector3,
    pub min: Vector3,
    pub max: Vector3,
}

impl Aabb {
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self { centroid: (min + max) * 0.5, min, max }
    }

    pub fn from_vecs(vecs: &[Vector3]) -> Self {
        let mut min = VECTOR_MAX;
        let mut max = VECTOR_MIN;

        for v in vecs {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);

            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        Self { centroid: (min + max) * 0.5, min, max }
    }

    pub fn union(&self, other: &Self) -> Self {
        let min = Vector3::new(
            self.min.x.min(other.min.x),
            self.min.y.min(other.min.y),
            self.min.z.min(other.min.z),
        );
        let max = Vector3::new(
            self.max.x.max(other.max.x),
            self.max.y.max(other.max.y),
            self.max.z.max(other.max.z),
        );

        Self { centroid: (min + max) * 0.5, min, max }
    }

    pub fn surface_area(&self) -> f64 {
        let xs = self.max.x - self.min.x;
        let ys = self.max.y - self.min.y;
        let zs = self.max.z - self.min.z;

        2. * xs * ys + 2. * xs * zs + 2. * ys * zs
    }

    pub fn extent(&self, axis: Axis) -> f64 {
        self.max.axis(axis) - self.min.axis(axis)
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        let size = self.max - self.centroid;

        let nro = self.centroid - ray.origin;
        let s = Vector3::new(
            -ray.direction.x.signum(),
            -ray.direction.y.signum(),
            -ray.direction.z.signum(),
        );

        let ri = ray.inverse();
        let ssize = s * size;
        let t1 = ri * (nro + ssize);
        let t2 = ri * (nro - ssize);
        let tn = f64::max(f64::max(t1.x, t1.y), t1.z);
        let tf = f64::min(f64::min(t2.x, t2.y), t2.z);

        !(tn > tf || tf < 0.)
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            centroid: Vector3::new(0., 0., 0.),
            min: VECTOR_MAX,
            max: VECTOR_MIN,
        }
    }
}

impl From<AabbIntersector> for Aabb {
    fn from(intersector: AabbIntersector) -> Self {
        let (min, max) = intersector.bounds();
        Aabb { centroid: (min + max) * 0.5, min, max }
    }
}

#[derive(Debug, Clone)]
pub struct Split {
    pub axis: Axis,
    pub position: f64,
}

pub trait Primitive: Sized {
    fn points(&self) -> &[Vector3];
    fn split(&self, split: Split) -> (Self, Option<Self>);
    fn bounding_box(&self) -> &Aabb;
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub prim_idx: usize,
    pub bounding_box: Aabb,
}

#[derive(Debug, Clone, Default)]
pub struct ObjectBucket {
    bounding_box: Aabb,
    count: usize,
}

fn object_split_candidate(
    refs: &[Reference],
    aabb_total: &Aabb,
    input: &[usize],
    split_axis: Axis,
) -> (f64, usize) {
    let mut buckets: [ObjectBucket; OBJECT_BUCKETS] = Default::default();
    let extent = aabb_total.extent(split_axis);

    // place refs into buckets
    for idx in input {
        let bucket_idx = ((refs[*idx].bounding_box.centroid.axis(split_axis)
            - aabb_total.min.axis(split_axis))
            / extent
            * (OBJECT_BUCKETS as f64)) as usize;

        buckets[bucket_idx].count += 1;
        buckets[bucket_idx].bounding_box = buckets[bucket_idx]
            .bounding_box
            .union(&refs[*idx].bounding_box);
    }

    // compute all split left-hand sides
    let mut lhs_splits: [(usize, Aabb); OBJECT_BUCKETS - 1] = Default::default();
    let mut lhs_split = (0, Aabb::default());
    for i in 0..OBJECT_BUCKETS - 1 {
        lhs_split.0 += buckets[i].count;
        lhs_split.1 = lhs_split.1.union(&buckets[i].bounding_box);
        lhs_splits[i] = lhs_split.clone();
    }

    // compute split rhs and find split with minimum SAH
    let mut rhs_split = (0, Aabb::default());
    let mut min_cost = f64::MAX;
    let mut min_idx = 0;
    for i in (1..OBJECT_BUCKETS).rev() {
        rhs_split.0 += buckets[i].count;
        rhs_split.1 = rhs_split.1.union(&buckets[i].bounding_box);

        let lhs_split = &lhs_splits[i - 1];
        let traverse_cost = 1.;
        let n_lhs = lhs_split.0 as f64;
        let n_rhs = rhs_split.0 as f64;
        let cost = traverse_cost
            + (n_lhs * lhs_split.1.surface_area() + n_rhs * rhs_split.1.surface_area())
                / aabb_total.surface_area();

        if cost < min_cost {
            min_cost = cost;
            min_idx = i;
        }
    }

    (min_cost, min_idx)
}

fn object_split(
    refs: &[Reference],
    aabb_total: &Aabb,
    input: &[usize],
    output: &mut [usize],
    split_axis: Axis,
    bucket_idx: usize,
) -> usize {
    let extent = aabb_total.extent(split_axis);

    let mut lhs_ptr = 0;
    let mut rhs_ptr = output.len() - 1;
    for idx in input {
        let this_bucket = ((refs[*idx].bounding_box.centroid.axis(split_axis)
            - aabb_total.min.axis(split_axis))
            / extent
            * (OBJECT_BUCKETS as f64)) as usize;

        if this_bucket < bucket_idx {
            output[lhs_ptr] = *idx;
            lhs_ptr += 1;
        } else {
            output[rhs_ptr] = *idx;
            rhs_ptr -= 1;
        }
    }

    lhs_ptr
}

#[derive(Debug)]
pub enum SbvhNode {
    Node { lhs: usize, rhs: usize },
    Leaf { refs: Vec<Reference> },
}

pub struct Sbvh {
    pub nodes: AtomicArena<SbvhNode>,
    pub root_node: usize,
}

impl Sbvh {
    pub fn new<P: Primitive>(prims: &[P]) -> Self {
        let refs = prims
            .iter()
            .enumerate()
            .map(|(i, prim)| Reference {
                prim_idx: i,
                bounding_box: prim.bounding_box().clone(),
            })
            .collect::<Vec<_>>();

        let mut buffer_a = Vec::with_capacity(refs.len());
        for i in 0..refs.len() {
            buffer_a.push(i);
        }
        let mut buffer_b = vec![0; refs.len()];

        let output = AtomicArena::new(2 * refs.len() - 1);

        let root_node = Self::build(&refs, &mut buffer_a, &mut buffer_b, &output);

        Self {
            nodes: output,
            root_node,
        }
    }

    fn build(
        refs: &[Reference],
        input_buffer: &mut [usize],
        output_buffer: &mut [usize],
        output: &AtomicArena<SbvhNode>,
    ) -> usize {
        if input_buffer.len() < 2 {
            return Self::create_leaf(refs, input_buffer, output);
        }

        let mut aabb_total = Aabb::default();
        for idx in input_buffer.iter() {
            aabb_total = aabb_total.union(&refs[*idx].bounding_box);
        }

        let (ex, ey, ez) = (
            aabb_total.extent(Axis::X),
            aabb_total.extent(Axis::Y),
            aabb_total.extent(Axis::Z),
        );
        let split_axis = if ex >= ey && ex >= ez {
            Axis::X
        } else if ey >= ez {
            Axis::Y
        } else {
            Axis::Z
        };

        let leaf_cost = input_buffer.len() as f64;

        let (object_split_cost, object_split_bucket) =
            object_split_candidate(refs, &aabb_total, input_buffer, split_axis);

        if object_split_cost < leaf_cost {
            let split_point = object_split(
                refs,
                &aabb_total,
                input_buffer,
                output_buffer,
                split_axis,
                object_split_bucket,
            );

            let (a_in, b_in) = output_buffer.split_at_mut(split_point);
            let (a_out, b_out) = input_buffer.split_at_mut(split_point);

            let (lhs, rhs) = rayon::join(
                || Self::build(refs, a_in, a_out, output),
                || Self::build(refs, b_in, b_out, output),
            );

            output.push(SbvhNode::Node { lhs, rhs })
        } else {
            Self::create_leaf(refs, input_buffer, output)
        }
    }

    fn create_leaf(refs: &[Reference], idxs: &[usize], output: &AtomicArena<SbvhNode>) -> usize {
        output.push(SbvhNode::Leaf {
            refs: idxs.iter().map(|idx| refs[*idx].clone()).collect(),
        })
    }

    pub fn node_bounding_box(&self, idx: usize) -> Aabb {
        let node = self.nodes.get(idx).unwrap();
        match node {
            SbvhNode::Node { lhs, rhs } => self
                .node_bounding_box(*lhs)
                .union(&self.node_bounding_box(*rhs)),
            SbvhNode::Leaf { refs } => {
                let mut aabb = Aabb::default();
                for r in refs {
                    aabb = aabb.union(&r.bounding_box)
                }
                aabb
            }
        }
    }

    pub fn bounding_box(&self) -> Aabb {
        self.node_bounding_box(self.root_node)
    }
}

pub enum TreeNode {
    Branch {
        a: Box<TreeNode>,
        b: Box<TreeNode>,
        bounding: Aabb,
    },
    Leaf {
        indices: Vec<usize>,
        bounding: Aabb,
    },
}

fn sbvh_to_tree_node(sbvh: &Sbvh, idx: usize) -> TreeNode {
    let bounding = sbvh.node_bounding_box(idx);

    match sbvh.nodes.get(idx).unwrap() {
        SbvhNode::Leaf { refs } => TreeNode::Leaf {
            indices: refs.into_iter().map(|r| r.prim_idx).collect(),
            bounding,
        },
        SbvhNode::Node { lhs, rhs } => TreeNode::Branch {
            a: Box::new(sbvh_to_tree_node(sbvh, *lhs)),
            b: Box::new(sbvh_to_tree_node(sbvh, *rhs)),
            bounding,
        },
    }
}

impl Into<TreeNode> for Sbvh {
    fn into(self) -> TreeNode {
        sbvh_to_tree_node(&self, self.root_node)
    }
}
