# sdl

This crate is the interpreter for the raytracer's SDL (scene description language). It is
a (mostly) declarative language that is responsible for describing the objects in the scene
to the raytracer. It is designed to be fast and readable. It is reminiscent of POV-Ray's
SDL, JSON, and maybe even shader languages like GLSL or HLSL.

## Specification

The `sdl` crate is capable of rendering scenes from `sdl` files. Some examples are in
the `scenes` folder in this repository.

Comments can be added anywhere with `#`. Everything after a comment indicator will be ignored
until the next newline.

## Types and values

As far as values go, there are a few primitive types:

* Numbers, which are constructed with literal numbers like `1`, `2.4`, `-5.0`, ...
* Strings, which are constructed with literal strings like `"hello world!"`, `"I say \"hello\""`, ...
* Booleans, which are constructed with the keywords `true`/`yes` or `false`/`no`
* Vectors, which are constructed with the syntax `<x, y, z>`, where x, y, and z are numbers, like `<0, 0, 0>`, `<-1.2, 3.4, 5.06>`
* Colors, which are constructed with the familiar function call syntax `color(r, g, b)`, where r, g, and b are numbers from 0-255
* Dictionaries, which are constructed much like JSON objects. They are wrapped in curly braces and are a collection of comma-separated key-values, like `{key: value, another_key: another_value}`

## Functions

There are a number of functions that can be used as values.

#### Operators

* `add(x, y)` adds two values together
* `sub(x, y)` subtracts two values from one another
* `mul(x, y)` multiplies two values together
* `div(x, y)` divides two values from each other

#### Constructors

* `vec(x, y, z)` constructs a vector from 3 numbers (this is useful as syntax like `<pi(), pi(), pi()>` is not supported. instead, prefer `vec(pi(), pi(), pi())`)

#### Floating point functions

* `sin(x)`, `cos(y)`, and `tan(z)` are all traditional trigonometric functions
* `asin(x)`, `acos(y)`, and `atan(z)` are all traditional inverse trigonometric functions
* `abs(x)` returns the absolute value of x
* `floor(x)` returns the floor of x
* `ceil(y)` returns the ceiling of y
* `random(x, y)` returns a random floating point number between `x` and `y`, inclusive

#### Vector functions

* `normalize(v)` returns a normalized v
* `magnitude(v)` returns the magnitude of v
* `angle(a, b)` returns the angle between vectors a and b

#### Constants

* `pi()` returns 3.141592653...

### Objects

At the top-level of every SDL file, a number of objects can be declared. An object is in
the form:

```
[object name] {
    [properties]
}
```

There are a collection of valid object names, like

* `camera`, used to define the camera transform and viewport\*
* `aabb`, an object that is an axis-aligned bounding box
* `mesh`, an object that can be loaded from an `obj` file and is a mesh
* `plane`, an object that is a plane
* `sphere`, an object that is a sphere
* `point_light`, a point light
* `sun`, a sun light

*\* This object can only be defined once.*

#### Object properties

The properties of an object are a dictionary of comma-separated keys and values, like

```
object {
    key: value,
    another_key: another_value,
}
```

### Example object

Below is an example object. Each part will be explained beneath it.

```
aabb {
    position: <0, 0, 0>,
    size: <1, 1, 1>,
    material: {
        texture: solid(color(255, 80, 80)),
        reflectiveness: 0.6,
    }
}
```

This syntax creates an AABB object at the origin `<0, 0, 0>` with size `<1, 1, 1>`. Its material
declaration says that it has a solid red color texture, and is 60% reflective.

Each object has its own properties. For example, the `position` property on `aabb` is not valid
for, say, `camera`. Read on to see what properties are valid for what objects.

#### Specific object properties

* `camera` (defined once)
  * `vw` (number), the view width
  * `vh` (number), the view height
  * `origin` (vector), the origin of the camera
  * `yaw` (number), the yaw of camera rotation in radians
  * `pitch` (number), the pitch of camera rotation in radians
  * `fov` (number), the field of view of the camera in degrees
* `aabb` (a scene object)
  * `position`\* (vector), the center of the AABB
  * `size`\* (vector), the distance from one corner to the center of the AABB (radial size if you will)
  * `material` (dictionary), see below
* `mesh` (a scene object)
  * `mesh`\* (string), the filename of the OBJ to load from
  * `position` (vector), the center of the mesh
  * `scale` (number), the scale factor
  * `material` (dictionary), see below
* `plane` (a scene object)
  * `origin`\* (vector), the origin of the plane
  * `normal` (vector), the normal vector of the plane
  * `uv_wrap` (number), the number of units before UVs on the plane wrap around
  * `material` (dictionary), see below
* `sphere` (a scene object)
  * `position`\* (vector), the position of the sphere
  * `radius`\* (number), the radius of the sphere
  * `material` (dictionary), see below
* `point_light` | `pointlight` (a light)
  * `position`\* (vector), the position of the point light
  * `color` (color), the color of the light
  * `intensity` (number), the intensity of the light
  * `specular_power` (number), the power to raise specular light to
  * `specular_strength` (number), the coefficient of specular light
  * `max_distance` (number), the max distance a hit can be before this light is no longer considered
* `sun` | `sun_light` | `sunlight` (a light)
  * `vector`\* (vector), the vector this sun is facing (automatically normalized)
  * `color` (color), the color of the sun
  * `intensity` (number), the sun's intensity
  * `specular_power` (number), the power to raise specular light to
  * `specular_strength` (number), the coefficient of specular light
  * `shadows` (boolean), whether or not this sun should draw shadows
  * `shadow_coefficient` (number), what % of normal object color ambient light should be, from 0 - 1

*\* This property is required.*

### Material declaration

On all scene objects, the `material` property can be linked to a dictionary with the following
properties:

* `texture`, which can be one of the following:
  * `solid(color)`, which sets the texture to a solid color, e.g. `texture: solid(color(255, 0, 0))`
  * `checkerboard(color_a, color_b)`, which sets the texture to a 2x2 checkerboard of colors `color_a` and `color_b`, e.g. `texture: checkerboard(color(0, 0, 0), color(255, 255, 255))`
  * `image(filename)`, which sets the texture to an image loaded from `filename`, e.g. `texture: image("assets/texture.png")`
* `reflectiveness`, which is a number from 0 - 1, representing how reflective the object is
* `transparency`, which is a number from 0 - 1, representing how opaque or transparent the object is
* `ior`, the index of refraction

## An example scene

Here is an example scene that renders a fedora, from `assets/fedora.obj` and `assets/fedora.png`.
The fedora will randomly be bigger or smaller in size.

```
camera {
    vw: 1920,
    vh: 1080,
    origin: <1.5, 0.6, 3>,
    yaw: -0.5,
    pitch: -0.3,
}

sun {
    vector: <-0.8, -1, -0.3>,
    intensity: 0.8,
    specular_power: 64,
}

mesh {
    obj: "assets/fedora.obj",
    position: <0, -0.5, 0>,
    scale: random(0.5, 1.5),
    material: {
        texture: image("assets/fedora.png"),
    }
}

plane {
    origin: <0, -1, 0>,
    material: {
        reflectiveness: 0.6,
    }
}
```
