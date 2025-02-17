use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: Rc<str>,
}

impl Token {
    pub fn special(literal: char, peek: char) -> Self {
        let kind = TokenKind::special(literal, peek);
        let literal = if kind.is_double() {
            format!("{literal}{peek}").into()
        } else {
            literal.to_string().into()
        };
        Self { literal, kind }
    }

    pub fn word(literal: impl AsRef<str>) -> Self {
        Self {
            literal: literal.as_ref().into(),
            kind: TokenKind::word(literal),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.kind.is_keyword() {
            write!(f, "keyword \"{}\"", self.kind)
        } else if self.kind.is_operator() {
            write!(f, "operator \"{}\"", self.kind)
        } else if self.kind.is_literal() {
            write!(f, "literal \"{}\"", self.literal)
        } else if self.kind == TokenKind::Ident {
            write!(f, "identifier \"{}\"", self.literal)
        } else {
            self.kind.fmt(f)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Illegal,

    Ident,
    Int,
    True,
    False,

    Assign,
    Plus,
    Minus,
    Not,
    Mult,
    Div,

    Equal,
    NotEqual,
    Less,
    Greater,

    Comma,
    Semi,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Fn,
    Let,
    If,
    Else,
    Return,
}

impl TokenKind {
    pub fn special(literal: char, peek: char) -> Self {
        match literal {
            '=' => match peek {
                '=' => Self::Equal,
                _ => Self::Assign,
            },
            '+' => Self::Plus,
            ',' => Self::Comma,
            ';' => Self::Semi,
            '(' => Self::LParen,
            ')' => Self::RParen,
            '{' => Self::LBrace,
            '}' => Self::RBrace,
            '-' => Self::Minus,
            '!' => match peek {
                '=' => Self::NotEqual,
                _ => Self::Not,
            },
            '*' => Self::Mult,
            '/' => Self::Div,
            '<' => Self::Less,
            '>' => Self::Greater,
            _ => Self::Illegal,
        }
    }

    pub fn is_double(&self) -> bool {
        matches!(self, Self::Equal | Self::NotEqual)
    }

    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Let | Self::Fn | Self::If | Self::Else | Self::Return
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Self::True | Self::False | Self::Int)
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Mult
                | Self::Div
                | Self::Assign
                | Self::Equal
                | Self::NotEqual
                | Self::Less
                | Self::Greater
                | Self::Not
        )
    }

    pub fn word(literal: impl AsRef<str>) -> Self {
        let literal = literal.as_ref();
        match literal {
            "let" => Self::Let,
            "fn" => Self::Fn,
            "if" => Self::If,
            "else" => Self::Else,
            "return" => Self::Return,
            "true" => Self::True,
            "false" => Self::False,
            _ => Self::Ident,
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Let => "let",
            Self::Fn => "fn",
            Self::If => "if",
            Self::Else => "else",
            Self::Return => "return",

            Self::True | Self::False => "boolean literal",
            Self::Int => "integer",

            Self::Ident => "identifier",

            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "less than",
            Self::Greater => "greater than",
            Self::Not => "!",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Assign => "assign",

            Self::LParen => "open parenthesis",
            Self::RParen => "closing parenthesis",
            Self::LBrace => "open brace",
            Self::RBrace => "closing brace",

            Self::Comma => "comma",

            Self::Illegal => "illegal",
            Self::Semi => "semicolon",
        };
        write!(f, "{name}")
    }
}
