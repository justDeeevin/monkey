use std::fmt::{Debug, Display};

#[derive(Default, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn join(self, other: Span) -> Span {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl ariadne::Span for Span {
    type SourceId = ();

    fn source(&self) -> &Self::SourceId {
        &()
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

impl<T: std::ops::RangeBounds<usize>> From<T> for Span {
    fn from(value: T) -> Self {
        let start = match value.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(start) => *start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match value.end_bound() {
            std::ops::Bound::Included(end) => *end,
            std::ops::Bound::Excluded(end) => *end - 1,
            std::ops::Bound::Unbounded => usize::MAX,
        };
        Self { start, end }
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

pub trait Node: Spanned + Display + Debug {}

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

#[derive(Debug)]
pub struct Identifier<'a> {
    pub name: &'a str,
    pub span: Span,
}

impl Spanned for Identifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Node for Identifier<'_> {}

#[derive(Debug)]
pub enum Statement<'a> {
    Let {
        let_span: Span,
        name: Identifier<'a>,
        value: Expression<'a>,
    },
    Return {
        return_span: Span,
        value: Expression<'a>,
    },
    Expression(Expression<'a>),
}

impl Spanned for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Let {
                let_span, value, ..
            } => let_span.join(value.span()),
            Self::Return { return_span, value } => return_span.join(value.span()),
            Self::Expression(expr) => expr.span(),
        }
    }
}

impl DisplayIndented for Statement<'_> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Let { name, value, .. } => write!(f, "let {name} = {value}"),
            Self::Return { value, .. } => write!(f, "return {value}"),
            Self::Expression(expr) => expr.fmt_indented(f, indent),
        }
    }
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl Node for Statement<'_> {}

#[derive(Debug)]
pub struct Block<'a> {
    pub open_span: Span,
    pub statements: Vec<Statement<'a>>,
    pub close_span: Span,
}

impl Spanned for Block<'_> {
    fn span(&self) -> Span {
        self.open_span.join(self.close_span)
    }
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

impl Node for Block<'_> {}

#[derive(Debug)]
pub enum Expression<'a> {
    Identifier(Identifier<'a>),
    Integer {
        span: Span,
        value: i64,
    },
    Prefix {
        prefix: Prefix,
        right: Box<Self>,
    },
    Infix {
        left: Box<Self>,
        operator: InfixOperator,
        right: Box<Self>,
    },
    Boolean {
        span: Span,
        value: bool,
    },
    If {
        if_span: Span,
        condition: Box<Self>,
        consequence: Block<'a>,
        alternative: Option<Block<'a>>,
    },
    Function {
        fn_span: Span,
        parameters: Vec<Identifier<'a>>,
        body: Block<'a>,
    },
    Call {
        function: Box<Self>,
        arguments: Vec<Self>,
        close_span: Span,
    },
    Null(Span),
    String {
        span: Span,
        value: String,
    },
    Array {
        open_span: Span,
        elements: Vec<Self>,
        close_span: Span,
    },
    Index {
        collection: Box<Self>,
        index: Box<Self>,
        close_span: Span,
    },
    Map {
        open_span: Span,
        elements: Vec<(Self, Self)>,
        close_span: Span,
    },
}

impl Spanned for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(ident) => ident.span(),
            Self::Integer { span, .. } => *span,
            Self::Prefix {
                prefix: operator,
                right,
            } => operator.span.join(right.span()),
            Self::Infix { left, right, .. } => left.span().join(right.span()),
            Self::Boolean { span, .. } => *span,
            Self::If {
                if_span,
                consequence,
                alternative,
                ..
            } => if_span.join(alternative.as_ref().unwrap_or(consequence).span()),
            Self::Function { fn_span, body, .. } => fn_span.join(body.span()),
            Self::Call {
                function,
                close_span,
                ..
            } => function.span().join(*close_span),
            Self::Null(span) => *span,
            Self::String { span, .. } => *span,
            Self::Array {
                open_span,
                close_span,
                ..
            } => open_span.join(*close_span),
            Self::Index {
                collection,
                close_span,
                ..
            } => collection.span().join(*close_span),
            Self::Map {
                open_span,
                close_span,
                ..
            } => open_span.join(*close_span),
        }
    }
}

impl DisplayIndented for Expression<'_> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Identifier(ident) => Display::fmt(ident, f),
            Self::Integer { value, .. } => Display::fmt(value, f),
            Self::Prefix { prefix, right } => {
                Display::fmt(prefix, f)?;
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
            Self::Boolean { value, .. } => write!(f, "{value}"),
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
                    parameters
                        .first()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                )?;
                for parameter in parameters.iter().skip(1) {
                    write!(f, ", {parameter}")?;
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
            Self::Null(_) => write!(f, "null"),
            Self::String { value, .. } => write!(f, "{value:?}"),
            Self::Array { elements, .. } => {
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
            Self::Map { elements, .. } => {
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

impl Node for Expression<'_> {}

#[derive(Debug)]
pub struct Prefix {
    pub span: Span,
    pub operator: PrefixOperator,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.operator {
            PrefixOperator::Neg => write!(f, "-"),
            PrefixOperator::Not => write!(f, "!"),
        }
    }
}

#[derive(Debug)]
pub enum PrefixOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum InfixOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    LT,
    GT,
}

impl Display for InfixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfixOperator::Add => write!(f, "+"),
            InfixOperator::Sub => write!(f, "-"),
            InfixOperator::Mul => write!(f, "*"),
            InfixOperator::Div => write!(f, "/"),
            InfixOperator::Eq => write!(f, "=="),
            InfixOperator::Neq => write!(f, "!="),
            InfixOperator::LT => write!(f, "<"),
            InfixOperator::GT => write!(f, ">"),
        }
    }
}

impl InfixOperator {
    pub fn precedence(&self) -> (u8, u8) {
        match self {
            Self::Eq | Self::Neq => (1, 2),
            Self::LT | Self::GT => (3, 4),
            Self::Add | Self::Sub => (5, 6),
            Self::Mul | Self::Div => (7, 8),
        }
    }
}

#[derive(Debug)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl Spanned for Program<'_> {
    fn span(&self) -> Span {
        self.statements
            .first()
            .map(Spanned::span)
            .and_then(|span| Some(span.join(self.statements.last()?.span())))
            .unwrap_or_default()
    }
}

impl Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{statement}")?;
        }
        Ok(())
    }
}
