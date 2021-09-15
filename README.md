# raytracer

This is a raytracer written in Rust for my apprenticeship project at UW-L.

TODO: write more of me

## Things to research

Since the goal of this raytracer is to be fast, here are some things I want to research more on:

* Acceleration structures
  * Octrees (I have an implementation of this already)
  * BVH ([this paper by Nvidia](https://www.nvidia.in/docs/IO/77714/sbvh.pdf) looks like a great resource)
  * More?
* Better parallelism (currently made possible by Rayon, a data parallelism library)
* Run on the GPU somehow?
  * If not, look into using SIMD for accelerated ray intersection math

## Progress

#### 9/14/2021

Today, we decided that I would work on a raytracer in Rust. I began working on it. By the end of the day,
it is capable of rendering spheres, planes, meshes (arbitrary wavefront OBJ models), reflections, sun lights,
and can shade with Blinn-Phong shading. I parallelized it with the Rust library Rayon.

Below is a screenshot, 800x600, that renders in 0.022 seconds.

![Progress screenshot from 9/14/2021](/images/readme/9_14_2021.png)
