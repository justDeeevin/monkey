use crate::token::Token;
use std::fmt::Display;

#[cfg(test)]
mod test;

pub trait Node: Display {
    fn literal(&self) -> &str;
}

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

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(StatementKind))]
pub enum Statement<'a> {
    Let(Let<'a>),
    Return(Return<'a>),
    Expression(Expression<'a>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Let(l) => l.fmt(f),
            Self::Return(r) => r.fmt(f),
            Self::Expression(e) => e.fmt(f),
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

#[derive(strum::EnumDiscriminants)]
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
            Self::Identifier(i) => i.fmt(f),
            Self::Integer(i) => i.fmt(f),
            Self::Prefix(p) => p.fmt(f),
            Self::Infix(i) => i.fmt(f),
            Self::Boolean(b) => b.fmt(f),
            Self::If(i) => i.fmt(f),
            Self::Function(fun) => fun.fmt(f),
            Self::Call(call) => call.fmt(f),
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

pub struct Call<'a> {
    pub token: Token<'a>,
    pub function: Box<Expression<'a>>,
    pub arguments: Vec<Expression<'a>>,
}

impl Display for Call<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.function.fmt(f)?;
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
        self.body.fmt(f)
    }
}

impl Node for Function<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

pub struct If<'a> {
    pub token: Token<'a>,
    pub condition: Box<Expression<'a>>,
    pub consequence: BlockStatement<'a>,
    pub alternative: Option<BlockStatement<'a>>,
}

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

pub struct Boolean<'a> {
    pub token: Token<'a>,
    pub value: bool,
}

impl Display for Boolean<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl Node for Boolean<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

pub struct Infix<'a> {
    pub token: Token<'a>,
    pub left: Box<Expression<'a>>,
    pub operator: InfixOperator,
    pub right: Box<Expression<'a>>,
}

#[derive(strum::Display, PartialEq, Eq, Debug)]
pub enum InfixOperator {
    #[strum(serialize = "`+`")]
    Add,
    #[strum(serialize = "`-`")]
    Sub,
    #[strum(serialize = "`*`")]
    Mul,
    #[strum(serialize = "`/`")]
    Div,
    #[strum(serialize = "`==`")]
    Eq,
    #[strum(serialize = "`!=`")]
    Neq,
    #[strum(serialize = "`<`")]
    LT,
    #[strum(serialize = "`>`")]
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
        self.operator.fmt(f)?;
        self.operand.fmt(f)
    }
}

impl Node for Prefix<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

pub struct Integer<'a> {
    pub token: Token<'a>,
    pub value: i64,
}

impl Display for Integer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl Node for Integer<'_> {
    fn literal(&self) -> &str {
        self.token.literal
    }
}

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

pub struct Identifier<'a> {
    pub token: Token<'a>,
    pub value: &'a str,
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
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
