use std::rc::Rc;

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
