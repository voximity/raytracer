use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use raytracer::{
    lighting,
    material::{Color, Material, Texture},
    math::Vector3,
    object,
    scene::Scene,
};
use thiserror::Error;

use crate::{
    ast::{self, AstError, AstParser},
    tokenize::{TokenizeError, Tokenizer},
};

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

    #[error("property was expected to be {0}, is actually {1}")]
    PropertyTypeMismatch(&'static str, String),

    #[error("invalid args to function call")]
    InvalidCallArgs,
}

macro_rules! required_property {
    ($properties:ident, $name:literal, $variant:path) => {
        match $properties.get($name) {
            Some($variant(value)) => value,
            _ => return Err(InterpretError::RequiredPropertyMissing($name)),
        }
    };
}

macro_rules! optional_property {
    ($properties:ident, $name:literal, $variant:path) => {
        match $properties.get($name) {
            Some($variant(value)) => Some(value),
            Some(n) => {
                return Err(InterpretError::PropertyTypeMismatch(
                    stringify!($variant),
                    format!("{:?}", n),
                ))
            }
            _ => None,
        }
    };
}

macro_rules! deconstruct_args {
    ($args:ident, $($v:path: $a:ident),+$(,)?) => {
        match &$args[..] {
            &[$($v($a)),+] => ($($a),+),
            _ => return Err(InterpretError::InvalidCallArgs),
        }
    }
}

/// The interpreter is the general runtime for the SDL interpreter. It is responsible for storing
/// AST data, scene data, and interpreting the AST at scene construction time to develop the
/// scene.
pub struct Interpreter {
    root: ast::Node,
    scene: Scene,
}

impl Interpreter {
    /// Create a new interpreter. This will instantiate a `Tokenizer` and tokenize the input, as well
    /// as instantiate an `AstParser` and parse the tokenized input. From there, the interpreter
    /// can operate on the root AST node.
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self, InterpretError> {
        Ok(Interpreter {
            root: AstParser::new(Tokenizer::new(reader).tokenize()?).parse_root()?,
            scene: Scene::default(),
        })
    }

    /// Start execution of the interpreter.
    pub fn run(mut self) -> Result<Scene, InterpretError> {
        let root = match self.root {
            ast::Node::Root(root) => root,
            _ => unreachable!(),
        };

        // check for duplicate camera objects
        let mut object_names: Vec<String> = Vec::new();

        for node in root.into_iter() {
            match node {
                ast::Node::Object {
                    name,
                    ref properties,
                } => {
                    match name.as_str() {
                        // one-time scene properties
                        "camera" => {
                            if object_names.iter().any(|n| n.as_str() == "camera") {
                                return Err(InterpretError::NonUniqueObject("camera"));
                            }

                            let vw = optional_property!(properties, "vw", ast::Node::Number)
                                .map(|&f| f as i32);
                            let vh = optional_property!(properties, "vh", ast::Node::Number)
                                .map(|&f| f as i32);
                            let origin =
                                optional_property!(properties, "origin", ast::Node::Vector)
                                    .copied();
                            let yaw =
                                optional_property!(properties, "yaw", ast::Node::Number).copied();
                            let pitch =
                                optional_property!(properties, "pitch", ast::Node::Number).copied();
                            let fov =
                                optional_property!(properties, "fov", ast::Node::Number).copied();

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

                        // objects
                        "aabb" => {
                            let pos = required_property!(properties, "position", ast::Node::Vector);
                            let size = required_property!(properties, "size", ast::Node::Vector);
                            let material = Self::read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Aabb::new(*pos, *size, material)));
                        }
                        "plane" => {
                            let origin =
                                *required_property!(properties, "origin", ast::Node::Vector);
                            let normal =
                                optional_property!(properties, "normal", ast::Node::Vector)
                                    .map(|&v| v)
                                    .unwrap_or_else(|| Vector3::new(0., 1., 0.))
                                    .normalize();
                            let material = Self::read_material(properties)?;
                            let uv_wrap =
                                optional_property!(properties, "uv_wrap", ast::Node::Number)
                                    .map(|&f| f as f32)
                                    .unwrap_or(1.);

                            self.scene.objects.push(Box::new(object::Plane {
                                origin,
                                normal,
                                material,
                                uv_wrap,
                            }));
                        }
                        "sphere" => {
                            let pos = required_property!(properties, "position", ast::Node::Vector);
                            let radius =
                                required_property!(properties, "radius", ast::Node::Number);
                            let material = Self::read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Sphere::new(*pos, *radius, material)));
                        }

                        // lights
                        "point" => {
                            let default = lighting::Point::default();

                            let color =
                                optional_property!(properties, "color", ast::Node::Color).copied();
                            let intensity =
                                optional_property!(properties, "intensity", ast::Node::Number)
                                    .copied();
                            let specular_power =
                                optional_property!(properties, "specular_power", ast::Node::Number)
                                    .map(|&f| f as i32);
                            let specular_strength = optional_property!(
                                properties,
                                "specular_strength",
                                ast::Node::Number
                            )
                            .copied();
                            let position =
                                *required_property!(properties, "position", ast::Node::Vector);
                            let max_distance =
                                optional_property!(properties, "max_distance", ast::Node::Number)
                                    .copied();

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
                        "sun" => {
                            let default = lighting::Sun::default();

                            let color =
                                optional_property!(properties, "color", ast::Node::Color).copied();
                            let intensity =
                                optional_property!(properties, "intensity", ast::Node::Number)
                                    .copied();
                            let specular_power =
                                optional_property!(properties, "specular_power", ast::Node::Number)
                                    .map(|&f| f as i32);
                            let specular_strength = optional_property!(
                                properties,
                                "specular_strength",
                                ast::Node::Number
                            )
                            .copied();
                            let vector =
                                required_property!(properties, "vector", ast::Node::Vector)
                                    .normalize();
                            let shadows =
                                optional_property!(properties, "shadows", ast::Node::Boolean)
                                    .copied();
                            let shadow_coefficient = optional_property!(
                                properties,
                                "shadow_coefficient",
                                ast::Node::Number
                            )
                            .copied();

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
                        _ => return Err(InterpretError::UnknownObject(name)),
                    }

                    object_names.push(name);
                }
                _ => (),
            }
        }

        Ok(self.scene)
    }

    /// Read a material from a dictionary node.
    fn read_material(properties: &HashMap<String, ast::Node>) -> Result<Material, InterpretError> {
        match properties.get("material") {
            Some(ast::Node::Dictionary(map)) => {
                let reflectiveness =
                    *optional_property!(map, "reflectiveness", ast::Node::Number).unwrap_or(&0.);
                let transparency =
                    *optional_property!(map, "transparency", ast::Node::Number).unwrap_or(&0.);
                let ior = *optional_property!(map, "ior", ast::Node::Number).unwrap_or(&1.3);

                let texture = match map.get("texture") {
                    Some(node) => Self::read_texture(node)?,
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
    fn read_texture(node: &ast::Node) -> Result<Texture, InterpretError> {
        match node {
            ast::Node::Call(name, args) => match name.as_str() {
                "solid" => {
                    let c = deconstruct_args!(args, ast::Node::Color: c);
                    Ok(Texture::Solid(c))
                }
                "checkerboard" => {
                    let (a, b) = deconstruct_args!(args, ast::Node::Color: a, ast::Node::Color: b);

                    Ok(Texture::Checkerboard(a, b))
                }
                _ => Err(InterpretError::InvalidCallArgs),
            },
            _ => Err(InterpretError::InvalidCallArgs),
        }
    }
}
