use crate::{
    ast::Expression,
    parser::{self, Parser},
};

#[derive(Debug, Clone)]
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

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Illegal => "Illegal token",
            Self::Ident => "Identifier",
            Self::Int => "Integer literal",

            Self::Assign => "`=`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Not => "`!`",
            Self::Mul => "`*`",
            Self::Div => "`/`",

            Self::LT => "`<`",
            Self::GT => "`>`",
            Self::Eq => "`==`",
            Self::Neq => "`!=`",

            Self::Comma => "`,`",
            Self::Semicolon => "`;`",

            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",

            Self::Fn => "`fn`",
            Self::Let => "`let`",
            Self::True => "`true`",
            Self::False => "`false`",
            Self::If => "`if`",
            Self::Else => "`else`",
            Self::Return => "`return`",
        };

        str.fmt(f)
    }
}

impl TokenKind {
    pub const fn prefix_parse<'a>(
        self,
    ) -> Option<fn(&mut Parser<'a>, Token<'a>) -> parser::Result<'a, Expression<'a>>> {
        match self {
            Self::Ident => Some(Parser::parse_identifier),
            Self::Int => Some(Parser::parse_integer),
            Self::Not | Self::Minus => Some(Parser::parse_prefix),
            Self::True | Self::False => Some(Parser::parse_boolean),
            Self::LParen => Some(Parser::parse_grouped_expression),
            Self::If => Some(Parser::parse_if),
            Self::Fn => Some(Parser::parse_function),
            _ => None,
        }
    }

    pub const fn is_infix(self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Mul
                | Self::Div
                | Self::Eq
                | Self::Neq
                | Self::LT
                | Self::GT
        )
    }
}

#[derive(Default, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl<T: std::ops::RangeBounds<usize>> From<T> for Span {
    fn from(value: T) -> Self {
        let start = match value.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(start) => *start + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match value.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => usize::MAX,
        };

        Self { start, end }
    }
}

impl ariadne::Span for Span {
    type SourceId = &'static str;

    fn source(&self) -> &Self::SourceId {
        &"input"
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
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
