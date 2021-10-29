# raytracer

This is a raytracer written in Rust for my apprenticeship project at UW-L.

This project is my current work for my apprenticeship project at the University of Wisconsin-La
Crosse. It is a rewrite of an [older project](https://github.com/voximity/omegga-raytracer-cr)
in Rust, designed for maximum performance. Over the course of this school year, I am working with
Dr. Kenny Hunt to research rendering and optimization techniques as a way to further my
understanding of high-performance computing, especially with parallelization.

## Crates

As of now, there are several crates in this project:

* `raytracer` - The raytracer itself, which takes a scene, raytraces it, and outputs it to a file.
* `stitcher` - A cubemap stitcher. Provided 6 cubemap faces, this outputs a single atlas that can be used by the raytracer.
* `sdl` - The raytracer's proprietary scene description language, loosely inspired by POV-Ray's. This crate has its own tokenizer, AST, and interpreter for parsing SDL files.

For more information on the SDL (scene description language), please visit [its README](/sdl/README.md).

## Using

Start by building with `cargo build --release -p sdl`.

You can render a scene with `./target/release/sdl <source file> [-o <output file>]`. For example,
you can render `fedora.sdl` to `fedora.png` with `./target/release/sdl fedora.sdl -o fedora.png`.

To write your own scene, see [the `sdl` README](/sdl/README.md).

## Contributions

You are welcome to fork and tinker with this project, but I will not be accepting contributions.
Sorry!

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
- [x] A scene description method
  - [x] Variables
  - [x] Loops
  - [x] Functions
  - [x] User-defined functions

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

#### 10/7/2021

At this point, the SDL is in a usable state. It can successfully describe and render the following
scene:

```
# This is a test scene.

camera {
    # Render the image as 1920x1080.
    vw: 1920,
    vh: 1080,

    fov: 60,
    origin: <-3, 3, 0>,
    pitch: -0.5,
    yaw: 0.4,
}

# We just need one sun light.
sun {
    vector: <-0.8, -1, -0.3>,
    intensity: 0.8,
    specular_power: 64,
}

# This is the main UWL cube in the front.
aabb {
    position: vec(random(-1, 1), random(0, 1), random(-9, -7)),
    size: <1, 1, 1>,
    material: {
        texture: image("assets/uwl.png"),
    }
}

# This cube is reflective, to the right of the cube.
sphere {
    position: <2.5, 0, -5>,
    radius: 1,
    material: {
        texture: solid(color(200, 200, 200)),
        reflectiveness: 0.7,
    }
}

# This cube is solid opaque, to the left of the cube.
sphere {
    position: <-2.5, 0, -5>,
    radius: 1,
    material: {
        texture: solid(color(200, 200, 200)),
    }
}

# This is a wall behind all of the objects.
aabb {
    position: <0, 2, -12>,
    size: <10, 3, 1>,
}

# Finally, a checkered ground plate.
plane {
    origin: <0, -1, 0>,
    material: {
        texture: checkerboard(color(128, 128, 128), color(255, 255, 255)),
    }
}

point_light {
    position: <0, 0, -3.5>,
    color: color(255, 100, 100),
    intensity: 2,
}

```

The SDL is capable of functions like `sin(x)`, `cos(x)`, `normalize(vector)`, `add(vector, vector)`,
`mul(x, y)`, and so on. You can safely nest functions, add comments with `#`, add any number of objects,
add image textures, and more. This is all done in the `sdl` crate.

![Progress screenshot from 10/7/2021](/images/readme/10_7_2021.png)

#### 10/19/2021

Today, I got scope stack and for loops working. The following code produces the following image:

```
tau = mul(pi(), 2)
segments = 16

sun {
    vector: <-0.8, -1, -0.2>,
}

for i in 0 to segments {
    frac = div(i, segments)
    inner = mul(frac, tau)
    color_channel = mul(frac, 255)

    sphere {
        position: vec(cos(inner), sin(inner), -4),
        radius: 0.3,
        material: {
            texture: solid(color(color_channel, color_channel, color_channel)),
        }
    }
}
```

![Progress screenshot from 10/19/2021](/images/readme/10_19_2021.png)

#### 10/20/2021

I have done a lot with the SDL today! I refined variables, added a [Shunting-yard](https://en.wikipedia.org/wiki/Shunting-yard_algorithm)
expression parser, streamlined the vector constructor, and added time parameterization to create GIFs.
The SDL can now generate a sequence of PNGs, and then you can use a tool like FFmpeg to convert these
to a coherent GIF. Here is a coherent GIF below generated from the adjacent code:

![Progress animation from 10/20/2021](/images/readme/10_20_2021.gif)

```
# Some variables for quick customization. We insert them into our camera...
let vw = 500
let vh = 500
let fov = 40
camera { vw, vh, fov, yaw: 0.0001, pitch: 0.0002 }

# Add a basic skybox.
skybox {
    type: "cubemap",
    image: "assets/space.png"
}

# Here are some variables defined in the top-level scope.
let dist = 3
let radius = 0.5
let n = 24
let time_scale = PI / 32

# Add a basic sun light...
sun {
    vector: <-0.8, -1, -0.2>,
    intensity: 0.8
}

# A for loop, over the range [0..n)
for i in 0 to n {
    # Add a sphere in each iteration...
    sphere {
        # With a position following a circle, with radius `dist`.
        position: <
            cos(i / n * TAU) * dist * cos(t * time_scale),
            sin(i / n * TAU) * dist,
            cos(i / n * TAU) * dist * sin(t * time_scale) - 12
        >,

        # Set the radius to our variable `radius`. Leaving out a value (e.g. `radius: 1`)
        # tries to pull a value out of a variable of the same name, in this case, one we set in the
        # top-level scope.
        radius,

        material: {
            # Use the HSV color constructor to pick colors off of a rainbow.
            texture: solid(hsv(i / n * 360, 1, 1)),
            reflectiveness: 0.2
        }
    }
}

# Finally one shiny sphere in the middle, because why not!
sphere {
    position: <0, 0, -13>,
    radius: 2,
    material: {
        reflectiveness: 0.8
    }
}
```

#### 10/28/2021

Since my last progress update, I have added a number of features to the SDL, as well as a few rendering features.
I've added area lights (with a shoddy implementation) as well as user-defined functions, comparison operators,
logic operators, if/if-else/if-else-if/if-else-if-else statements, and probably more.

Here's an example of fizz-buzz implemented in the scene description language:

```
for n in 1 to 50 {
    if n % 3 == 0 && n % 5 == 0 {
        print("FizzBuzz")
    } else if n % 3 == 0 {
        print("Fizz")
    } else if n % 5 == 0 {
        print("Buzz")
    } else {
        print(n)
    }
}
```
