use std::rc::Rc;

pub struct Token {
    pub kind: TokenKind,
    pub literal: Rc<str>,
}

impl Token {
    pub fn special(literal: char) -> Option<Self> {
        Some(Self {
            literal: literal.to_string().into(),
            kind: TokenKind::special(literal)?,
        })
    }

    pub fn word(literal: impl AsRef<str>) -> Option<Self> {
        Some(Self {
            literal: literal.as_ref().into(),
            kind: TokenKind::word(literal)?,
        })
    }

    pub fn illegal(literal: char) -> Self {
        Self {
            literal: literal.to_string().into(),
            kind: TokenKind::Illegal,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Illegal,

    Ident,
    Int,

    Assign,
    Plus,

    Comma,
    Semi,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Fn,
    Let,
}

impl TokenKind {
    pub fn special(literal: char) -> Option<Self> {
        match literal {
            '=' => Some(Self::Assign),
            '+' => Some(Self::Plus),
            ',' => Some(Self::Comma),
            ';' => Some(Self::Semi),
            '(' => Some(Self::LParen),
            ')' => Some(Self::RParen),
            '{' => Some(Self::LBrace),
            '}' => Some(Self::RBrace),
            _ => None,
        }
    }

    pub fn word(literal: impl AsRef<str>) -> Option<Self> {
        let literal = literal.as_ref();
        if Self::special(literal.chars().next()?).is_some() {
            return None;
        }
        match literal {
            "let" => Some(Self::Let),
            "fn" => Some(Self::Fn),
            _ => Some(Self::Ident),
        }
    }
}
