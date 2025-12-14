use crate::token::{Span, Token};
use std::fmt::{Debug, Display};

#[cfg(test)]
mod test;

pub trait Node: Display + Debug {
    fn span(&self) -> Span;
}

#[derive(Debug)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{statement}")?;
        }

        Ok(())
    }
}

impl Node for Program<'_> {
    fn span(&self) -> Span {
        let start = self
            .statements
            .first()
            .map(|s| s.span().start)
            .unwrap_or_default();

        let end = self
            .statements
            .last()
            .map(|s| s.span().end)
            .unwrap_or_default();

        Span { start, end }
    }
}

#[derive(strum::EnumDiscriminants, Debug)]
#[strum_discriminants(name(StatementKind))]
pub enum Statement<'a> {
    Let(Let<'a>),
    Return(Return<'a>),
    Expression(Expression<'a>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Let(l) => Display::fmt(l, f),
            Self::Return(r) => Display::fmt(r, f),
            Self::Expression(e) => Display::fmt(e, f),
        }?;

        write!(f, ";")
    }
}

impl Node for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Let(l) => l.span(),
            Self::Return(r) => r.span(),
            Self::Expression(e) => e.span(),
        }
    }
}

#[derive(strum::EnumDiscriminants, Debug)]
#[strum_discriminants(name(ExpressionKind))]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    Integer(Integer<'a>),
    Prefix(Prefix<'a>),
    Infix(Infix<'a>),
    Boolean(Boolean<'a>),
    If(If<'a>),
    Function(Function<'a>),
    Call(Call<'a>),
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        match self {
            Self::Identifier(i) => Display::fmt(i, f),
            Self::Integer(i) => Display::fmt(i, f),
            Self::Prefix(p) => Display::fmt(p, f),
            Self::Infix(i) => Display::fmt(i, f),
            Self::Boolean(b) => Display::fmt(b, f),
            Self::If(i) => Display::fmt(i, f),
            Self::Function(fun) => Display::fmt(fun, f),
            Self::Call(call) => Display::fmt(call, f),
        }?;
        write!(f, ")")
    }
}

impl Node for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span(),
            Self::Integer(integer) => integer.span(),
            Self::Prefix(prefix) => prefix.span(),
            Self::Infix(infix) => infix.span(),
            Self::Boolean(boolean) => boolean.span(),
            Self::If(i) => i.span(),
            Self::Function(f) => f.span(),
            Self::Call(call) => call.span(),
        }
    }
}

#[derive(Debug)]
pub struct Call<'a> {
    pub function: Box<Expression<'a>>,
    pub arguments: Vec<Expression<'a>>,
    pub close: Token<'a>,
}

impl Display for Call<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.function, f)?;
        write!(f, "(")?;
        if !self.arguments.is_empty() {
            for argument in self.arguments.iter().take(self.arguments.len() - 1) {
                write!(f, "{}, ", argument)?;
            }
            write!(f, "{}", self.arguments.last().unwrap())?;
        }
        write!(f, ")")
    }
}

impl Node for Call<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.function.span().start,
            end: self.close.span.end,
        }
    }
}

#[derive(Debug)]
pub struct Function<'a> {
    pub token: Token<'a>,
    pub parameters: Vec<Identifier<'a>>,
    pub body: BlockStatement<'a>,
}

impl Display for Function<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn(")?;
        if !self.parameters.is_empty() {
            for parameter in self.parameters.iter().take(self.parameters.len() - 1) {
                write!(f, "{}, ", parameter)?;
            }
            write!(f, "{}", self.parameters.last().unwrap())?;
        }
        write!(f, ") ")?;
        Display::fmt(&self.body, f)
    }
}

impl Node for Function<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.token.span.start,
            end: self.body.span().end,
        }
    }
}

#[derive(Debug)]
pub struct If<'a> {
    pub token: Token<'a>,
    pub condition: Box<Expression<'a>>,
    pub consequence: BlockStatement<'a>,
    pub alternative: Option<BlockStatement<'a>>,
}

#[derive(Debug)]
pub struct BlockStatement<'a> {
    pub open: Token<'a>,
    pub statements: Vec<Statement<'a>>,
    pub close: Token<'a>,
}

impl Display for BlockStatement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.statements {
            writeln!(f, "\t{statement}")?;
        }
        write!(f, "}}")
    }
}

impl Node for BlockStatement<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.open.span.start,
            end: self.close.span.end,
        }
    }
}

impl Display for If<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence,)?;

        if let Some(alternative) = &self.alternative {
            write!(f, " else {}", alternative)?;
        }

        Ok(())
    }
}

impl Node for If<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.token.span.start,
            end: self
                .alternative
                .as_ref()
                .map(|s| s.span().end)
                .unwrap_or(self.consequence.span().end),
        }
    }
}

#[derive(Debug)]
pub struct Boolean<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl Display for Boolean<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Node for Boolean<'_> {
    fn span(&self) -> Span {
        self.token.span
    }
}

#[derive(Debug)]
pub struct Infix<'a> {
    pub token: Token<'a>,
    pub left: Box<Expression<'a>>,
    pub operator: InfixOperator,
    pub right: Box<Expression<'a>>,
}

#[derive(strum::Display, PartialEq, Eq, Debug)]
pub enum InfixOperator {
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "-")]
    Sub,
    #[strum(serialize = "*")]
    Mul,
    #[strum(serialize = "/")]
    Div,
    #[strum(serialize = "==")]
    Eq,
    #[strum(serialize = "!=")]
    Neq,
    #[strum(serialize = "<")]
    LT,
    #[strum(serialize = ">")]
    GT,
}

impl Display for Infix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

impl Node for Infix<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.left.span().start,
            end: self.right.span().end,
        }
    }
}

#[derive(Debug)]
pub struct Prefix<'a> {
    pub token: Token<'a>,
    pub operator: PrefixOperator,
    // Boxed for indirection :<
    pub operand: Box<Expression<'a>>,
}

#[derive(strum::Display, PartialEq, Eq, Debug)]
pub enum PrefixOperator {
    #[strum(serialize = "!")]
    Not,
    #[strum(serialize = "-")]
    Neg,
}

impl Display for Prefix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.operator, f)?;
        Display::fmt(&self.operand, f)
    }
}

impl Node for Prefix<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.token.span.start,
            end: self.operand.span().end,
        }
    }
}

#[derive(Debug)]
pub struct Integer<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl Display for Integer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Node for Integer<'_> {
    fn span(&self) -> Span {
        self.token.span
    }
}

#[derive(Debug)]
pub struct Let<'a> {
    pub token: Token<'a>,
    pub name: Identifier<'a>,
    pub value: Expression<'a>,
}

impl Display for Let<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} = {}",
            self.token.literal, self.name.value, self.value
        )
    }
}

impl Node for Let<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.token.span.start,
            end: self.value.span().end,
        }
    }
}

#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.value, f)
    }
}

impl Node for Identifier<'_> {
    fn span(&self) -> Span {
        self.token.span
    }
}

#[derive(Debug)]
pub struct Return<'a> {
    pub token: Token<'a>,
    pub value: Expression<'a>,
}

impl Display for Return<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {}", self.value)
    }
}

impl Node for Return<'_> {
    fn span(&self) -> Span {
        Span {
            start: self.token.span.start,
            end: self.value.span().end,
        }
    }
}
