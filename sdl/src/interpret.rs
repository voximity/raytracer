use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Read, Seek},
};

use image::{ImageBuffer, Rgb};
use lazy_static::lazy_static;
use rand::Rng;
use raytracer::{
    lighting::{self, AreaSurface},
    material::{Color, Material, Texture},
    math::{Lerp, Vector3},
    object,
    scene::Scene,
    skybox,
};
use thiserror::Error;

use crate::{
    ast::{self, AstError, AstParser, NodeKind},
    function::Function,
    tokenize::{TokenizeError, Tokenizer},
};

macro_rules! optional_property {
    ($self:ident, $scene:ident, $properties:ident, $name:literal, $k:ident) => {
        $self
            .optional_property($scene, &mut $properties, $name, ast::NodeKind::$k)?
            .map(|v| unwrap_variant!(v, Value::$k))
    };
}

macro_rules! required_property {
    ($self:ident, $scene:ident, $properties:ident, $name:literal, $k:ident) => {
        match optional_property!($self, $scene, $properties, $name, $k) {
            Some(a) => a,
            _ => return Err(InterpretError::RequiredPropertyMissing($name)),
        }
    };
}

macro_rules! unwrap_variant {
    ($matching:expr, $variant:path) => {
        match $matching {
            $variant(a) => a,
            _ => panic!("unwrapped variant {}", stringify!($variant)),
        }
    };
}

/// An interpreter error.
#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("tokenizer error: {0}")]
    Tokenizer(#[from] TokenizeError),

    #[error("ast error: {0}")]
    Ast(#[from] AstError),

    #[error("there are too many definitions of the {0} object, expecting zero or one")]
    NonUniqueObject(&'static str),

    #[error("unknown scene object/scene definition {0}")]
    UnknownObject(String),

    #[error("property {0} must be specified (it is not optional)")]
    RequiredPropertyMissing(&'static str),

    #[error("materials object must be a dictionary")]
    InvalidMaterials,

    #[error("invalid args to function call")]
    InvalidCallArgs,

    #[error("expected {0} args, got {1}")]
    InvalidArgCount(usize, usize),

    #[error("generic image error {0}")]
    ImageError(#[from] image::ImageError),

    #[error("no function by the name {0}")]
    UnknownFunction(String),

    #[error("no variable defined by the name {0}")]
    UndefinedVariable(String),

    #[error("cannot convert AST node to a definite type")]
    NonValueNode,
}

/// A definite value, which has been reduced from
/// an AST node that was a literal, a call, or a variable.
#[derive(Debug, Clone)]
pub enum Value {
    /// The unit type. Represents no data.
    Unit,

    /// A string.
    String(String),

    /// A number.
    Number(f64),

    /// A vector.
    Vector(Vector3),

    /// A color.
    Color(Color),

    /// A boolean.
    Boolean(bool),

    /// A dictionary.
    Dictionary(HashMap<String, Value>),
}

impl From<Value> for ast::Node {
    fn from(v: Value) -> Self {
        match v {
            Value::Unit => Self::Unit,
            Value::String(s) => Self::String(s),
            Value::Number(n) => Self::Number(n),
            Value::Vector(v) => Self::Vector(
                Box::new(Self::Number(v.x)),
                Box::new(Self::Number(v.y)),
                Box::new(Self::Number(v.z)),
            ),
            Value::Color(c) => Self::Color(c),
            Value::Boolean(b) => Self::Boolean(b),
            Value::Dictionary(m) => {
                Self::Dictionary(m.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl Value {
    fn from_node(
        interpreter: &mut Interpreter,
        scene: &mut Scene,
        node: ast::Node,
    ) -> Result<Self, InterpretError> {
        let value = match node {
            ast::Node::Identifier(name) => interpreter
                .variable_value(&name)
                .ok_or(InterpretError::UndefinedVariable(name))?,
            ast::Node::Call(name, args) => interpreter.call_func(scene, name, args)?,
            ast::Node::String(s) => Self::String(s),
            ast::Node::Number(n) => Self::Number(n),
            ast::Node::Vector(x, y, z) => {
                let x = Self::from_node(interpreter, scene, *x)?;
                let y = Self::from_node(interpreter, scene, *y)?;
                let z = Self::from_node(interpreter, scene, *z)?;
                Self::Vector(Vector3::new(
                    unwrap_variant!(x, Self::Number),
                    unwrap_variant!(y, Self::Number),
                    unwrap_variant!(z, Self::Number),
                ))
            }
            ast::Node::Color(c) => Self::Color(c),
            ast::Node::Boolean(b) => Self::Boolean(b),
            ast::Node::Dictionary(m) => Self::Dictionary(
                m.into_iter()
                    .filter_map(|(k, v)| {
                        Value::from_node(interpreter, scene, v).ok().map(|v| (k, v))
                    })
                    .collect(),
            ),
            // arithmetic operators
            ast::Node::Add(a, b) => Self::from_node(
                interpreter,
                scene,
                ast::Node::Call(String::from("add"), vec![*a, *b]),
            )?,
            ast::Node::Sub(a, b) => Self::from_node(
                interpreter,
                scene,
                ast::Node::Call(String::from("sub"), vec![*a, *b]),
            )?,
            ast::Node::Mul(a, b) => Self::from_node(
                interpreter,
                scene,
                ast::Node::Call(String::from("mul"), vec![*a, *b]),
            )?,
            ast::Node::Div(a, b) => Self::from_node(
                interpreter,
                scene,
                ast::Node::Call(String::from("div"), vec![*a, *b]),
            )?,
            _ => return Err(InterpretError::NonValueNode),
        };

        Ok(value)
    }

    fn from_nodes(
        interpreter: &mut Interpreter,
        scene: &mut Scene,
        nodes: Vec<ast::Node>,
    ) -> Result<Vec<Self>, InterpretError> {
        let mut values = vec![];
        for node in nodes.into_iter() {
            values.push(Value::from_node(interpreter, scene, node)?);
        }
        Ok(values)
    }
}

impl PartialEq<ast::NodeKind> for Value {
    fn eq(&self, other: &ast::NodeKind) -> bool {
        match (self, other) {
            (Self::Dictionary(_), ast::NodeKind::Dictionary) => true,
            (Self::String(_), ast::NodeKind::String) => true,
            (Self::Number(_), ast::NodeKind::Number) => true,
            (Self::Vector(_), ast::NodeKind::Vector) => true,
            (Self::Color(_), ast::NodeKind::Color) => true,
            (Self::Boolean(_), ast::NodeKind::Boolean) => true,
            _ => false,
        }
    }
}

/// A user-defined function.
#[derive(Debug, Clone)]
struct UserFunction {
    params: Vec<String>,
    body: Vec<ast::Node>,
}

/// A scope is a wrapper around a dictionary from identifier
/// to AST node. The AST node is expected to be fully reduced.
struct Scope {
    vars: HashMap<String, Value>,
    funcs: HashMap<String, UserFunction>,
}

/// The image cache, that is, a map between file names and loaded images.
type ImageCache = HashMap<String, ImageBuffer<Rgb<u8>, Vec<u8>>>;

/// The interpreter is the general runtime for the SDL interpreter. It is responsible for storing
/// AST data, scene data, and interpreting the AST at scene construction time to develop the
/// scene.
pub struct Interpreter {
    root: ast::Node,
    images: ImageCache,
    scope_stack: Vec<Scope>,
    object_names: Vec<String>,
}

impl Interpreter {
    /// Create a new interpreter. This will instantiate a `Tokenizer` and tokenize the input, as well
    /// as instantiate an `AstParser` and parse the tokenized input. From there, the interpreter
    /// can operate on the root AST node.
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self, InterpretError> {
        // inject constants into the global namespace
        let stack = vec![Scope {
            vars: vec![
                (String::from("PI"), Value::Number(std::f64::consts::PI)),
                (String::from("TAU"), Value::Number(std::f64::consts::TAU)),
                (String::from("E"), Value::Number(std::f64::consts::E)),
                (String::from("t"), Value::Number(0.)),
            ]
            .into_iter()
            .collect(),
            funcs: HashMap::new(),
        }];

        let tokens = Tokenizer::new(reader).tokenize()?;

        Ok(Interpreter {
            root: AstParser::new(tokens).parse_root()?,
            images: HashMap::new(),
            scope_stack: stack,
            object_names: Vec::new(),
        })
    }

    pub fn set_global(&mut self, identifier: String, value: Value) {
        self.scope_stack[0].vars.insert(identifier, value);
    }

    /// Start execution of the interpreter.
    pub fn run(mut self) -> Result<Scene, InterpretError> {
        let root = match self.root {
            ast::Node::Root(root) => root,
            _ => unreachable!(),
        };

        // this is so that `self` is not fully destructed
        // and we can continue to pass it to methods
        // that receive `&mut self`
        self.root = ast::Node::Root(vec![]);

        // instantiate a new scene
        let mut scene = Scene::default();

        // match nodes that can be in the root node
        self.run_scope(&mut scene, root)?;

        Ok(scene)
    }

    /// Start execution of the interpreter, cloning the root node.
    /// Use this if you are going to continually reuse the interpreter.
    pub fn run_cloned(&mut self) -> Result<Scene, InterpretError> {
        let root = match self.root.clone() {
            ast::Node::Root(root) => root,
            _ => unreachable!(),
        };

        // generate a scene
        let mut scene = Scene::default();

        // clear object names
        self.object_names = vec![];

        // execute the scene
        self.run_scope(&mut scene, root)?;

        Ok(scene)
    }

    fn run_scope(
        &mut self,
        scene: &mut Scene,
        nodes: Vec<ast::Node>,
    ) -> Result<Value, InterpretError> {
        for node in nodes.into_iter() {
            match node {
                ast::Node::Assign {
                    name,
                    declare,
                    value,
                } => {
                    let value = Value::from_node(self, scene, *value)?;
                    if declare {
                        // set in the top-most stack
                        self.scope_stack
                            .last_mut()
                            .unwrap()
                            .vars
                            .insert(name, value);
                    } else {
                        // assign to existing variable in nearest scope, or
                        // set it globally
                        for (i, scope) in self.scope_stack.iter_mut().enumerate().rev() {
                            match scope.vars.entry(name.clone()) {
                                Entry::Occupied(mut ent) => {
                                    ent.insert(value);
                                    break;
                                }
                                Entry::Vacant(ent) if i == 0 => {
                                    ent.insert(value);
                                    break;
                                }
                                _ => (),
                            }
                        }
                    }
                }
                ast::Node::For {
                    var,
                    from,
                    to,
                    body,
                } => {
                    let from = unwrap_variant!(Value::from_node(self, scene, *from)?, Value::Number)
                        .floor() as i32;
                    let to = unwrap_variant!(Value::from_node(self, scene, *to)?, Value::Number)
                        .floor() as i32;

                    for i in from..to {
                        // push a new scope to the stack with the index variable
                        self.scope_stack.push(Scope {
                            vars: vec![(var.clone(), Value::Number(i as f64))]
                                .into_iter()
                                .collect(),
                            funcs: HashMap::new(),
                        });

                        // run the scope body
                        self.run_scope(scene, body.clone())?;

                        // pop the scope from the stack
                        self.scope_stack.pop();
                    }
                }
                ast::Node::Function { name, params, body } => {
                    self.scope_stack
                        .last_mut()
                        .unwrap()
                        .funcs
                        .insert(name, UserFunction { params, body });
                }
                ast::Node::Return(value) => {
                    return Ok(Value::from_node(self, scene, *value)?);
                }
                ast::Node::Call(name, args) => {
                    self.call_func(scene, name, args)?;
                }
                ast::Node::Object {
                    name,
                    mut properties,
                } => {
                    match name.as_str() {
                        // one-time scene properties
                        "scene" => {
                            if self.object_names.iter().any(|n| n.as_str() == "scene") {
                                return Err(InterpretError::NonUniqueObject("scene"));
                            }

                            let max_ray_depth = optional_property!(
                                self,
                                scene,
                                properties,
                                "max_ray_depth",
                                Number
                            )
                            .map(|f| f as u32);
                            let ambient =
                                optional_property!(self, scene, properties, "ambient", Color);

                            if let Some(mrd) = max_ray_depth {
                                scene.options.max_ray_depth = mrd;
                            }

                            if let Some(ambient) = ambient {
                                scene.options.ambient = ambient;
                            }
                        }
                        "camera" => {
                            if self.object_names.iter().any(|n| n.as_str() == "camera") {
                                return Err(InterpretError::NonUniqueObject("camera"));
                            }

                            let vw = optional_property!(self, scene, properties, "vw", Number)
                                .map(|f| f as i32);
                            let vh = optional_property!(self, scene, properties, "vh", Number)
                                .map(|f| f as i32);
                            let origin =
                                optional_property!(self, scene, properties, "origin", Vector);
                            let yaw = optional_property!(self, scene, properties, "yaw", Number);
                            let pitch =
                                optional_property!(self, scene, properties, "pitch", Number);
                            let fov = optional_property!(self, scene, properties, "fov", Number);

                            if let Some(vw) = vw {
                                scene.camera.vw = vw;
                            }
                            if let Some(vh) = vh {
                                scene.camera.vh = vh;
                            }
                            if let Some(origin) = origin {
                                scene.camera.origin = origin;
                            }
                            if let Some(yaw) = yaw {
                                scene.camera.yaw = yaw;
                            }
                            if let Some(pitch) = pitch {
                                scene.camera.pitch = pitch;
                            }
                            if let Some(fov) = fov {
                                scene.camera.set_fov(fov);
                            }
                        }
                        "skybox" => {
                            if self.object_names.iter().any(|n| n.as_str() == "skybox") {
                                return Err(InterpretError::NonUniqueObject("skybox"));
                            }

                            let t = required_property!(self, scene, properties, "type", String);

                            match t.as_str() {
                                "normal" => scene.skybox = Box::new(skybox::Normal),
                                "solid" => {
                                    let color =
                                        required_property!(self, scene, properties, "color", Color);
                                    scene.skybox = Box::new(skybox::Solid(color));
                                }
                                "cubemap" => {
                                    let filename = required_property!(
                                        self, scene, properties, "image", String
                                    );
                                    let img = match self.images.entry(filename) {
                                        Entry::Occupied(buf) => buf.get().clone(),
                                        Entry::Vacant(ent) => {
                                            let img = image::open(ent.key())?.into_rgb8();
                                            ent.insert(img.clone());
                                            img
                                        }
                                    };

                                    scene.skybox = Box::new(skybox::Cubemap::new(img));
                                }
                                _ => return Err(InterpretError::InvalidMaterials),
                            }
                        }

                        // objects
                        "aabb" | "box" => {
                            let pos =
                                required_property!(self, scene, properties, "position", Vector);
                            let size = required_property!(self, scene, properties, "size", Vector);
                            let material = self.read_material(scene, properties)?;

                            scene
                                .objects
                                .push(Box::new(object::Aabb::new(pos, size, material)));
                        }
                        "mesh" => {
                            let obj = required_property!(self, scene, properties, "obj", String);
                            let position =
                                optional_property!(self, scene, properties, "position", Vector)
                                    .unwrap_or_else(|| Vector3::default());
                            let scale =
                                optional_property!(self, scene, properties, "scale", Number)
                                    .unwrap_or(1.);
                            let rotate_xyz =
                                optional_property!(self, scene, properties, "rotate_xyz", Vector);
                            let rotate_zyx =
                                optional_property!(self, scene, properties, "rotate_zyx", Vector);
                            let material = self.read_material(scene, properties)?;

                            let mut mesh = object::Mesh::from_obj(obj, material);

                            if scale != 1. {
                                mesh.scale(scale);
                            }

                            mesh.center();

                            if let Some(rotate_xyz) = rotate_xyz {
                                if rotate_zyx.is_some() {
                                    return Err(InterpretError::RequiredPropertyMissing(
                                        "one of rotate_xyz, rotate_zyx, not duplicates",
                                    ));
                                }

                                mesh.rotate_xyz(rotate_xyz);
                            }

                            if let Some(rotate_zyx) = rotate_zyx {
                                mesh.rotate_zyx(rotate_zyx);
                            }

                            if position != Vector3::default() {
                                mesh.shift(position);
                            }

                            mesh.recalculate();
                            scene.objects.push(Box::new(mesh));
                        }
                        "plane" => {
                            let origin =
                                required_property!(self, scene, properties, "origin", Vector);
                            let normal =
                                optional_property!(self, scene, properties, "normal", Vector)
                                    .unwrap_or_else(|| Vector3::new(0., 1., 0.))
                                    .normalize();
                            let uv_wrap =
                                optional_property!(self, scene, properties, "uv_wrap", Number)
                                    .map(|f| f as f32)
                                    .unwrap_or(1.);
                            let material = self.read_material(scene, properties)?;

                            scene.objects.push(Box::new(object::Plane {
                                origin,
                                normal,
                                material,
                                uv_wrap,
                            }));
                        }
                        "sphere" => {
                            let pos =
                                required_property!(self, scene, properties, "position", Vector);
                            let radius =
                                required_property!(self, scene, properties, "radius", Number);
                            let material = self.read_material(scene, properties)?;

                            scene
                                .objects
                                .push(Box::new(object::Sphere::new(pos, radius, material)));
                        }

                        // lights
                        "point_light" | "pointlight" => {
                            let default = lighting::Point::default();

                            let color = optional_property!(self, scene, properties, "color", Color);
                            let intensity =
                                optional_property!(self, scene, properties, "intensity", Number);
                            let specular_power = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_power",
                                Number
                            )
                            .map(|f| f as i32);
                            let specular_strength = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_strength",
                                Number
                            );
                            let position =
                                required_property!(self, scene, properties, "position", Vector);
                            let max_distance =
                                optional_property!(self, scene, properties, "max_distance", Number);

                            let light = lighting::Point {
                                color: color.unwrap_or(default.color),
                                intensity: intensity.unwrap_or(default.intensity),
                                specular_power: specular_power.unwrap_or(default.specular_power),
                                specular_strength: specular_strength
                                    .unwrap_or(default.specular_strength),
                                position,
                                max_distance: max_distance.unwrap_or(default.max_distance),
                            };

                            scene.lights.push(Box::new(light));
                        }
                        "sun" | "sun_light" | "sunlight" => {
                            let default = lighting::Sun::default();

                            let color = optional_property!(self, scene, properties, "color", Color);
                            let intensity =
                                optional_property!(self, scene, properties, "intensity", Number);
                            let specular_power = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_power",
                                Number
                            )
                            .map(|f| f as i32);
                            let specular_strength = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_strength",
                                Number
                            );
                            let vector =
                                required_property!(self, scene, properties, "vector", Vector)
                                    .normalize();
                            let shadows =
                                optional_property!(self, scene, properties, "shadows", Boolean);
                            let shadow_coefficient = optional_property!(
                                self,
                                scene,
                                properties,
                                "shadow_coefficient",
                                Number
                            );

                            let light = lighting::Sun {
                                color: color.unwrap_or(default.color),
                                intensity: intensity.unwrap_or(default.intensity),
                                specular_power: specular_power.unwrap_or(default.specular_power),
                                specular_strength: specular_strength
                                    .unwrap_or(default.specular_strength),
                                vector,
                                shadows: shadows.unwrap_or(default.shadows),
                                shadow_coefficient: shadow_coefficient
                                    .unwrap_or(default.shadow_coefficient),
                            };

                            scene.lights.push(Box::new(light));
                        }
                        "area_light" | "arealight" => {
                            let default = lighting::Area::default();

                            let color = optional_property!(self, scene, properties, "color", Color);
                            let intensity =
                                optional_property!(self, scene, properties, "intensity", Number);
                            let specular_power = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_power",
                                Number
                            )
                            .map(|f| f as i32);
                            let specular_strength = optional_property!(
                                self,
                                scene,
                                properties,
                                "specular_strength",
                                Number
                            );
                            let surface = match required_property!(
                                self, scene, properties, "surface", String
                            )
                            .as_str()
                            {
                                "sphere" => AreaSurface::Sphere(
                                    required_property!(self, scene, properties, "position", Vector),
                                    required_property!(self, scene, properties, "radius", Number),
                                ),
                                "rectangle" => AreaSurface::Rectangle([
                                    required_property!(self, scene, properties, "c00", Vector),
                                    required_property!(self, scene, properties, "c01", Vector),
                                    required_property!(self, scene, properties, "c10", Vector),
                                    required_property!(self, scene, properties, "c11", Vector),
                                ]),
                                _ => return Err(InterpretError::InvalidMaterials),
                            };
                            let iterations =
                                optional_property!(self, scene, properties, "iterations", Number);
                            let max_distance =
                                optional_property!(self, scene, properties, "max_distance", Number);

                            let light = lighting::Area {
                                color: color.unwrap_or(default.color),
                                intensity: intensity.unwrap_or(default.intensity),
                                specular_power: specular_power.unwrap_or(default.specular_power),
                                specular_strength: specular_strength
                                    .unwrap_or(default.specular_strength),
                                surface,
                                iterations: iterations
                                    .map(|f| f as u32)
                                    .unwrap_or(default.iterations),
                                max_distance: max_distance.unwrap_or(default.max_distance),
                            };

                            scene.lights.push(Box::new(light));
                        }
                        _ => return Err(InterpretError::UnknownObject(name.clone())),
                    }

                    self.object_names.push(name.clone());
                }
                _ => (),
            }
        }

        Ok(Value::Unit)
    }

    /// Read a material from a dictionary node.
    fn read_material(
        &mut self,
        scene: &mut Scene,
        mut properties: HashMap<String, ast::Node>,
    ) -> Result<Material, InterpretError> {
        match properties.remove("material") {
            Some(ast::Node::Dictionary(mut map)) => {
                let reflectiveness =
                    optional_property!(self, scene, map, "reflectiveness", Number).unwrap_or(0.);
                let transparency =
                    optional_property!(self, scene, map, "transparency", Number).unwrap_or(0.);
                let ior = optional_property!(self, scene, map, "ior", Number).unwrap_or(1.5);
                let emissivity =
                    optional_property!(self, scene, map, "emissivity", Number).unwrap_or(0.);

                let texture = match map.remove("texture") {
                    Some(node) => self.read_texture(scene, node)?,
                    None => Texture::Solid(Color::white()),
                };

                Ok(Material {
                    texture,
                    reflectiveness,
                    transparency,
                    ior,
                    emissivity,
                })
            }
            Some(_) => Err(InterpretError::InvalidMaterials),
            _ => Ok(Material::default()),
        }
    }

    /// Read a texture from a call node.
    ///
    /// A texture can be `solid(color(r, g, b))` or `checkerboard(color(r, g, b), color(r, g, b))`.
    fn read_texture(
        &mut self,
        scene: &mut Scene,
        node: ast::Node,
    ) -> Result<Texture, InterpretError> {
        match node {
            ast::Node::Call(name, args) => match name.as_str() {
                "solid" => {
                    let value = Value::from_nodes(self, scene, args)?;
                    let args = self.deconstruct_args(value, &[ast::NodeKind::Color])?;
                    Ok(Texture::Solid(unwrap_variant!(args[0], Value::Color)))
                }
                "checkerboard" => {
                    let value = Value::from_nodes(self, scene, args)?;
                    let args = self
                        .deconstruct_args(value, &[ast::NodeKind::Color, ast::NodeKind::Color])?;

                    Ok(Texture::Checkerboard(
                        unwrap_variant!(args[0], Value::Color),
                        unwrap_variant!(args[1], Value::Color),
                    ))
                }
                "image" => {
                    let value = Value::from_nodes(self, scene, args)?;
                    let args = self.deconstruct_args(value, &[ast::NodeKind::String])?;

                    match self.images.entry(unwrap_variant!(
                        args.into_iter().next().unwrap(),
                        Value::String
                    )) {
                        Entry::Occupied(buf) => Ok(Texture::Image(buf.get().clone())),
                        Entry::Vacant(ent) => {
                            let img = image::open(ent.key())?.into_rgb8();
                            ent.insert(img.clone());
                            Ok(Texture::Image(img))
                        }
                    }
                }
                _ => Err(InterpretError::InvalidCallArgs),
            },
            _ => Err(InterpretError::InvalidCallArgs),
        }
    }

    /// Call a named function with some arguments.
    /// Its result is another node that can be used as other values.
    fn call_func(
        &mut self,
        scene: &mut Scene,
        name: String,
        args: Vec<ast::Node>,
    ) -> Result<Value, InterpretError> {
        macro_rules! float_func {
            ($n:ident) => {
                |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number).$n()))
            };
        }

        macro_rules! vector_func {
            ($n:ident, $r:ident) => {
                |v| Ok(Value::$r(unwrap_variant!(v[0], Value::Vector).$n()))
            };
            ($n:ident, $r:ident,) => {
                |v| {
                    Ok(Value::$r(
                        unwrap_variant!(v[0], Value::Vector)
                            .$n(unwrap_variant!(v[1], Value::Vector)),
                    ))
                }
            };
        }

        lazy_static! {
            static ref FUNCTIONS: Vec<Function> = vec![
                // floating point operators
                Function::new(&["add"], &[NodeKind::Number, NodeKind::Number], |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number) + unwrap_variant!(v[1], Value::Number)))),
                Function::new(&["sub"], &[NodeKind::Number, NodeKind::Number], |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number) - unwrap_variant!(v[1], Value::Number)))),
                Function::new(&["mul"], &[NodeKind::Number, NodeKind::Number], |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number) * unwrap_variant!(v[1], Value::Number)))),
                Function::new(&["div"], &[NodeKind::Number, NodeKind::Number], |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number) / unwrap_variant!(v[1], Value::Number)))),
                Function::new(&["mod"], &[NodeKind::Number, NodeKind::Number], |v| Ok(Value::Number(unwrap_variant!(v[0], Value::Number) % unwrap_variant!(v[1], Value::Number)))),

                // vector operators
                Function::new(&["add"], &[NodeKind::Vector, NodeKind::Vector], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) + unwrap_variant!(v[1], Value::Vector)))),
                Function::new(&["sub"], &[NodeKind::Vector, NodeKind::Vector], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) - unwrap_variant!(v[1], Value::Vector)))),
                Function::new(&["mul"], &[NodeKind::Vector, NodeKind::Vector], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) * unwrap_variant!(v[1], Value::Vector)))),
                Function::new(&["div"], &[NodeKind::Vector, NodeKind::Vector], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) / unwrap_variant!(v[1], Value::Vector)))),

                // vector (*|/) floating point
                Function::new(&["mul"], &[NodeKind::Vector, NodeKind::Number], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) * unwrap_variant!(v[1], Value::Number)))),
                Function::new(&["div"], &[NodeKind::Vector, NodeKind::Number], |v| Ok(Value::Vector(unwrap_variant!(v[0], Value::Vector) / unwrap_variant!(v[1], Value::Number)))),

                // string operators
                Function::new(&["add"], &[NodeKind::String, NodeKind::String], |v| Ok(Value::String(format!("{}{}", unwrap_variant!(&v[0], Value::String), unwrap_variant!(&v[1], Value::String))))),

                // constructors
                Function::new(&["color", "rgb"], &[NodeKind::Number, NodeKind::Number, NodeKind::Number], |v| {
                    Ok(Value::Color(Color::new(
                        unwrap_variant!(v[0], Value::Number) as u8,
                        unwrap_variant!(v[1], Value::Number) as u8,
                        unwrap_variant!(v[2], Value::Number) as u8,
                    )))
                }),
                Function::new(&["hsv"], &[NodeKind::Number, NodeKind::Number, NodeKind::Number], |v| {
                    let (h, s, v) = (
                        unwrap_variant!(v[0], Value::Number) % 360.,
                        unwrap_variant!(v[1], Value::Number),
                        unwrap_variant!(v[2], Value::Number),
                    );

                    let c = v * s;
                    let x = c * (1. - (h / 60. % 2. - 1.).abs());
                    let m = v - c;

                    let (r, g, b) = if (0. ..60.).contains(&h) {
                        (c, x, 0.)
                    } else if (60. ..120.).contains(&h) {
                        (x, c, 0.)
                    } else if (120. ..180.).contains(&h) {
                        (0., c, x)
                    } else if (180. ..240.).contains(&h) {
                        (0., x, c)
                    } else if (240. ..300.).contains(&h) {
                        (x, 0., c)
                    } else if (300. ..360.).contains(&h) {
                        (c, 0., x)
                    } else {
                        unreachable!();
                    };

                    Ok(Value::Color(Color::new(
                        ((r + m) * 255.) as u8,
                        ((g + m) * 255.) as u8,
                        ((b + m) * 255.) as u8,
                    )))
                }),

                // floating point functions
                Function::new(&["sin"], &[NodeKind::Number], float_func!(sin)),
                Function::new(&["cos"], &[NodeKind::Number], float_func!(cos)),
                Function::new(&["tan"], &[NodeKind::Number], float_func!(tan)),
                Function::new(&["asin"], &[NodeKind::Number], float_func!(asin)),
                Function::new(&["acos"], &[NodeKind::Number], float_func!(acos)),
                Function::new(&["atan"], &[NodeKind::Number], float_func!(atan)),
                Function::new(&["abs"], &[NodeKind::Number], float_func!(abs)),
                Function::new(&["floor"], &[NodeKind::Number], float_func!(floor)),
                Function::new(&["ceil"], &[NodeKind::Number], float_func!(ceil)),
                Function::new(&["rad"], &[NodeKind::Number], float_func!(to_radians)),
                Function::new(&["deg"], &[NodeKind::Number], float_func!(to_degrees)),
                Function::new(&["random"], &[NodeKind::Number, NodeKind::Number], |v| {
                    Ok(Value::Number(rand::thread_rng().gen_range(
                        unwrap_variant!(v[0], Value::Number)
                            ..=unwrap_variant!(v[1], Value::Number),
                    )))
                }),
                Function::new(&["lerp"], &[NodeKind::Number, NodeKind::Number, NodeKind::Number], |v| {
                    Ok(Value::Number(Lerp::lerp(
                        unwrap_variant!(v[0], Value::Number),
                        unwrap_variant!(v[1], Value::Number),
                        unwrap_variant!(v[2], Value::Number),
                    )))
                }),

                // vector functions
                Function::new(&["normalize"], &[NodeKind::Vector], vector_func!(normalize, Vector)),
                Function::new(&["magnitude"], &[NodeKind::Vector], vector_func!(magnitude, Number)),
                Function::new(&["angle"], &[NodeKind::Vector], vector_func!(angle, Number,)),
                Function::new(&["lerp"], &[NodeKind::Vector, NodeKind::Vector, NodeKind::Number], |v| {
                    Ok(Value::Vector(
                        unwrap_variant!(v[0], Value::Vector).lerp(
                            unwrap_variant!(v[1], Value::Vector),
                            unwrap_variant!(v[2], Value::Number),
                        ))
                    )
                }),
            ];
        }

        let values = Value::from_nodes(self, scene, args)?;

        for func in FUNCTIONS
            .iter()
            .filter(|f| f.names.contains(&name.as_str()))
        {
            if let Some(r) = func.try_eval(values.clone()) {
                return r;
            }
        }

        let func = self
            .scope_stack
            .iter()
            .rev()
            .filter_map(|s| s.funcs.iter().find(|(n, _)| n == &&name).map(|(_, f)| f))
            .next()
            .cloned();

        if let Some(func) = func {
            // make a new scope, inject the parameter values, and run the body
            let new_scope = Scope {
                vars: func
                    .params
                    .clone()
                    .into_iter()
                    .zip(values.clone().into_iter())
                    .collect(),
                funcs: HashMap::new(),
            };

            self.scope_stack.push(new_scope);
            let ret = self.run_scope(scene, func.body.clone())?;
            self.scope_stack.pop();

            return Ok(ret);
        }

        return Err(InterpretError::UnknownFunction(name));
    }

    /// Deconstruct a list of arguments based on `NodeKind`s.
    fn deconstruct_args(
        &self,
        args: Vec<Value>,
        dest: &[ast::NodeKind],
    ) -> Result<Vec<Value>, InterpretError> {
        // first, confirm that both lengths are identical
        if args.len() != dest.len() {
            return Err(InterpretError::InvalidArgCount(dest.len(), args.len()));
        }

        let mut out = Vec::new();

        macro_rules! match_kinds {
            ($nk:ident, $v:ident, $o:ident, $($t:ident),+,) => {
                match $nk {
                    $(
                        ast::NodeKind::$t => {
                            if matches!($v, Value::$t(_)) {
                                $o.push($v)
                            }
                        }
                    ),+
                }
            }
        }

        // now iterate through each dest arg and compare with the arg we have
        for (node_kind, value) in dest.into_iter().zip(args.into_iter()) {
            match_kinds!(
                node_kind, value, out, Boolean, Color, Dictionary, Number, String, Vector,
            );
        }

        // we return `args` again if and only if they match the intended destination
        // now the receiver can forcibly unwrap each variant
        Ok(out)
    }

    /// Fetch an optional property out of a properties dictionary.
    fn optional_property(
        &mut self,
        scene: &mut Scene,
        properties: &mut HashMap<String, ast::Node>,
        name: &'static str,
        kind: ast::NodeKind,
    ) -> Result<Option<Value>, InterpretError> {
        macro_rules! match_kinds {
            ($nk:ident, $v:ident, $($t:ident),+,) => {
                match $nk {
                    $(
                        ast::NodeKind::$t => {
                            if matches!($v, Value::$t(_)) {
                                Ok(Some($v))
                            } else {
                                Err(InterpretError::InvalidCallArgs)
                            }
                        }
                    ),+
                }
            }
        }

        match properties.remove(name) {
            Some(node) => {
                let value = Value::from_node(self, scene, node)?;
                match_kinds!(kind, value, Boolean, Color, Dictionary, Number, String, Vector,)
            }
            None => Ok(None),
        }
    }

    /// Gets the value of a variable, somewhere along the stack, moving backwards.
    /// This clones the value of the variable.
    fn variable_value(&self, identifier: &String) -> Option<Value> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.vars.get(identifier) {
                return Some(value.to_owned());
            }
        }

        None
    }
}
