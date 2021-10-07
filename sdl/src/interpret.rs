use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use raytracer::{material::{Material, Texture}, math::Vector3, object, scene::Scene};
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
}

/// A type in the SDL.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number(f64),
    String(String),
    Vector(Vector3),
}

macro_rules! ast_type {
    ($self:ident, $node:ident: $($in:path => $ast:path),+$(,)?) => {
        match $self {
            $(
                $in(val) => match $node {
                    $ast(fr) => {
                        *val = fr;
                    }
                    _ => return false,
                }
            )+
        }
    }
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
            Some(n) => return Err(InterpretError::PropertyTypeMismatch(stringify!($variant), format!("{:?}", n))),
            _ => None,
        }
    };
}

impl Type {
    /// Attempts to fill the type with an AST node.
    /// Returns false if the fill does not succeed,
    /// e.g. if an `ast::Node::String` tries to fill a `Type::Number`.
    pub fn fill(&mut self, node: ast::Node) -> bool {
        ast_type!(self, node:
            Type::Number => ast::Node::Number,
            Type::String => ast::Node::String,
            Type::Vector => ast::Node::Vector,
        );

        true
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
        let mut object_names = Vec::new();

        for node in root.into_iter() {
            match node {
                ast::Node::Object {
                    name,
                    ref properties,
                } => {
                    match name.as_str() {
                        "aabb" => {
                            let pos = required_property!(properties, "position", ast::Node::Vector);
                            let size = required_property!(properties, "size", ast::Node::Vector);
                            let material = Self::read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Aabb::new(*pos, *size, material)));
                        }
                        "sphere" => {
                            let pos = required_property!(properties, "position", ast::Node::Vector);
                            let radius = required_property!(properties, "radius", ast::Node::Number);
                            let material = Self::read_material(properties)?;

                            self.scene
                                .objects
                                .push(Box::new(object::Sphere::new(*pos, *radius, material)));
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

    fn read_material(properties: &HashMap<String, ast::Node>) -> Result<Material, InterpretError> {
        match properties.get("material") {
            Some(ast::Node::Dictionary(map)) => {
                let reflectiveness =
                    *optional_property!(map, "reflectiveness", ast::Node::Number).unwrap_or(&0.);
                let transparency =
                    *optional_property!(map, "transparency", ast::Node::Number).unwrap_or(&0.);
                let ior = *optional_property!(map, "ior", ast::Node::Number).unwrap_or(&1.3);

                Ok(Material {
                    reflectiveness,
                    transparency,
                    ior,
                    ..Default::default()
                })
            }
            Some(_) => Err(InterpretError::InvalidMaterials),
            _ => Ok(Material::default()),
        }
    }

    fn read_texture(node: ast::Node) -> Result<Texture, InterpretError> {
        todo!()
    }
}
