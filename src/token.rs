use std::{collections::HashMap, sync::LazyLock};

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub literal: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Illegal,
    Ident,
    Int,

    Assign,
    Plus,
    Minus,
    Not,
    Mul,
    Div,

    LT,
    GT,
    Eq,
    Neq,

    Comma,
    Semicolon,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Fn,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenKind>> = LazyLock::new(|| {
    HashMap::from([
        ("fn", TokenKind::Fn),
        ("let", TokenKind::Let),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("return", TokenKind::Return),
    ])
});
