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
