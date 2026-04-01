use chumsky::span::{SimpleSpan, SpanWrap, Spanned};

use crate::{ast::*, intrinsic::find_intrinsic, value::*};
use std::{collections::HashMap, rc::Rc};

pub type Result<'a, T, E = Spanned<Error<'a>>> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error<'a> {
    #[error("unknown identifier: {0}")]
    UnknownIdentifier(&'a str),
    #[error("cannot negate {0}")]
    InvalidNeg(Type),
    #[error("cannot use {0} on {1} and {2}")]
    InvalidInfix(InfixOperator, Type, Type),
    #[error("attempted to call non-function ({0})")]
    NonFunction(Type),
    #[error(
        "attempted to call function with wrong number of arguments (expected {expected}, found {found})"
    )]
    WrongNumberOfArguments { expected: usize, found: usize },
    #[error("index out of bounds; len was {len} but index was {index}")]
    IndexOutOfBounds { len: usize, index: i64 },
    #[error("cannot index {0} with {1}")]
    InvalidIndex(Type, Type),
    #[error("cannot use {0} as a map key")]
    InvalidMapKey(Type),
}

impl Error<'_> {
    pub fn note(&self) -> Option<&'static str> {
        match self {
            Self::InvalidNeg(_) => Some("Only integers can be negated"),
            Self::IndexOutOfBounds { index: ..0, .. } => Some("Index cannot be negative"),
            Self::InvalidMapKey(_) => Some("Only strings, integers, and booleans can be map keys"),
            _ => None,
        }
    }
}

impl Error<'_> {
    pub fn report(error: Spanned<Self>, input: &str) {
        use ariadne::{Color, Label, Report, ReportKind, Source};

        let mut builder = Report::build(ReportKind::Error, error.span.into_range())
            .with_message(&error.inner)
            .with_label(Label::new(error.span.into_range()).with_color(Color::Red));

        if let Some(note) = error.inner.note() {
            builder = builder.with_note(note);
        }

        builder.finish().eprint(Source::from(input)).unwrap();
    }
}

#[derive(Default)]
pub struct Environment<'a> {
    pub locals: HashMap<&'a str, Value<'a>>,
}

impl<'a> Environment<'a> {
    pub fn eval(&mut self, program: Program<'a>) -> Result<'a, Value<'a>> {
        self.eval_statements(program.statements)
    }

    fn eval_statements(
        &mut self,
        statements: Vec<Spanned<Statement<'a>>>,
    ) -> Result<'a, Value<'a>> {
        for statement in statements {
            if let Some(ret) = self.eval_statement(statement)? {
                return Ok(ret);
            }
        }

        Ok(Value::Null)
    }

    fn eval_statement(
        &mut self,
        statement: Spanned<Statement<'a>>,
    ) -> Result<'a, Option<Value<'a>>> {
        match statement.inner {
            Statement::Let { name, value, .. } => {
                let value = self.eval_expression(value, Some(name))?;
                self.locals.insert(&name, value);
                Ok(None)
            }
            Statement::Return(value) | Statement::Expression { value, semi: false } => {
                self.eval_expression(value, None).map(Some)
            }
            Statement::Expression { value, .. } => {
                let _ = self.eval_expression(value, None)?;
                Ok(None)
            }
        }
    }

    fn eval_expression(
        &mut self,
        expression: Spanned<Expression<'a>>,
        name: Option<Spanned<&'a str>>,
    ) -> Result<'a, Value<'a>> {
        match expression.inner {
            Expression::Identifier(ident) => self
                .locals
                .get(&ident)
                .cloned()
                .ok_or(Error::UnknownIdentifier(ident).with_span(expression.span)),
            Expression::Integer(value) => Ok(Value::Int(value)),
            Expression::Prefix { prefix, right } => {
                let right = self.eval_expression(*right, None)?;
                match (prefix, right) {
                    (PrefixOperator::Neg, Value::Int(value)) => Ok(Value::Int(-value)),
                    (PrefixOperator::Not, right) => Ok(Value::Bool(!right.truthy())),
                    (PrefixOperator::Neg, right) => {
                        Err(Error::InvalidNeg(right.into()).with_span(expression.span))
                    }
                }
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left = self.eval_expression(*left, None)?;
                let right = self.eval_expression(*right, None)?;
                match (left, operator, right) {
                    (left, InfixOperator::Eq, right) => Ok(Value::Bool(left == right)),
                    (left, InfixOperator::Neq, right) => Ok(Value::Bool(left != right)),
                    (Value::Int(l), _, Value::Int(r)) => match operator {
                        InfixOperator::Add => Ok(Value::Int(l + r)),
                        InfixOperator::Sub => Ok(Value::Int(l - r)),
                        InfixOperator::Mul => Ok(Value::Int(l * r)),
                        InfixOperator::Div => Ok(Value::Int(l / r)),
                        InfixOperator::LT => Ok(Value::Bool(l < r)),
                        InfixOperator::GT => Ok(Value::Bool(l > r)),
                        _ => Err(Error::InvalidInfix(operator, Type::Int, Type::Int)
                            .with_span(expression.span)),
                    },
                    (Value::String(l), InfixOperator::Add, Value::String(r)) => {
                        Ok(Value::String(l + &r))
                    }
                    (left, _, right) => {
                        Err(Error::InvalidInfix(operator, left.into(), right.into())
                            .with_span(expression.span))
                    }
                }
            }
            Expression::Boolean(value) => Ok(Value::Bool(value)),
            Expression::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                let condition = self.eval_expression(*condition, None)?;
                if condition.truthy() {
                    self.eval_statements(consequence.inner.statements)
                } else if let Some(alternative) = alternative {
                    self.eval_statements(alternative.inner.statements)
                } else {
                    Ok(Value::Null)
                }
            }
            Expression::Function {
                parameters, body, ..
            } => Ok(Value::Function(Rc::new(
                Function {
                    name,
                    parameters,
                    body,
                }
                .with_span(expression.span),
            ))),
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                if let Expression::Identifier(ident) = function.inner
                    && let Some(intrinsic) = find_intrinsic(ident)
                {
                    return intrinsic(
                        expression.span,
                        arguments
                            .into_iter()
                            .map(|arg| self.eval_expression(arg, None))
                            .collect::<Result<_>>()?,
                    );
                }
                let function =
                    match self.eval_expression(function.inner.with_span(function.span), None)? {
                        Value::Function(function) => function,
                        value => {
                            return Err(Error::NonFunction(value.into()).with_span(expression.span));
                        }
                    };

                let arguments = arguments
                    .into_iter()
                    .map(|arg| self.eval_expression(arg, None))
                    .collect::<Result<_>>()?;

                self.invoke(expression.span, function, arguments)
            }
            Expression::Null => Ok(Value::Null),
            Expression::String(value) => Ok(Value::String(value)),
            Expression::Array(elements) => Ok(Value::Array(
                elements
                    .into_iter()
                    .map(|e| self.eval_expression(e, None))
                    .collect::<Result<_>>()?,
            )),
            Expression::Index {
                collection, index, ..
            } => {
                let collection = self.eval_expression(*collection, None)?;
                let index = self.eval_expression(*index, None)?;
                match (collection, index) {
                    (Value::Array(array), Value::Int(index)) => {
                        if index < 0 || index as usize >= array.len() {
                            Err(Error::IndexOutOfBounds {
                                len: array.len(),
                                index,
                            }
                            .with_span(expression.span))
                        } else {
                            Ok(array[index as usize].clone())
                        }
                    }
                    (
                        Value::Map(map),
                        index @ Value::String(_) | index @ Value::Int(_) | index @ Value::Bool(_),
                    ) => Ok(map.get(&index).cloned().unwrap_or(Value::Null)),
                    (collection, index) => {
                        Err(Error::InvalidIndex(collection.into(), index.into())
                            .with_span(expression.span))
                    }
                }
            }
            Expression::Map(elements) => Ok(Value::Map(
                elements
                    .into_iter()
                    .map(|(key, value)| {
                        let key_span = key.span;
                        let key = match self.eval_expression(key, None)? {
                            key @ Value::String(_) | key @ Value::Int(_) | key @ Value::Bool(_) => {
                                key
                            }
                            key => {
                                return Err(Error::InvalidMapKey(key.into()).with_span(key_span));
                            }
                        };
                        let value = self.eval_expression(value, None)?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
            )),
        }
    }

    fn invoke(
        &self,
        call_span: SimpleSpan,
        function: Rc<Spanned<Function<'a>>>,
        arguments: Vec<Value<'a>>,
    ) -> Result<'a, Value<'a>> {
        if arguments.len() != function.parameters.len() {
            return Err(Error::WrongNumberOfArguments {
                expected: function.parameters.len(),
                found: arguments.len(),
            }
            .with_span(call_span));
        }
        let mut inner = Environment::default();

        inner.locals.extend(self.locals.clone());
        inner
            .locals
            .extend(function.parameters.iter().cloned().zip(arguments));
        if let Some(name) = function.name
            && !inner.locals.contains_key(&name)
        {
            inner.locals.insert(name, Value::Function(function.clone()));
        }

        inner.eval_statements(function.body.statements.clone())
    }
}
