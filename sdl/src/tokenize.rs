use std::{
    fmt::{self, Display, Formatter},
    io::{self, Read, Seek, SeekFrom},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("a generic IO error")]
    Io(#[from] io::Error),

    #[error("unexpected character")]
    UnexpectedCharacter(char),

    #[error("error parsing number")]
    NumberParseError,

    #[error("wrong character, expected another")]
    WrongCharacter { expected: char, got: char },
}

/// An operator.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Lt,
    Gt,
}

/// A separator.
#[derive(Debug, Clone, PartialEq)]
pub enum Sep {
    Comma,
    Colon,
    BraceOpen,
    BraceClose,
    ParensOpen,
    ParensClose,
    BracketOpen,
    BracketClose,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A separator.
    Sep(Sep),

    /// An operator.
    Op(Op),

    /// An identifier. Not escaped, quoted, etc.
    Identifier(String),

    /// A string. Any text that is surrounded by quotes. Supports quote escaping.
    String(String),

    /// A number. Decimals optional.
    Number(f64),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sep(Sep::Comma) => write!(f, ","),
            Self::Sep(Sep::Colon) => write!(f, ":"),
            Self::Sep(Sep::BraceOpen) => write!(f, "{{"),
            Self::Sep(Sep::BraceClose) => write!(f, "}}"),
            Self::Sep(Sep::ParensOpen) => write!(f, "("),
            Self::Sep(Sep::ParensClose) => write!(f, ")"),
            Self::Sep(Sep::BracketOpen) => write!(f, "["),
            Self::Sep(Sep::BracketClose) => write!(f, "]"),

            Self::Op(Op::Lt) => write!(f, "<"),
            Self::Op(Op::Gt) => write!(f, ">"),

            Self::Identifier(ident) => write!(f, "{}", ident),
            Self::String(str) => write!(f, "\"{}\"", str),
            Self::Number(num) => write!(f, "{}", num),
        }
    }
}

impl From<Op> for Token {
    fn from(op: Op) -> Self {
        Self::Op(op)
    }
}

impl From<Sep> for Token {
    fn from(sep: Sep) -> Self {
        Self::Sep(sep)
    }
}

pub struct Tokenizer<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> Tokenizer<R> {
    pub fn new(reader: R) -> Self {
        Tokenizer { reader }
    }

    /// Tokenize from the reader, converting to a `Result<Vec<Token>, TokenizeError>`.
    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizeError> {
        let mut tokens = vec![];

        while let Ok(c) = self.peek_next() {
            match c {
                // whitespace: ignore
                _ if c.is_whitespace() => self.skip()?,

                // alphabetical characters: identifier
                'A'..='Z' | 'a'..='z' => tokens.push(Token::Identifier(self.read_identifier()?)),

                // a quote: string
                '"' => tokens.push(Token::String(self.read_string()?)),

                // a number: number
                '0'..='9' | '.' | '-' => tokens.push(Token::Number(self.read_number()?)),

                '<' => {
                    tokens.push(Token::Op(Op::Lt));
                    self.skip()?;
                }
                '>' => {
                    tokens.push(Token::Op(Op::Gt));
                    self.skip()?;
                }
                ',' => {
                    tokens.push(Sep::Comma.into());
                    self.skip()?;
                }
                '{' => {
                    tokens.push(Sep::BraceOpen.into());
                    self.skip()?;
                }
                '}' => {
                    tokens.push(Sep::BraceClose.into());
                    self.skip()?;
                }
                '(' => {
                    tokens.push(Sep::ParensOpen.into());
                    self.skip()?;
                }
                ')' => {
                    tokens.push(Sep::ParensClose.into());
                    self.skip()?;
                }
                '[' => {
                    tokens.push(Sep::BracketOpen.into());
                    self.skip()?;
                }
                ']' => {
                    tokens.push(Sep::BracketClose.into());
                    self.skip()?;
                }
                ':' => {
                    tokens.push(Sep::Colon.into());
                    self.skip()?;
                }

                x => return Err(TokenizeError::UnexpectedCharacter(x)),
            }
        }

        Ok(tokens)
    }

    /// Read an identifier, which is just an alphanumeric bit of text.
    fn read_identifier(&mut self) -> Result<String, TokenizeError> {
        Ok(self
            .read_while(char::is_alphanumeric)?
            .into_iter()
            .collect())
    }

    /// Read a string, which is two quotations surrounding any amount of text.
    fn read_string(&mut self) -> Result<String, TokenizeError> {
        // skip the initial quotation
        self.skip()?;

        let mut escape = false;
        let mut string = String::new();

        while let Ok(c) = self.next() {
            match c {
                '"' if !escape => break,
                '\\' if !escape => escape = true,
                'n' if escape => string.push('\n'),
                c => {
                    string.push(c);
                    escape = false;
                }
            }
        }

        Ok(string)
    }

    /// Read a number, which is an f64. Decimal optional.
    fn read_number(&mut self) -> Result<f64, TokenizeError> {
        let negative = if let Ok('-') = self.peek_next() {
            self.next()?;
            true
        } else {
            false
        };
        let mut pre_dec = String::new(); // chars before the .
        let mut post_dec = String::new(); // chars after the .
        let mut dec_seen = false;

        loop {
            let c = match self.next() {
                Ok(c) => c,
                Err(_) => break,
            };

            match c {
                '.' if dec_seen => return Err(TokenizeError::UnexpectedCharacter('.')),
                '.' if !dec_seen => dec_seen = true,
                '0'..='9' => {
                    if dec_seen {
                        post_dec.push(c);
                    } else {
                        pre_dec.push(c);
                    }
                }
                _ => {
                    self.back()?;
                    break;
                }
            }
        }

        match (pre_dec.is_empty(), post_dec.is_empty()) {
            (true, true) => return Err(TokenizeError::NumberParseError),
            (true, false) => pre_dec.push('0'),
            (false, true) => post_dec.push('0'),
            _ => (),
        }

        format!("{}.{}", pre_dec, post_dec)
            .parse()
            .map(|n: f64| if negative { -n } else { n })
            .map_err(|_| TokenizeError::NumberParseError)
    }

    /// Read the next character in the reader, an `Option<char>`.
    fn next(&mut self) -> Result<char, io::Error> {
        let mut byte = [0u8];
        self.reader.read_exact(&mut byte).map(|_| byte[0] as char)
    }

    /// Reads the next character, expecting it to be one.
    fn next_expecting(&mut self, expecting: char) -> Result<(), TokenizeError> {
        let next = self.next()?;
        if next != expecting {
            Err(TokenizeError::WrongCharacter {
                expected: expecting,
                got: next,
            })
        } else {
            Ok(())
        }
    }

    /// Skips the next character in the reader.
    fn skip(&mut self) -> Result<(), io::Error> {
        self.reader.seek(SeekFrom::Current(1)).map(|_| ())
    }

    /// Skips until there is no more whitespace.
    fn skip_whitespace(&mut self) -> Result<(), io::Error> {
        self.read_while(char::is_whitespace).map(|_| ())
    }

    /// Goes back to the last character in the reader.
    fn back(&mut self) -> Result<(), io::Error> {
        self.reader.seek(SeekFrom::Current(-1)).map(|_| ())
    }

    /// Peeks ahead at the next character in the reader. This works by reading and then seeking back one on success.
    fn peek_next(&mut self) -> Result<char, io::Error> {
        self.next().map(|c| {
            self.reader.seek(SeekFrom::Current(-1)).unwrap();
            c
        })
    }

    /// Reads bytes until the predicate returns false.
    fn read_while<F>(&mut self, f: F) -> Result<Vec<char>, io::Error>
    where
        F: Fn(char) -> bool,
    {
        let mut v = vec![];

        loop {
            // get the next character and check if it passes the predicate
            let (ch, ok) = match self.next() {
                Ok(c) => (Some(c), f(c)),
                Err(_) => (None, false),
            };

            // if it does,
            if ok {
                // add it to the array
                v.push(ch.unwrap());
            } else {
                // otherwise seek back one and break out of the loop
                if let Some(_) = ch {
                    self.reader.seek(SeekFrom::Current(-1))?;
                }

                break;
            }
        }

        Ok(v)
    }
}
