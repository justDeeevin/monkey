#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub literal: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::Display)]
pub enum TokenKind {
    #[strum(serialize = "Illegal token")]
    Illegal,
    #[strum(serialize = "Identifier")]
    Ident,
    #[strum(serialize = "Integer literal")]
    Int,

    #[strum(serialize = "Assignment")]
    Assign,
    Plus,
    Minus,
    Not,
    #[strum(serialize = "Multiply")]
    Mul,
    #[strum(serialize = "Divide")]
    Div,

    #[strum(serialize = "Less than")]
    LT,
    #[strum(serialize = "Greater than")]
    GT,
    #[strum(serialize = "Equal")]
    Eq,
    #[strum(serialize = "Not equal")]
    Neq,

    Comma,
    Semicolon,

    #[strum(serialize = "Left parenthesis")]
    LParen,
    #[strum(serialize = "Right parenthesis")]
    RParen,
    #[strum(serialize = "Left brace")]
    LBrace,
    #[strum(serialize = "Right brace")]
    RBrace,

    #[strum(serialize = "Function")]
    Fn,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

#[derive(Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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
