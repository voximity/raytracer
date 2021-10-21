use std::{collections::HashMap, iter::Peekable, vec::IntoIter};

use lazy_static::lazy_static;
use raytracer::material::Color;
use thiserror::Error;

use crate::tokenize::{Op, Sep, Token};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
}

lazy_static! {
    static ref OP_PRECEDENCE: HashMap<Op, u8> =
        vec![(Op::Mul, 2), (Op::Div, 2), (Op::Add, 1), (Op::Sub, 1),]
            .into_iter()
            .collect::<HashMap<_, _>>();
    static ref OP_ASSOCIATIVITY: HashMap<Op, Associativity> = vec![
        (Op::Add, Associativity::Left),
        (Op::Sub, Associativity::Left),
        (Op::Mul, Associativity::Left),
        (Op::Div, Associativity::Left),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();
}

/// An error while parsing to the AST.
#[derive(Debug, Error)]
pub enum AstError {
    #[error("expected '{0}', got '{1:?}'")]
    UnexpectedToken(String, Token),

    #[error("expected more tokens, got end")]
    UnexpectedEof,

    #[error("error parsing arithmetic expression")]
    ArithmeticError,

    #[error("too many closing parenthesis")]
    ArithmeticExcessCloseParensError(Option<Node>),
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
    Assign {
        name: String,
        declare: bool,
        value: Box<Node>,
    },

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
    Vector(Box<Node>, Box<Node>, Box<Node>),

    /// A color. This is really just a Call("color", vec![R, G, B]).
    Color(Color),

    /// A boolean.
    Boolean(bool),

    /// A scope terminator.
    ScopeTerminator,

    /// The addition of two nodes.
    Add(Box<Node>, Box<Node>),

    /// The subtraction of two nodes.
    Sub(Box<Node>, Box<Node>),

    /// The multiplication of two nodes.
    Mul(Box<Node>, Box<Node>),

    /// The division of two nodes.
    Div(Box<Node>, Box<Node>),
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
        let mut op_stack: Vec<Token> = vec![];
        let mut out_queue: Vec<Node> = vec![];

        let mut last_op = true;
        loop {
            let peeking = self.tokens.peek().ok_or(AstError::UnexpectedEof)?;
            match peeking {
                Token::Number(_) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    let n = match self.next()? {
                        Token::Number(n) => n,
                        _ => unreachable!(),
                    };

                    out_queue.push(Node::Number(n));
                }
                Token::String(_) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    let s = match self.next()? {
                        Token::String(s) => s,
                        _ => unreachable!(),
                    };

                    out_queue.push(Node::String(s));
                }
                Token::Boolean(_) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    let b = match self.next()? {
                        Token::Boolean(b) => b,
                        _ => unreachable!(),
                    };

                    out_queue.push(Node::Boolean(b));
                }
                Token::Sep(Sep::BraceOpen) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    self.next()?;

                    out_queue.push(self.read_dict()?);
                }
                Token::Op(Op::Lt) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    self.next()?;
                    out_queue.push(self.read_vector()?);
                }
                Token::Identifier(_) => {
                    if !last_op {
                        break;
                    } else {
                        last_op = false;
                    }

                    let ident = match self.next()? {
                        Token::Identifier(ident) => ident,
                        _ => unreachable!(),
                    };

                    if let Some(Token::Sep(Sep::ParensOpen)) = self.tokens.peek() {
                        self.next()?;

                        let mut v = Vec::new();

                        loop {
                            // if we hit the close token, stop the loop early
                            if let Some(t) = self.tokens.peek() {
                                if t == &Token::Sep(Sep::ParensClose) {
                                    self.next()?;
                                    break;
                                }
                            }

                            // continuously scan for more items
                            let (next_item, ct) = match self.parse_value() {
                                Ok(v) => (v, true),
                                Err(AstError::ArithmeticExcessCloseParensError(Some(v))) => {
                                    (v, false)
                                }
                                Err(e) => return Err(e),
                            };
                            v.push(next_item);

                            if !ct {
                                break;
                            }

                            // if we hit the close token, stop the loop, just like before
                            if let Some(t) = self.tokens.peek() {
                                if t == &Token::Sep(Sep::ParensClose) {
                                    self.next()?;
                                    break;
                                }
                            }

                            // if the next token wasn't the close token, expect the delimiter
                            self.read_sep(Sep::Comma)?;
                        }

                        out_queue.push(Node::Call(ident, v));
                    } else {
                        out_queue.push(Node::Identifier(ident));
                    }
                }
                Token::Op(op) => {

                    // this token is an operator, match it further
                    match op {
                        Op::Add | Op::Sub | Op::Mul | Op::Div => {
                            last_op = true;
        
                            let op = match self.next()? {
                                Token::Op(op) => op,
                                _ => unreachable!(),
                            };

                            loop {
                                let condition = match op_stack.last() {
                                    None => false,
                                    Some(Token::Sep(Sep::ParensOpen)) => false,
                                    Some(Token::Op(top)) => {
                                        let top_precedence = &OP_PRECEDENCE[top];
                                        let op_precedence = &OP_PRECEDENCE[&op];
                                        top_precedence > op_precedence
                                            || (top_precedence == op_precedence
                                                && OP_ASSOCIATIVITY[&op] == Associativity::Left)
                                    }
                                    _ => unimplemented!(),
                                };

                                if !condition {
                                    break;
                                }

                                let top_op_stack = op_stack.pop().unwrap();
                                match top_op_stack {
                                    Token::Op(Op::Add) => {
                                        let b = out_queue.pop().unwrap();
                                        let a = out_queue.pop().unwrap();
                                        out_queue.push(Node::Add(Box::new(a), Box::new(b)));
                                    }
                                    Token::Op(Op::Sub) => {
                                        let b = out_queue.pop().unwrap();
                                        let a = out_queue.pop().unwrap();
                                        out_queue.push(Node::Sub(Box::new(a), Box::new(b)));
                                    }
                                    Token::Op(Op::Mul) => {
                                        let b = out_queue.pop().unwrap();
                                        let a = out_queue.pop().unwrap();
                                        out_queue.push(Node::Mul(Box::new(a), Box::new(b)));
                                    }
                                    Token::Op(Op::Div) => {
                                        let b = out_queue.pop().unwrap();
                                        let a = out_queue.pop().unwrap();
                                        out_queue.push(Node::Div(Box::new(a), Box::new(b)));
                                    }
                                    _ => unimplemented!(),
                                }
                            }

                            op_stack.push(Token::Op(op));
                        }
                        _ => break,
                    }
                }
                Token::Sep(Sep::ParensOpen) => {
                    last_op = true;
                    self.next()?;
                    op_stack.push(Token::Sep(Sep::ParensOpen));
                }
                Token::Sep(Sep::ParensClose) => {
                    last_op = true;
                    self.next()?;

                    loop {
                        let condition = match op_stack.last() {
                            Some(Token::Sep(Sep::ParensOpen)) => false,
                            _ => true,
                        };

                        if !condition {
                            break;
                        }

                        // assert the operator stack is not empty
                        if op_stack.is_empty() {
                            return Err(AstError::ArithmeticExcessCloseParensError(
                                out_queue.into_iter().next(),
                            ));
                        }

                        // pop the operator from the operator stack into the output queue
                        let top_op_stack = op_stack.pop().unwrap();
                        match top_op_stack {
                            Token::Op(Op::Add) => {
                                let b = out_queue.pop().unwrap();
                                let a = out_queue.pop().unwrap();
                                out_queue.push(Node::Add(Box::new(a), Box::new(b)));
                            }
                            Token::Op(Op::Sub) => {
                                let b = out_queue.pop().unwrap();
                                let a = out_queue.pop().unwrap();
                                out_queue.push(Node::Sub(Box::new(a), Box::new(b)));
                            }
                            Token::Op(Op::Mul) => {
                                let b = out_queue.pop().unwrap();
                                let a = out_queue.pop().unwrap();
                                out_queue.push(Node::Mul(Box::new(a), Box::new(b)));
                            }
                            Token::Op(Op::Div) => {
                                let b = out_queue.pop().unwrap();
                                let a = out_queue.pop().unwrap();
                                out_queue.push(Node::Div(Box::new(a), Box::new(b)));
                            }
                            _ => (),
                        }
                    }

                    // assert there is a left parenthesis at the top of the operator stack
                    if let Some(Token::Sep(Sep::ParensOpen)) = op_stack.last() {
                        op_stack.pop();
                    } else {
                        return Err(AstError::ArithmeticError);
                    }
                }
                _ => {
                    break;
                }
            }
        }

        while !op_stack.is_empty() {
            if let Some(Token::Sep(Sep::ParensOpen)) = op_stack.last() {
                return Err(AstError::ArithmeticError);
            } else {
                let top_op_stack = op_stack.pop().unwrap();
                match top_op_stack {
                    Token::Op(Op::Add) => {
                        let b = out_queue.pop().unwrap();
                        let a = out_queue.pop().unwrap();
                        out_queue.push(Node::Add(Box::new(a), Box::new(b)));
                    }
                    Token::Op(Op::Sub) => {
                        let b = out_queue.pop().unwrap();
                        let a = out_queue.pop().unwrap();
                        out_queue.push(Node::Sub(Box::new(a), Box::new(b)));
                    }
                    Token::Op(Op::Mul) => {
                        let b = out_queue.pop().unwrap();
                        let a = out_queue.pop().unwrap();
                        out_queue.push(Node::Mul(Box::new(a), Box::new(b)));
                    }
                    Token::Op(Op::Div) => {
                        let b = out_queue.pop().unwrap();
                        let a = out_queue.pop().unwrap();
                        out_queue.push(Node::Div(Box::new(a), Box::new(b)));
                    }
                    _ => (),
                }
            }
        }

        Ok(out_queue.into_iter().next().unwrap())
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
            |s| s.read_expecting(Token::Sep(Sep::Comma)),
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
        let x = self.parse_value()?;
        self.read_sep(Sep::Comma)?;
        let y = self.parse_value()?;
        self.read_sep(Sep::Comma)?;
        let z = self.parse_value()?;
        self.read_expecting(Token::Op(Op::Gt))?;

        Ok(Node::Vector(Box::new(x), Box::new(y), Box::new(z)))
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
