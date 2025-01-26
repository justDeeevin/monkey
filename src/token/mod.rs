use std::rc::Rc;

pub struct Token {
    pub kind: TokenKind,
    pub literal: Rc<str>,
    pub span: Span,
}

impl Token {
    pub fn new(literal: impl Into<String>, span: Span) -> Self {
        let literal: String = literal.into();
        Self {
            span,
            literal: literal.clone().into(),
            kind: literal.into(),
        }
    }
}

// This feels like a fine default. Maybe change.
type Int = i32;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Illegal,

    Ident(Rc<str>),
    Int(Int),

    Assign,
    Plus,

    Comma,
    Semicolon,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Fn,
    Let,
}

impl<T: Into<String>> From<T> for TokenKind {
    fn from(value: T) -> Self {
        let value: String = value.into();

        if let Ok(i) = value.parse::<Int>() {
            return Self::Int(i);
        }
        match value.as_str() {
            "fn" => Self::Fn,
            "let" => Self::Let,
            "=" => Self::Assign,
            "+" => Self::Plus,
            "," => Self::Comma,
            ";" => Self::Semicolon,
            "(" => Self::LParen,
            ")" => Self::RParen,
            "{" => Self::LBrace,
            "}" => Self::RBrace,
            _ => Self::Ident(value.into()),
        }
    }
}

/// A span of text in a string, including both the start and end locations.
pub struct Span {
    pub start: Location,
    pub end: Location,
}

/// The location of a particular character in a string. Line and column numbers begin at 1.
#[derive(Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
