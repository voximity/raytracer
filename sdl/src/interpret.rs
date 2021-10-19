use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Read, Seek},
};

use image::{ImageBuffer, Rgb};
use rand::Rng;
use raytracer::{
    lighting,
    material::{Color, Material, Texture},
    math::Vector3,
    object,
    scene::Scene,
    skybox,
};
use thiserror::Error;

use crate::{
    ast::{self, AstError, AstParser},
    tokenize::{TokenizeError, Tokenizer},
};

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
}

macro_rules! optional_property {
    ($self:ident, $properties:ident, $name:literal, $k:ident) => {
        $self
            .optional_property(&mut $properties, $name, ast::NodeKind::$k)?
            .map(|v| unwrap_variant!(v, ast::Node::$k))
    };
}

macro_rules! required_property {
    ($self:ident, $properties:ident, $name:literal, $k:ident) => {
        match optional_property!($self, $properties, $name, $k) {
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

/// A scope is a wrapper around a dictionary from identifier
/// to AST node. The AST node is expected to be fully reduced.
struct Scope(HashMap<String, ast::Node>);

/// The image cache, that is, a map between file names and loaded images.
type ImageCache = HashMap<String, ImageBuffer<Rgb<u8>, Vec<u8>>>;

/// The interpreter is the general runtime for the SDL interpreter. It is responsible for storing
/// AST data, scene data, and interpreting the AST at scene construction time to develop the
/// scene.
pub struct Interpreter {
    root: ast::Node,
    scene: Scene,
    images: ImageCache,
    stack: Vec<Scope>,
    object_names: Vec<String>,
}

impl Interpreter {
    /// Create a new interpreter. This will instantiate a `Tokenizer` and tokenize the input, as well
    /// as instantiate an `AstParser` and parse the tokenized input. From there, the interpreter
    /// can operate on the root AST node.
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self, InterpretError> {
        Ok(Interpreter {
            root: AstParser::new(Tokenizer::new(reader).tokenize()?).parse_root()?,
            scene: Scene::default(),
            images: HashMap::new(),
            stack: vec![Scope(HashMap::new())],
            object_names: Vec::new(),
        })
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

        // match nodes that can be in the root node
        self.run_scope(root)?;

        Ok(self.scene)
    }

    fn run_scope(&mut self, nodes: Vec<ast::Node>) -> Result<(), InterpretError> {
        for node in nodes.into_iter() {
            match node {
                ast::Node::Assign { name, value } => {
                    let reduced = self.reduce_calls(vec![*value])?.into_iter().next().unwrap();
                    self.stack.last_mut().unwrap().0.insert(name, reduced);
                }
                ast::Node::For {
                    var,
                    from,
                    to,
                    body,
                } => {
                    let from = unwrap_variant!(
                        self.reduce_calls(vec![*from])?.into_iter().next().unwrap(),
                        ast::Node::Number
                    )
                    .floor() as i32;
                    let to = unwrap_variant!(
                        self.reduce_calls(vec![*to])?.into_iter().next().unwrap(),
                        ast::Node::Number
                    )
                    .floor() as i32;

                    for i in from..to {
                        // push a new scope to the stack with the index variable
                        self.stack.push(Scope(
                            vec![(var.clone(), ast::Node::Number(i as f64))]
                                .into_iter()
                                .collect(),
                        ));

                        // run the scope body
                        self.run_scope(body.clone())?;

                        // pop the scope from the stack
                        self.stack.pop();
                    }
                }
                ast::Node::Object {
                    name,
                    mut properties,
                } => {
                    match name.as_str() {
                        // one-time scene properties
                        "camera" => {
                            if self.object_names.iter().any(|n| n.as_str() == "camera") {
                                return Err(InterpretError::NonUniqueObject("camera"));
                            }

                            let vw = optional_property!(self, properties, "vw", Number)
                                .map(|f| f as i32);
                            let vh = optional_property!(self, properties, "vh", Number)
                                .map(|f| f as i32);
                            let origin = optional_property!(self, properties, "origin", Vector);
                            let yaw = optional_property!(self, properties, "yaw", Number);
                            let pitch = optional_property!(self, properties, "pitch", Number);
                            let fov = optional_property!(self, properties, "fov", Number);

                            if let Some(vw) = vw {
                                self.scene.camera.vw = vw;
                            }
                            if let Some(vh) = vh {
                                self.scene.camera.vh = vh;
                            }
                            if let Some(origin) = origin {
                                self.scene.camera.origin = origin;
                            }
                            if let Some(yaw) = yaw {
                                self.scene.camera.yaw = yaw;
                            }
                            if let Some(pitch) = pitch {
                                self.scene.camera.pitch = pitch;
                            }
                            if let Some(fov) = fov {
                                self.scene.camera.set_fov(fov);
                            }
                        }
                        "skybox" => {
                            if self.object_names.iter().any(|n| n.as_str() == "skybox") {
                                return Err(InterpretError::NonUniqueObject("skybox"));
                            }

                            let t = required_property!(self, properties, "type", String);

                            match t.as_str() {
                                "normal" => self.scene.skybox = Box::new(skybox::Normal),
                                "solid" => {
                                    let color =
                                        required_property!(self, properties, "color", Color);
                                    self.scene.skybox = Box::new(skybox::Solid(color));
                                }
                                "cubemap" => {
                                    let filename =
                                        required_property!(self, properties, "image", String);
                                    let img = match self.images.entry(filename) {
                                        Entry::Occupied(buf) => buf.get().clone(),
                                        Entry::Vacant(ent) => {
                                            let img = image::open(ent.key())?.into_rgb8();
                                            ent.insert(img.clone());
                                            img
                                        }
                                    };

                                    self.scene.skybox = Box::new(skybox::Cubemap::new(img));
                                }
                                _ => return Err(InterpretError::InvalidMaterials),
                            }
                        }

                        // objects
                        "aabb" => {
                            let pos = required_property!(self, properties, "position", Vector);
                            let size = required_property!(self, properties, "size", Vector);
                            let material = self.read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Aabb::new(pos, size, material)));
                        }
                        "mesh" => {
                            let obj = required_property!(self, properties, "obj", String);
                            let position = optional_property!(self, properties, "position", Vector)
                                .unwrap_or_else(|| Vector3::default());
                            let scale =
                                optional_property!(self, properties, "scale", Number).unwrap_or(1.);
                            let rotate_xyz =
                                optional_property!(self, properties, "rotate_xyz", Vector);
                            let rotate_zyx =
                                optional_property!(self, properties, "rotate_zyx", Vector);
                            let material = self.read_material(properties)?;

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
                            self.scene.objects.push(Box::new(mesh));
                        }
                        "plane" => {
                            let origin = required_property!(self, properties, "origin", Vector);
                            let normal = optional_property!(self, properties, "normal", Vector)
                                .unwrap_or_else(|| Vector3::new(0., 1., 0.))
                                .normalize();
                            let uv_wrap = optional_property!(self, properties, "uv_wrap", Number)
                                .map(|f| f as f32)
                                .unwrap_or(1.);
                            let material = self.read_material(properties)?;

                            self.scene.objects.push(Box::new(object::Plane {
                                origin,
                                normal,
                                material,
                                uv_wrap,
                            }));
                        }
                        "sphere" => {
                            let pos = required_property!(self, properties, "position", Vector);
                            let radius = required_property!(self, properties, "radius", Number);
                            let material = self.read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Sphere::new(pos, radius, material)));
                        }

                        // lights
                        "point_light" | "pointlight" => {
                            let default = lighting::Point::default();

                            let color = optional_property!(self, properties, "color", Color);
                            let intensity =
                                optional_property!(self, properties, "intensity", Number);
                            let specular_power =
                                optional_property!(self, properties, "specular_power", Number)
                                    .map(|f| f as i32);
                            let specular_strength =
                                optional_property!(self, properties, "specular_strength", Number);
                            let position = required_property!(self, properties, "position", Vector);
                            let max_distance =
                                optional_property!(self, properties, "max_distance", Number);

                            let light = lighting::Point {
                                color: color.unwrap_or(default.color),
                                intensity: intensity.unwrap_or(default.intensity),
                                specular_power: specular_power.unwrap_or(default.specular_power),
                                specular_strength: specular_strength
                                    .unwrap_or(default.specular_strength),
                                position,
                                max_distance: max_distance.unwrap_or(default.max_distance),
                            };

                            self.scene.lights.push(Box::new(light));
                        }
                        "sun" | "sun_light" | "sunlight" => {
                            let default = lighting::Sun::default();

                            let color = optional_property!(self, properties, "color", Color);
                            let intensity =
                                optional_property!(self, properties, "intensity", Number);
                            let specular_power =
                                optional_property!(self, properties, "specular_power", Number)
                                    .map(|f| f as i32);
                            let specular_strength =
                                optional_property!(self, properties, "specular_strength", Number);
                            let vector =
                                required_property!(self, properties, "vector", Vector).normalize();
                            let shadows = optional_property!(self, properties, "shadows", Boolean);
                            let shadow_coefficient =
                                optional_property!(self, properties, "shadow_coefficient", Number);

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

                            self.scene.lights.push(Box::new(light));
                        }
                        _ => return Err(InterpretError::UnknownObject(name.clone())),
                    }

                    self.object_names.push(name.clone());
                }
                _ => (),
            }
        }

        Ok(())
    }

    /// Read a material from a dictionary node.
    fn read_material(
        &mut self,
        mut properties: HashMap<String, ast::Node>,
    ) -> Result<Material, InterpretError> {
        match properties.remove("material") {
            Some(ast::Node::Dictionary(mut map)) => {
                let reflectiveness =
                    optional_property!(self, map, "reflectiveness", Number).unwrap_or(0.);
                let transparency =
                    optional_property!(self, map, "transparency", Number).unwrap_or(0.);
                let ior = optional_property!(self, map, "ior", Number).unwrap_or(1.3);

                let texture = match map.remove("texture") {
                    Some(node) => self.read_texture(node)?,
                    None => Texture::Solid(Color::white()),
                };

                Ok(Material {
                    texture,
                    reflectiveness,
                    transparency,
                    ior,
                })
            }
            Some(_) => Err(InterpretError::InvalidMaterials),
            _ => Ok(Material::default()),
        }
    }

    /// Read a texture from a call node.
    ///
    /// A texture can be `solid(color(r, g, b))` or `checkerboard(color(r, g, b), color(r, g, b))`.
    fn read_texture(&mut self, node: ast::Node) -> Result<Texture, InterpretError> {
        match node {
            ast::Node::Call(name, args) => match name.as_str() {
                "solid" => {
                    let args = self.deconstruct_args(args, &[ast::NodeKind::Color])?;
                    Ok(Texture::Solid(unwrap_variant!(args[0], ast::Node::Color)))
                }
                "checkerboard" => {
                    let args =
                        self.deconstruct_args(args, &[ast::NodeKind::Color, ast::NodeKind::Color])?;

                    Ok(Texture::Checkerboard(
                        unwrap_variant!(args[0], ast::Node::Color),
                        unwrap_variant!(args[1], ast::Node::Color),
                    ))
                }
                "image" => {
                    let args = self.deconstruct_args(args, &[ast::NodeKind::String])?;

                    match self.images.entry(unwrap_variant!(
                        args.into_iter().next().unwrap(),
                        ast::Node::String
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

    /// Take an array of arg nodes, and if any of them are `Call`s,
    /// perform the function execution and flatten.
    fn reduce_calls(&mut self, args: Vec<ast::Node>) -> Result<Vec<ast::Node>, InterpretError> {
        let mut out = vec![];

        for mut arg in args.into_iter() {
            if let ast::Node::Identifier(name) = arg {
                arg = match self.variable_value(&name) {
                    Some(value) => value,
                    None => return Err(InterpretError::UndefinedVariable(name)),
                };
            }

            while let ast::Node::Call(name, a) = arg {
                arg = self.call_func(name, a)?;
            }

            out.push(arg);
        }

        Ok(out)
    }

    /// Call a named function with some arguments.
    /// Its result is another node that can be used as other values.
    fn call_func(
        &mut self,
        name: String,
        mut args: Vec<ast::Node>,
    ) -> Result<ast::Node, InterpretError> {
        args = self.reduce_calls(args)?;

        // float operations
        macro_rules! op {
            ($op:tt) => {
                {
                    match args.get(0) {
                        Some(ast::Node::Number(_)) => {
                            let args = self.deconstruct_args(args, &[ast::NodeKind::Number, ast::NodeKind::Number])?;
                            Ok(ast::Node::Number(unwrap_variant!(args[0], ast::Node::Number) $op unwrap_variant!(args[1], ast::Node::Number)))
                        }
                        Some(ast::Node::Vector(_)) => {
                            let args = self.deconstruct_args(args, &[ast::NodeKind::Vector, ast::NodeKind::Vector])?;
                            Ok(ast::Node::Vector(unwrap_variant!(args[0], ast::Node::Vector) $op unwrap_variant!(args[1], ast::Node::Vector)))
                        }
                        _ => return Err(InterpretError::InvalidCallArgs),
                    }
                }
            }
        }

        macro_rules! float_func {
            ($n:ident) => {{
                let args = self.deconstruct_args(args, &[ast::NodeKind::Number])?;
                Ok(ast::Node::Number(
                    unwrap_variant!(args[0], ast::Node::Number).$n(),
                ))
            }};
        }

        macro_rules! vector_func {
            ($n:ident, $return:ident) => {{
                let args = self.deconstruct_args(args, &[ast::NodeKind::Vector])?;
                Ok(ast::Node::$return(
                    unwrap_variant!(args[0], ast::Node::Vector).$n(),
                ))
            }};
            ($n:ident, $return:ident,) => {{
                let args =
                    self.deconstruct_args(args, &[ast::NodeKind::Vector, ast::NodeKind::Vector])?;
                Ok(ast::Node::$return(
                    unwrap_variant!(args[0], ast::Node::Vector)
                        .$n(unwrap_variant!(args[1], ast::Node::Vector)),
                ))
            }};
        }

        match name.as_str() {
            // operations
            "add" => op!(+),
            "sub" => op!(-),
            "mul" => op!(*),
            "div" => op!(/),

            // constructors
            "color" => {
                let args = self.deconstruct_args(
                    args,
                    &[
                        ast::NodeKind::Number,
                        ast::NodeKind::Number,
                        ast::NodeKind::Number,
                    ],
                )?;

                Ok(ast::Node::Color(Color::new(
                    unwrap_variant!(args[0], ast::Node::Number) as u8,
                    unwrap_variant!(args[1], ast::Node::Number) as u8,
                    unwrap_variant!(args[2], ast::Node::Number) as u8,
                )))
            }
            "vec" => {
                let args = self.deconstruct_args(
                    args,
                    &[
                        ast::NodeKind::Number,
                        ast::NodeKind::Number,
                        ast::NodeKind::Number,
                    ],
                )?;
                Ok(ast::Node::Vector(Vector3::new(
                    unwrap_variant!(args[0], ast::Node::Number),
                    unwrap_variant!(args[1], ast::Node::Number),
                    unwrap_variant!(args[2], ast::Node::Number),
                )))
            }

            // floating point functions
            "sin" => float_func!(sin),
            "cos" => float_func!(cos),
            "tan" => float_func!(tan),
            "asin" => float_func!(asin),
            "acos" => float_func!(acos),
            "atan" => float_func!(atan),
            "abs" => float_func!(abs),
            "floor" => float_func!(floor),
            "ceil" => float_func!(ceil),
            "random" => {
                let args =
                    self.deconstruct_args(args, &[ast::NodeKind::Number, ast::NodeKind::Number])?;
                Ok(ast::Node::Number(rand::thread_rng().gen_range(
                    unwrap_variant!(args[0], ast::Node::Number)
                        ..=unwrap_variant!(args[1], ast::Node::Number),
                )))
            }

            // vector functions
            "normalize" => vector_func!(normalize, Vector),
            "magnitude" => vector_func!(magnitude, Number),
            "angle" => vector_func!(angle, Number,),

            // constants
            "pi" => Ok(ast::Node::Number(std::f64::consts::PI)),

            _ => Err(InterpretError::UnknownFunction(name)),
        }
    }

    /// Deconstruct a list of arguments based on `NodeKind`s.
    fn deconstruct_args(
        &mut self,
        args: Vec<ast::Node>,
        dest: &[ast::NodeKind],
    ) -> Result<Vec<ast::Node>, InterpretError> {
        // first, confirm that both lengths are identical
        if args.len() != dest.len() {
            return Err(InterpretError::InvalidArgCount(dest.len(), args.len()));
        }

        let mut out = Vec::new();

        // now iterate through each dest arg and compare with the arg we have
        for (node_kind, mut node) in dest.into_iter().zip(args.into_iter()) {
            // continuously call any functions until we have a non-call value
            while let ast::Node::Call(name, args) = node {
                node = self.call_func(name, args)?;
            }

            // now, we can compare node_kind with node
            match node_kind {
                ast::NodeKind::Boolean => {
                    if matches!(node, ast::Node::Boolean(_)) {
                        out.push(node)
                    }
                }
                ast::NodeKind::Color => {
                    if matches!(node, ast::Node::Color(_)) {
                        out.push(node)
                    }
                }
                ast::NodeKind::Dictionary => {
                    if matches!(node, ast::Node::Dictionary(_)) {
                        out.push(node)
                    }
                }
                ast::NodeKind::Number => {
                    if matches!(node, ast::Node::Number(_)) {
                        out.push(node)
                    }
                }
                ast::NodeKind::String => {
                    if matches!(node, ast::Node::String(_)) {
                        out.push(node)
                    }
                }
                ast::NodeKind::Vector => {
                    if matches!(node, ast::Node::Vector(_)) {
                        out.push(node)
                    }
                }
            }
        }

        // we return `args` again if and only if they match the intended destination
        // now the receiver can forcibly unwrap each variant
        Ok(out)
    }

    /// Fetch an optional property out of a properties dictionary.
    fn optional_property(
        &mut self,
        properties: &mut HashMap<String, ast::Node>,
        name: &'static str,
        kind: ast::NodeKind,
    ) -> Result<Option<ast::Node>, InterpretError> {
        match properties.remove(name) {
            Some(mut node) => {
                // continuously call any functions until we have a non-call value
                if let ast::Node::Identifier(name) = node {
                    node = match self.variable_value(&name) {
                        Some(value) => value,
                        None => return Err(InterpretError::UndefinedVariable(name)),
                    };
                }

                while let ast::Node::Call(name, args) = node {
                    node = self.call_func(name, args)?;
                }

                match kind {
                    ast::NodeKind::Boolean => {
                        if matches!(node, ast::Node::Boolean(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                    ast::NodeKind::Color => {
                        if matches!(node, ast::Node::Color(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                    ast::NodeKind::Dictionary => {
                        if matches!(node, ast::Node::Dictionary(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                    ast::NodeKind::Number => {
                        if matches!(node, ast::Node::Number(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                    ast::NodeKind::String => {
                        if matches!(node, ast::Node::String(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                    ast::NodeKind::Vector => {
                        if matches!(node, ast::Node::Vector(_)) {
                            Ok(Some(node))
                        } else {
                            return Err(InterpretError::InvalidCallArgs);
                        }
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Gets the value of a variable, somewhere along the stack, moving backwards.
    /// This clones the value of the variable.
    fn variable_value(&mut self, identifier: &String) -> Option<ast::Node> {
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.0.get(identifier) {
                return Some(value.to_owned());
            }
        }

        None
    }
}
