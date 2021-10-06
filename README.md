# raytracer

This is a raytracer written in Rust for my apprenticeship project at UW-L.

TODO: write more of me

## Crates

As of now, there are two crates in this project:

* `raytracer` - The raytracer itself, which takes a scene, raytraces it, and outputs it to a file.
* `stitcher` - A cubemap stitcher. Provided 6 cubemap faces, this outputs a single atlas that can be used by the raytracer.
* `sdl` - The raytracer's proprietary scene description language, loosely inspired by POV-Ray's. This crate has its own tokenizer, AST, and interpreter for parsing SDL files.
* `sdl_lua` - An SDL runtime that uses Lua to describe a scene. This crate is only included for completeness; it is not in a functional state.

## Things to research

Since the goal of this raytracer is to be fast, here are some things I want to research more on:

* Acceleration structures
  * Octrees (I have an implementation of this already)
  * BVH ([this paper by Nvidia](https://www.nvidia.in/docs/IO/77714/sbvh.pdf) looks like a great resource)
    * This has been implemented!
  * More?
* Better parallelism (currently made possible by Rayon, a data parallelism library)
* Run on the GPU somehow?
  * If not, look into using SIMD for accelerated ray intersection math

## Things to add

Along with the above research considerations, here are some rendering features
I'd like to add in the future:

- [x] Skyboxes
- [x] Proper refraction
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

#### 9/20/2021

I took a break because I went home this weekend, but today I added skybox support, including support for
cubemaps. Here's what the previous scene looks like with a cubemap I stole from Google Images:

![Progress screenshot from 9/20/2021](/images/readme/9_20_2021.png)

#### 10/3/2021

I haven't updated this in a while but since, I've added a number of extra textures, refraction, and most
importantly, an implementation of a BVH that builds from top-down, making mesh renders extremely fast.

Here's a scene with one light, 4 objects, a semi-detailed mesh, and refraction + reflections to
demonstrate. It is 1920x1080, and rendered in 0.1127 seconds.

![Progress screenshot from 10/3/2021](/images/readme/10_3_2021.png)

#### 10/4/2021

Today, I realized that scene construction when loading assets into memory is actually pretty slow, so I
added a way to differentiate between scene construction time and render time, which is a very important
thing to take into account for this scene in particular:

![Progress screenshot from 10/4/2021](/images/readme/10_4_2021.png)

This image renders in 0.41624472s, but 0.3187096s of those are dedicated to scene construction. This includes
loading assets into memory and processing them (decoding image files, reading OBJ files, etc.), but presumably
almost all of it is loading and decoding the 3 MB cubemap into memory. This means this render *actually* took
0.09753512s, which is very fast at 1920x1080, with a mesh, refractions, and reflections.

Here's an image of the same scene from above:

![Progress screenshot from 10/4/2021](/images/readme/10_4_2021_2.png)

#### 10/6/2021

Today, I started working on the SDL tokenizer and AST. It can parse basic SDL files. At this point, it is
capable of describing objects and their properties, but I would like to add some more imperative programming
constructs like loops over a range to automatically construct circles.

As of now, the SDL looks something like this:

```
sphere {
  position: <0, 0, 0>,
  radius: 1,
  material: {
    color: <1, 0, 0>,
    reflectiveness: 0.3,
  },
}

aabb {
  position: <3, 3, 3>,
  size: <4, 2, 2>,
  material: {
    color: <0, 0.6, 1>,
  }
}
```
