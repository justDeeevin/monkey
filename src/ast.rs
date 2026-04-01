use chumsky::span::Spanned;
use std::fmt::{Debug, Display};
use strum::Display;

fn write_indent(f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
    const INDENT: &str = "  ";
    for _ in 0..indent {
        f.write_str(INDENT)?;
    }
    Ok(())
}

trait DisplayIndented {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result;
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Let {
        name: Spanned<&'a str>,
        value: Spanned<Expression<'a>>,
    },
    Return(Spanned<Expression<'a>>),
    Expression {
        value: Spanned<Expression<'a>>,
        semi: bool,
    },
}

impl DisplayIndented for Statement<'_> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Let {
                name: Spanned { inner: name, .. },
                value: Spanned { inner: value, .. },
                ..
            } => write!(f, "let {name} = {value};"),
            Self::Return(Spanned { inner: value, .. }) => write!(f, "return {value};"),
            Self::Expression { value, semi } => {
                value.fmt_indented(f, indent)?;
                if *semi { write!(f, ";") } else { Ok(()) }
            }
        }
    }
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub statements: Vec<Spanned<Statement<'a>>>,
}

impl DisplayIndented for Block<'_> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in &self.statements {
            write_indent(f, indent + 1)?;
            statement.fmt_indented(f, indent + 1)?;
        }
        writeln!(f)?;
        write_indent(f, indent)?;
        write!(f, "}}")
    }
}

impl Display for Block<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Identifier(&'a str),
    Integer(i64),
    Prefix {
        prefix: PrefixOperator,
        right: Box<Spanned<Self>>,
    },
    Infix {
        left: Box<Spanned<Self>>,
        operator: InfixOperator,
        right: Box<Spanned<Self>>,
    },
    Boolean(bool),
    If {
        condition: Box<Spanned<Self>>,
        consequence: Spanned<Block<'a>>,
        alternative: Option<Spanned<Block<'a>>>,
    },
    Function {
        parameters: Vec<Spanned<&'a str>>,
        body: Spanned<Block<'a>>,
    },
    Call {
        function: Box<Spanned<Self>>,
        arguments: Vec<Spanned<Self>>,
    },
    Null,
    String(String),
    Array(Vec<Spanned<Self>>),
    Index {
        collection: Box<Spanned<Self>>,
        index: Box<Spanned<Self>>,
    },
    Map(Vec<(Spanned<Self>, Spanned<Self>)>),
}

impl DisplayIndented for Expression<'_> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Identifier(ident) => Display::fmt(ident, f),
            Self::Integer(value) => Display::fmt(&value, f),
            Self::Prefix { prefix, right } => {
                Display::fmt(&prefix, f)?;
                right.fmt_indented(f, indent)
            }
            Self::Infix {
                left,
                operator,
                right,
            } => {
                write!(f, "(")?;
                left.fmt_indented(f, indent)?;
                write!(f, " {operator} ")?;
                right.fmt_indented(f, indent)?;
                write!(f, ")")
            }
            Self::Boolean(value) => Display::fmt(&value, f),
            Self::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                write!(f, "if ")?;
                condition.fmt_indented(f, indent)?;
                write!(f, " ")?;
                consequence.fmt_indented(f, indent)?;
                if let Some(alternative) = alternative.as_ref() {
                    write!(f, " else ")?;
                    alternative.fmt_indented(f, indent)?;
                }
                Ok(())
            }
            Self::Function {
                parameters, body, ..
            } => {
                write!(
                    f,
                    "fn({}",
                    parameters.first().map(|v| v.inner).unwrap_or_default()
                )?;
                for parameter in parameters.iter().skip(1) {
                    write!(f, ", {}", parameter.inner)?;
                }
                write!(f, ") ")?;
                body.fmt_indented(f, indent)
            }
            Self::Call {
                function,
                arguments,
                ..
            } => {
                function.fmt_indented(f, indent)?;
                write!(f, "(")?;
                if let Some(first) = arguments.first() {
                    first.fmt_indented(f, indent)?;
                }
                for argument in arguments.iter().skip(1) {
                    write!(f, ", ")?;
                    argument.fmt_indented(f, indent)?;
                }
                write!(f, ")")
            }
            Self::Null => write!(f, "null"),
            Self::String(value) => write!(f, "{value:?}"),
            Self::Array(elements) => {
                write!(f, "[")?;
                if let Some(first) = elements.first() {
                    first.fmt_indented(f, indent)?;
                }
                for element in elements.iter().skip(1) {
                    write!(f, ", ")?;
                    element.fmt_indented(f, indent)?;
                }
                write!(f, "]")
            }
            Self::Index {
                collection, index, ..
            } => {
                collection.fmt_indented(f, indent)?;
                write!(f, "[")?;
                index.fmt_indented(f, indent)?;
                write!(f, "]")
            }
            Self::Map(elements) => {
                writeln!(f, "{{")?;
                for (key, value) in elements.iter() {
                    write_indent(f, indent + 1)?;
                    key.fmt_indented(f, indent + 1)?;
                    write!(f, ": ")?;
                    value.fmt_indented(f, indent + 1)?;
                    writeln!(f, ",")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

#[derive(Debug, Display, Clone)]
pub enum PrefixOperator {
    #[strum(to_string = "-")]
    Neg,
    #[strum(to_string = "!")]
    Not,
}

#[derive(Debug, Clone, Copy, Display)]
pub enum InfixOperator {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    Neq,
    #[strum(to_string = "<")]
    LT,
    #[strum(to_string = ">")]
    GT,
}

#[derive(Debug)]
pub struct Program<'a> {
    pub statements: Vec<Spanned<Statement<'a>>>,
}

impl Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{}", statement.inner)?;
        }
        Ok(())
    }
}
