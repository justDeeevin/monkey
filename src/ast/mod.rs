use crate::token::{Span, Token};
use std::fmt::{Debug, Display};

#[cfg(test)]
mod test;

pub trait Node: Display {
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

#[derive(strum::EnumDiscriminants, Debug, Clone)]
#[strum_discriminants(name(StatementKind))]
pub enum Statement<'a> {
    Let {
        let_token: Token<'a>,
        name: Identifier<'a>,
        value: Expression<'a>,
    },
    Return {
        return_token: Token<'a>,
        value: Expression<'a>,
    },
    Expression(Expression<'a>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Let { name, value, .. } => write!(f, "let {name} = {value};"),
            Self::Return { value, .. } => write!(f, "return {value};"),
            Self::Expression(e) => Display::fmt(e, f),
        }?;

        write!(f, ";")
    }
}

impl Node for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Let {
                let_token, value, ..
            } => Span {
                start: let_token.span.start,
                end: value.span().end,
            },
            Self::Return {
                return_token,
                value,
            } => Span {
                start: return_token.span.start,
                end: value.span().end,
            },
            Self::Expression(e) => e.span(),
        }
    }
}

#[derive(strum::EnumDiscriminants, Debug, Clone)]
#[strum_discriminants(name(ExpressionKind))]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    Integer {
        token: Token<'a>,
        value: i64,
    },
    Prefix {
        op_token: Token<'a>,
        operator: PrefixOperator,
        operand: Box<Self>,
    },
    Infix {
        left: Box<Self>,
        operator: InfixOperator,
        right: Box<Self>,
    },
    Boolean {
        token: Token<'a>,
        value: bool,
    },
    If {
        if_token: Token<'a>,
        condition: Box<Self>,
        consequence: BlockStatement<'a>,
        alternative: Option<BlockStatement<'a>>,
    },
    Function {
        fn_token: Token<'a>,
        parameters: Vec<Identifier<'a>>,
        body: BlockStatement<'a>,
    },
    Call {
        function: Box<Self>,
        open: Token<'a>,
        arguments: Vec<Self>,
        close: Token<'a>,
    },
    Null(Token<'a>),
    String {
        token: Token<'a>,
        value: &'a str,
    },
    Array {
        open: Token<'a>,
        elements: Vec<Self>,
        close: Token<'a>,
    },
    Index {
        collection: Box<Self>,
        index: Box<Self>,
        close: Token<'a>,
    },
    Map {
        open: Token<'a>,
        elements: Vec<(Expression<'a>, Expression<'a>)>,
        close: Token<'a>,
    },
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        match self {
            Self::Identifier(i) => Display::fmt(i, f),
            Self::Integer { value, .. } => Display::fmt(value, f),
            Self::Prefix {
                operator, operand, ..
            } => {
                Display::fmt(operator, f)?;
                Display::fmt(operand, f)
            }
            Self::Infix {
                left,
                operator,
                right,
            } => write!(f, "{left} {operator} {right}"),
            Self::Boolean { value, .. } => Display::fmt(value, f),
            Self::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                write!(f, "if {condition} {consequence}")?;
                if let Some(alternative) = alternative {
                    write!(f, " else {alternative}")?;
                }
                Ok(())
            }
            Self::Function {
                parameters, body, ..
            } => {
                write!(f, "fn(")?;
                if !parameters.is_empty() {
                    for parameter in parameters.iter().take(parameters.len() - 1) {
                        write!(f, "{}, ", parameter)?;
                    }
                    Display::fmt(parameters.last().unwrap(), f)?;
                }
                write!(f, ") ")?;
                Display::fmt(body, f)
            }
            Self::Call {
                function,
                arguments,
                ..
            } => {
                write!(f, "{function}(")?;
                if !arguments.is_empty() {
                    for argument in arguments.iter().take(arguments.len() - 1) {
                        write!(f, "{}, ", argument)?;
                    }
                    Display::fmt(arguments.last().unwrap(), f)?;
                }
                write!(f, ")")
            }
            Self::Null(_) => write!(f, "null"),
            Self::String { value, .. } => Display::fmt(value, f),
            Self::Array { elements, .. } => {
                write!(f, "[")?;
                if !elements.is_empty() {
                    for element in elements.iter().take(elements.len() - 1) {
                        write!(f, "{}, ", element)?;
                    }
                    Display::fmt(elements.last().unwrap(), f)?;
                }
                write!(f, "]")
            }
            Self::Index {
                collection: array,
                index,
                ..
            } => write!(f, "{array}[{index}]"),
            Self::Map { elements, .. } => {
                write!(f, "{{")?;
                if !elements.is_empty() {
                    for (key, value) in elements.iter().take(elements.len() - 1) {
                        write!(f, "{key}: {value}, ")?;
                    }
                    let (key, value) = elements.last().unwrap();
                    write!(f, "{key}: {value}",)?;
                }
                write!(f, "}}")
            }
        }?;
        write!(f, ")")
    }
}

impl Node for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span(),
            Self::Integer { token, .. } => token.span,
            Self::Prefix {
                op_token, operand, ..
            } => Span {
                start: op_token.span.start,
                end: operand.span().end,
            },
            Self::Infix { left, right, .. } => Span {
                start: left.span().start,
                end: right.span().end,
            },
            Self::Boolean { token, .. } => token.span,
            Self::If {
                if_token,
                consequence,
                alternative,
                ..
            } => Span {
                start: if_token.span.start,
                end: alternative
                    .as_ref()
                    .map(|s| s.span().end)
                    .unwrap_or(consequence.span().end),
            },
            Self::Function { fn_token, body, .. } => Span {
                start: fn_token.span.start,
                end: body.span().end,
            },
            Self::Call {
                function, close, ..
            } => Span {
                start: function.span().start,
                end: close.span.end,
            },
            Self::Null(null) => null.span,
            Self::String { token, .. } => token.span,
            Self::Array { open, close, .. } => Span {
                start: open.span.start,
                end: close.span.end,
            },
            Self::Index {
                collection: array,
                close,
                ..
            } => Span {
                start: array.span().start,
                end: close.span.end,
            },
            Self::Map { open, close, .. } => Span {
                start: open.span.start,
                end: close.span.end,
            },
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(strum::Display, PartialEq, Eq, Debug, Clone, Copy)]
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

#[derive(strum::Display, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PrefixOperator {
    #[strum(serialize = "!")]
    Not,
    #[strum(serialize = "-")]
    Neg,
}

#[derive(Debug, Clone)]
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
