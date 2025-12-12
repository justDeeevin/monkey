use crate::token::Token;

pub trait Node {
    fn literal(&self) -> &str;
}

pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl Node for Program<'_> {
    fn literal(&self) -> &str {
        self.statements
            .first()
            .map(Node::literal)
            .unwrap_or_default()
    }
}

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(StatementKind))]
pub enum Statement<'a> {
    Let(Let<'a>),
    Return(Return<'a>),
}

impl Node for Statement<'_> {
    fn literal(&self) -> &str {
        match self {
            Self::Let(l) => l.literal(),
            Self::Return(r) => r.literal(),
        }
    }
}

pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    Temp,
}

impl Node for Expression<'_> {
    fn literal(&self) -> &str {
        match self {
            Self::Identifier(identifier) => identifier.literal(),
            Self::Temp => panic!("temp expression!"),
        }
    }
}

pub struct Let<'a> {
    pub token: Token<'a>,
    pub name: Identifier<'a>,
    pub value: Expression<'a>,
}

impl Node for Let<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

pub struct Identifier<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl Node for Identifier<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

pub struct Return<'a> {
    pub token: Token<'a>,
    pub value: Expression<'a>,
}

impl Node for Return<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}
