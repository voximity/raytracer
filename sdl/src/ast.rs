use std::{collections::HashMap, iter::Peekable, vec::IntoIter};

use raytracer::{material::Color, math::Vector3};
use thiserror::Error;

use crate::tokenize::{Op, Sep, Token};

/// An error while parsing to the AST.
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

    /// Assignment to a variable. The `declare` field dictates whether or not this will
    /// declare a new variable in the local scope, or update one in the next scope.
    Assign { name: String, declare: bool, value: Box<Node> },

    /// A for loop.
    For {
        var: String,
        from: Box<Node>,
        to: Box<Node>,
        body: Vec<Node>,
    },

    /// A dictionary. It acts as a map whose keys are identifiers and whose values are more AST nodes.
    Dictionary(HashMap<String, Node>),

    /// A function call, including a list of its parameters.
    Call(String, Vec<Node>),

    /// An identifier.
    Identifier(String),

    /// A string.
    String(String),

    /// A number.
    Number(f64),

    /// A vector.
    Vector(Vector3),

    /// A color. This is really just a Call("color", vec![R, G, B]).
    Color(Color),

    /// A boolean.
    Boolean(bool),

    /// A scope terminator.
    ScopeTerminator,
}

/// A kind of node *value*, rather than just any node. Used to allow functions to specify
/// their parameter types.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NodeKind {
    Dictionary,
    String,
    Number,
    Vector,
    Color,
    Boolean,
}

/// An AST parser, which takes in a list of tokens from the tokenizer and parses out to
/// a root AST node, which is composed of many more AST nodes.
#[derive(Debug)]
pub struct AstParser {
    tokens: Peekable<IntoIter<Token>>,
}

impl AstParser {
    // for now, we will just assume that all identifiers on the root node are objects
    // after all, the only other thing they could be is keywords

    /// Construct a new AST parser from a list of tokens from the tokenizer.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Parse the root node, that is, the global scope.
    pub fn parse_root(mut self) -> Result<Node, AstError> {
        let body = self.parse_scope()?;
        if let Some(Node::ScopeTerminator) = body.last() {
            Err(AstError::UnexpectedToken(
                "not a closing brace".into(),
                Token::Sep(Sep::BraceClose),
            ))
        } else {
            Ok(Node::Root(body))
        }
    }

    /// Parse as much of a scope as possible, returning all `Node`s.
    pub fn parse_scope(&mut self) -> Result<Vec<Node>, AstError> {
        let mut nodes = vec![];

        while let Ok(token) = self.next() {
            match token {
                Token::Identifier(identifier) => {
                    // once we read the identifier, we have to consider two cases:
                    // 1. the user is trying to create a scene object
                    // 2. the user is trying to set the value of a variable
                    // 3. we are using some sort of loop or condition
                    match identifier.as_str() {
                        "for" => {
                            let ident = match self.next()? {
                                Token::Identifier(i) => i,
                                t => {
                                    return Err(AstError::UnexpectedToken(
                                        "an identifier".into(),
                                        t,
                                    ))
                                }
                            };

                            self.read_expecting(Token::Identifier("in".into()))?;

                            let from = self.parse_value()?;
                            self.read_expecting(Token::Identifier("to".into()))?;
                            let to = self.parse_value()?;

                            self.read_expecting(Token::Sep(Sep::BraceOpen))?;
                            let body = self.parse_scope()?;
                            match body.last() {
                                Some(Node::ScopeTerminator) => (),
                                _ => return Err(AstError::UnexpectedEof),
                            }

                            nodes.push(Node::For {
                                var: ident,
                                from: Box::new(from),
                                to: Box::new(to),
                                body,
                            });

                            continue;
                        }
                        "let" => {
                            let ident = match self.next()? {
                                Token::Identifier(i) => i,
                                t => {
                                    return Err(AstError::UnexpectedToken(
                                        "an identifier".into(),
                                        t,
                                    ))
                                }
                            };

                            self.read_expecting(Token::Op(Op::Assign))?;

                            nodes.push(Node::Assign {
                                name: ident,
                                declare: true,
                                value: Box::new(self.parse_value()?),
                            });

                            continue;
                        }
                        _ => (),
                    }

                    match self.tokens.peek() {
                        Some(Token::Op(Op::Assign)) => {
                            self.tokens.next();
                            nodes.push(Node::Assign {
                                name: identifier,
                                declare: false,
                                value: Box::new(self.parse_value()?),
                            })
                        }
                        _ => nodes.push(self.read_object(identifier)?),
                    }
                }
                Token::Sep(Sep::BraceClose) => {
                    nodes.push(Node::ScopeTerminator);
                    return Ok(nodes);
                }
                t => {
                    return Err(AstError::UnexpectedToken(
                        "something usable in a scope, or a scope terminator".into(),
                        t,
                    ))
                }
            }
        }

        Ok(nodes)
    }

    /// Parse any "value": effectively an expression that has some value.
    fn parse_value(&mut self) -> Result<Node, AstError> {
        match self.next()? {
            Token::Identifier(ident) => {
                if let Some(Token::Sep(Sep::ParensOpen)) = self.tokens.peek() {
                    self.tokens.next();

                    let params = self.read_list(
                        Self::parse_value,
                        |s| s.read_sep(Sep::Comma),
                        Token::Sep(Sep::ParensClose),
                    )?;

                    Ok(Node::Call(ident, params))
                } else {
                    Ok(Node::Identifier(ident))
                }
            }
            Token::Boolean(bool) => Ok(Node::Boolean(bool)),
            Token::String(str) => Ok(Node::String(str)),
            Token::Number(num) => Ok(Node::Number(num)),
            Token::Op(Op::Lt) => Ok(self.read_vector()?),
            Token::Sep(Sep::BraceOpen) => Ok(self.read_dict()?),
            t => Err(AstError::UnexpectedToken("a value".into(), t)),
        }
    }

    /// Read a scene object.
    ///
    /// An example scene object:
    /// ```
    /// sphere {
    ///     position: <1, 2, 3>,
    ///     radius: 4,
    ///     material: {
    ///         texture: solid(color(random(0, 1), random(0, 1), random(0, 1))),
    ///         reflectiveness: 0.4,
    ///     },
    /// }
    /// ```
    fn read_object(&mut self, identifier: String) -> Result<Node, AstError> {
        // the identifier has already been read

        // read the opening brace
        self.read_sep(Sep::BraceOpen)?;

        // read the properties
        let props = self.read_dict()?;

        Ok(Node::Object {
            name: identifier,
            properties: match props {
                Node::Dictionary(dict) => dict,
                _ => unreachable!(),
            },
        })
    }

    /// Read a dictionary.
    ///
    /// An example dictionary:
    /// ```
    /// {
    ///     key: value,
    ///     key: "value",
    ///     someOtherKey: value(),
    ///     nested: {
    ///         a: "you can nest dictionaries!"
    ///     }
    /// }
    fn read_dict(&mut self) -> Result<Node, AstError> {
        // we assume the opening brace has already been read

        let dict = self.read_list(
            |s| {
                let key = match s.next()? {
                    Token::Identifier(ident) => ident,
                    t => {
                        return Err(AstError::UnexpectedToken(
                            "a key-value or closing brace".into(),
                            t,
                        ))
                    }
                };

                if let Some(Token::Sep(Sep::Colon)) = s.tokens.peek() {
                    s.next()?;
                    Ok((key, s.parse_value()?))
                } else {
                    Ok((key.clone(), Node::Identifier(key)))
                }
            },
            |s| {
                if let Some(Token::Sep(Sep::Comma)) = s.tokens.peek() {
                    s.next()?;
                }

                Ok(())
            },
            Token::Sep(Sep::BraceClose),
        )?;

        Ok(Node::Dictionary(dict.into_iter().collect()))
    }

    /// Read a vector.
    ///
    /// An example vector:
    /// ```
    /// <1.1, 2.4, 6.7>
    /// ```
    fn read_vector(&mut self) -> Result<Node, AstError> {
        fn read_num(me: &mut AstParser) -> Result<f64, AstError> {
            match me.next()? {
                Token::Number(num) => Ok(num),
                t => return Err(AstError::UnexpectedToken("a number".into(), t)),
            }
        }

        let x = read_num(self)?;
        self.read_sep(Sep::Comma)?;
        let y = read_num(self)?;
        self.read_sep(Sep::Comma)?;
        let z = read_num(self)?;
        self.read_expecting(Token::Op(Op::Gt))?;

        Ok(Node::Vector(Vector3::new(x, y, z)))
    }

    /// Read from the token stream, expecting a token.
    /// Errors with `AstError::UnexpectedToken` if any other token is received.
    fn read_expecting(&mut self, token: Token) -> Result<(), AstError> {
        let got = self.next()?;
        if got == token {
            Ok(())
        } else {
            Err(AstError::UnexpectedToken(format!("{}", token), got.clone()))
        }
    }

    /// Read from the token stream, expecting a certain `Sep`arator.
    fn read_sep(&mut self, sep: Sep) -> Result<(), AstError> {
        self.read_expecting(Token::Sep(sep))
    }

    /// Read a list of parsable things from closure `item: I`, delimited by closure `delimiter: D`,
    /// or closed by list closer token `close_token`.
    ///
    /// For example, a list of values can simply be read with `read_list(Self::read_value, |s| s.read_sep(Sep::Comma), Token::Sep(Sep::ParensClose))`.
    fn read_list<I, D, T>(
        &mut self,
        item: I,
        delimiter: D,
        close_token: Token,
    ) -> Result<Vec<T>, AstError>
    where
        I: Fn(&mut Self) -> Result<T, AstError>,
        D: Fn(&mut Self) -> Result<(), AstError>,
    {
        let mut v = Vec::new();

        loop {
            // if we hit the close token, stop the loop early
            if let Some(t) = self.tokens.peek() {
                if t == &close_token {
                    self.next()?;
                    break;
                }
            }

            // continuously scan for more items
            v.push(item(self)?);

            // if we hit the close token, stop the loop, just like before
            if let Some(t) = self.tokens.peek() {
                if t == &close_token {
                    self.next()?;
                    break;
                }
            }

            // if the next token wasn't the close token, expect the delimiter
            delimiter(self)?;
        }

        Ok(v)
    }

    /// Advance the token stream, or error with `AstError::UnexpectedEof`.
    fn next(&mut self) -> Result<Token, AstError> {
        self.tokens.next().ok_or(AstError::UnexpectedEof)
    }
}
