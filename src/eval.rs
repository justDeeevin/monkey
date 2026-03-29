use crate::{ast::*, intrinsic::find_intrinsic, value::*};
use std::{collections::HashMap, rc::Rc};

pub type Result<'a, T, E = Error<'a>> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error("{kind}")]
pub struct Error<'a> {
    pub span: Span,
    pub kind: ErrorKind<'a>,
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind<'a> {
    #[error("unknown identifier: {0}")]
    UnknownIdentifier(Identifier<'a>),
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
}

#[derive(Default)]
pub struct Environment<'a> {
    pub locals: HashMap<Identifier<'a>, Value<'a>>,
}

impl<'a> Environment<'a> {
    pub fn eval(&mut self, program: Program<'a>) -> Result<'a, Value<'a>> {
        self.eval_statements(program.statements)
    }

    fn eval_statements(&mut self, statements: Vec<Statement<'a>>) -> Result<'a, Value<'a>> {
        for statement in statements {
            if let Some(ret) = self.eval_statement(statement)? {
                return Ok(ret);
            }
        }

        Ok(Value::Null)
    }

    fn eval_statement(&mut self, statement: Statement<'a>) -> Result<'a, Option<Value<'a>>> {
        match statement {
            Statement::Let { name, value, .. } => {
                let value = self.eval_expression(value, Some(name.clone()))?;
                self.locals.insert(name, value);
                Ok(None)
            }
            Statement::Return { value, .. } | Statement::Expression { value, semi: false } => {
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
        expression: Expression<'a>,
        name: Option<Identifier<'a>>,
    ) -> Result<'a, Value<'a>> {
        let span = expression.span();
        match expression {
            Expression::Identifier(ident) => self.locals.get(&ident).cloned().ok_or(Error {
                span,
                kind: ErrorKind::UnknownIdentifier(ident),
            }),
            Expression::Integer { value, .. } => Ok(Value::Int(value)),
            Expression::Prefix { prefix, right } => {
                let right = self.eval_expression(*right, None)?;
                match (prefix.operator, right) {
                    (PrefixOperator::Neg, Value::Int(value)) => Ok(Value::Int(-value)),
                    (PrefixOperator::Not, right) => Ok(Value::Bool(!right.truthy())),
                    (PrefixOperator::Neg, right) => Err(Error {
                        span,
                        kind: ErrorKind::InvalidNeg(right.into()),
                    }),
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
                        _ => Err(Error {
                            span,
                            kind: ErrorKind::InvalidInfix(operator, Type::Int, Type::Int),
                        }),
                    },
                    (Value::String(l), InfixOperator::Add, Value::String(r)) => {
                        Ok(Value::String(l + &r))
                    }
                    (left, _, right) => Err(Error {
                        span,
                        kind: ErrorKind::InvalidInfix(operator, left.into(), right.into()),
                    }),
                }
            }
            Expression::Boolean { value, .. } => Ok(Value::Bool(value)),
            Expression::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                let condition = self.eval_expression(*condition, None)?;
                if condition.truthy() {
                    self.eval_statements(consequence.statements)
                } else if let Some(alternative) = alternative {
                    self.eval_statements(alternative.statements)
                } else {
                    Ok(Value::Null)
                }
            }
            Expression::Function {
                parameters, body, ..
            } => Ok(Value::Function(Rc::new(Function {
                name,
                parameters,
                body,
            }))),
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                if let Expression::Identifier(ident) = function.as_ref()
                    && let Some(intrinsic) = find_intrinsic(ident.name)
                {
                    return intrinsic(
                        span,
                        arguments
                            .into_iter()
                            .map(|arg| self.eval_expression(arg, None))
                            .collect::<Result<_>>()?,
                    );
                }
                let function = match self.eval_expression(*function, None)? {
                    Value::Function(function) => function,
                    value => {
                        return Err(Error {
                            span,
                            kind: ErrorKind::NonFunction(value.into()),
                        });
                    }
                };

                let arguments = arguments
                    .into_iter()
                    .map(|arg| self.eval_expression(arg, None))
                    .collect::<Result<_>>()?;

                self.invoke(span, function, arguments)
            }
            Expression::Null(_) => Ok(Value::Null),
            Expression::String { value, .. } => Ok(Value::String(value)),
            Expression::Array { elements, .. } => Ok(Value::Array(
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
                            Err(Error {
                                span,
                                kind: ErrorKind::IndexOutOfBounds {
                                    len: array.len(),
                                    index,
                                },
                            })
                        } else {
                            Ok(array[index as usize].clone())
                        }
                    }
                    (
                        Value::Map(map),
                        index @ Value::String(_) | index @ Value::Int(_) | index @ Value::Bool(_),
                    ) => Ok(map.get(&index).cloned().unwrap_or(Value::Null)),
                    (collection, index) => Err(Error {
                        span,
                        kind: ErrorKind::InvalidIndex(collection.into(), index.into()),
                    }),
                }
            }
            Expression::Map { elements, .. } => Ok(Value::Map(
                elements
                    .into_iter()
                    .map(|(key, value)| {
                        let key = self.eval_expression(key, None)?;
                        let value = self.eval_expression(value, None)?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
            )),
        }
    }

    fn invoke(
        &mut self,
        call_span: Span,
        function: Rc<Function<'a>>,
        arguments: Vec<Value<'a>>,
    ) -> Result<'a, Value<'a>> {
        if arguments.len() != function.parameters.len() {
            return Err(Error {
                span: call_span,
                kind: ErrorKind::WrongNumberOfArguments {
                    expected: function.parameters.len(),
                    found: arguments.len(),
                },
            });
        }
        let mut inner = Environment::default();

        inner.locals.extend(self.locals.clone());
        inner
            .locals
            .extend(function.parameters.iter().cloned().zip(arguments));
        if let Some(name) = function.name.clone()
            && !inner.locals.contains_key(&name)
        {
            inner.locals.insert(name, Value::Function(function.clone()));
        }

        inner.eval_statements(function.body.statements.clone())
    }
}
