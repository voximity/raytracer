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

## Things to add

Along with the above research considerations, here are some rendering features
I'd like to add in the future:

- [ ] Skyboxes
- [ ] Proper refraction
- [ ] Textures (Proper UVs for objects)
  - [ ] Normal maps
  - [ ] Reflectiveness maps
  - [ ] Roughness maps
- [ ] Ambient occlusion
- [ ] Global illumination *(possibly)*
- [ ] Caustics *(possibly, an extremely tricky subject)*

## Progress

#### 9/14/2021

Today, we decided that I would work on a raytracer in Rust. I began working on it. By the end of the day,
it is capable of rendering spheres, planes, meshes (arbitrary wavefront OBJ models), reflections, sun lights,
and can shade with Blinn-Phong shading. I parallelized it with the Rust library Rayon.

Below is a screenshot, 800x600, that renders in 0.022 seconds.

![Progress screenshot from 9/14/2021](/images/readme/9_14_2021.png)

#### 9/15/2021

Development was slower today because I have already implemented most features I would like to demo before I
go further. I worked on optimization a bit, made my Rust code more idiomatic, and added point lights.
Here is a screenshot, 1920x1080, that rendered in 0.102s.

![Progress screenshot from 9/15/2021](/images/readme/9_15_2021.png)
