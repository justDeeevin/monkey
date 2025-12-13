use crate::token::Token;
use std::fmt::{Debug, Display};

#[cfg(test)]
mod test;

pub trait Node: Display + Debug {
    fn literal(&self) -> &str;
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
    fn literal(&self) -> &str {
        self.statements
            .first()
            .map(Node::literal)
            .unwrap_or_default()
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
    fn literal(&self) -> &str {
        match self {
            Self::Let(l) => l.literal(),
            Self::Return(r) => r.literal(),
            Self::Expression(e) => e.literal(),
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
    fn literal(&self) -> &str {
        match self {
            Self::Identifier(identifier) => identifier.literal(),
            Self::Integer(integer) => integer.literal(),
            Self::Prefix(prefix) => prefix.literal(),
            Self::Infix(infix) => infix.literal(),
            Self::Boolean(boolean) => boolean.literal(),
            Self::If(i) => i.literal(),
            Self::Function(f) => f.literal(),
            Self::Call(call) => call.literal(),
        }
    }
}

#[derive(Debug)]
pub struct Call<'a> {
    pub token: Token<'a>,
    pub function: Box<Expression<'a>>,
    pub arguments: Vec<Expression<'a>>,
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    pub token: Token<'a>,
    pub statements: Vec<Statement<'a>>,
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
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
    fn literal(&self) -> &str {
        self.token.literal
    }
}
