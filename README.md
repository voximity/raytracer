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
- [x] Textures (Proper UVs for objects)
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

#### 9/16/2021

Along with a tiny bit of work last night, I added a feature my old raytracer did not have: textures. I went
through each primitive I'd implemented so far and added a way for a ray hit to also return the UV coordinates
of where to pull from its texture. For cubes, it's quite simple: each face just renders the image back out as
it was. For meshes, however, it's a lot more complicated. Every triangle vertice holds an index that refers
back to the `texcoords` list of UVs in the mesh itself. When a ray strikes a triangle, the barycentric UVs
are calculated, and later are converted to be in the space of the image. This took an immense amount of trial
and error. I had to try different permutations of UVW from barycentric coordinates (turns out the solution
was WUV), and mess with a bunch of other random trial-and-error stuff like not inverting `v` to be in the
space of the image (i.e. `v` should have been `1. - v` when moving to image space).

Here is an image of a fedora mesh with a texture that I ripped from the game Roblox for testing's sake.

![Progress screenshot from 9/16/2021](/images/readme/9_16_2021.png)

Later today, or at some point in the future, I'd like to work on adding normal maps (as well as maps for other
physical properties), but this will require some more implementation details.

Here's another scene I threw together with 8 lights and 10 objects. It is 2560x1440, and took 6.961 seconds
to render. Not bad, but not great.

![Progress screenshot from 9/16/2021, pt. 2](/images/readme/9_16_2021_2.png)
