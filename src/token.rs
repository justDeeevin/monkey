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

pub fn lookup_keyword(input: &str) -> Option<TokenKind> {
    match input {
        "fn" => Some(TokenKind::Fn),
        "let" => Some(TokenKind::Let),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "return" => Some(TokenKind::Return),
        _ => None,
    }
}
