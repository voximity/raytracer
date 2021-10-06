use std::{collections::HashMap, vec::IntoIter};

use raytracer::math::Vector3;
use thiserror::Error;

use crate::tokenize::{Op, Token};

#[derive(Debug, Error)]
pub enum AstError {
    #[error("expected '{0}', got '{1:?}'")]
    UnexpectedToken(String, Token),

    #[error("expected more tokens, got end")]
    UnexpectedEof,
}

#[derive(Debug, Clone)]
pub enum Node {
    /// The root AST node. In essence, the entire source file is this AST node.
    Root(Vec<Node>),

    /// A scene object. It has a name (its object identifier, e.g. 'sphere' or 'aabb'), and some properties.
    Object {
        name: String,
        properties: HashMap<String, Node>,
    },

    /// An identifier.
    Identifier(String),

    /// A string.
    String(String),

    /// A number.
    Number(f64),

    /// A vector.
    Vector(Vector3),
}

#[derive(Debug)]
pub struct AstParser {
    tokens: IntoIter<Token>,
}

impl AstParser {
    // for now, we will just assume that all identifiers on the root node are objects
    // after all, the only other thing they could be is keywords

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter(),
        }
    }

    pub fn parse_root(mut self) -> Result<Node, AstError> {
        let mut nodes = vec![];

        while let Ok(token) = self.next() {
            match token {
                Token::Identifier(identifier) => nodes.push(self.read_object(identifier)?),
                t => return Err(AstError::UnexpectedToken("a scene object".into(), t)),
            }
        }

        Ok(Node::Root(nodes))
    }

    fn parse_value(&mut self) -> Result<Node, AstError> {
        match self.next()? {
            Token::Identifier(ident) => Ok(Node::Identifier(ident)),
            Token::String(str) => Ok(Node::String(str)),
            Token::Number(num) => Ok(Node::Number(num)),
            Token::Op(Op::Lt) => Ok(self.read_vector()?),
            t => Err(AstError::UnexpectedToken("a value".into(), t)),
        }
    }

    fn read_object(&mut self, identifier: String) -> Result<Node, AstError> {
        // read the opening brace
        self.read_expecting(Token::Brace(true))?;

        // read as many key-values as possible
        let mut dict = vec![];

        loop {
            match self.next()? {
                Token::Identifier(key) => {
                    self.read_expecting(Token::Colon)?;
                    let value = self.parse_value()?;
                    dict.push((key, value));
                    self.read_expecting(Token::Comma)?;
                }
                Token::Brace(false) => break,
                t => {
                    return Err(AstError::UnexpectedToken(
                        "a key: value or closing brace".into(),
                        t,
                    ))
                }
            }
        }

        Ok(Node::Object {
            name: identifier,
            properties: dict.into_iter().collect(),
        })
    }

    fn read_vector(&mut self) -> Result<Node, AstError> {
        fn read_num(me: &mut AstParser) -> Result<f64, AstError> {
            match me.next()? {
                Token::Number(num) => Ok(num),
                t => return Err(AstError::UnexpectedToken("a number".into(), t)),
            }
        }

        let x = read_num(self)?;
        self.read_expecting(Token::Comma)?;
        let y = read_num(self)?;
        self.read_expecting(Token::Comma)?;
        let z = read_num(self)?;
        self.read_expecting(Token::Op(Op::Gt))?;

        Ok(Node::Vector(Vector3::new(x, y, z)))
    }

    fn read_expecting(&mut self, token: Token) -> Result<(), AstError> {
        let got = self.next()?;
        if got == token {
            Ok(())
        } else {
            Err(AstError::UnexpectedToken(format!("{}", token), got.clone()))
        }
    }

    fn next(&mut self) -> Result<Token, AstError> {
        self.tokens.next().ok_or(AstError::UnexpectedEof)
    }
}
