use crate::{
    ast::*,
    eval::intrinsic::lookup_intrinsic,
    object::{Function as FunctionObject, Map, Object, ObjectKind},
    token::Span,
};
use std::{collections::HashMap, rc::Rc};

mod intrinsic;
#[cfg(test)]
mod test;

#[derive(thiserror::Error, Debug)]
#[error("{kind}")]
pub struct Error<'a> {
    span: Span,
    kind: ErrorKind<'a>,
}

impl Error<'_> {
    pub fn report(&self, input: &str) {
        use ariadne::{Color, Label, Report, ReportKind, Source};

        Report::build(ReportKind::Error, self.span)
            .with_message(self.to_string())
            .with_label(Label::new(self.span).with_color(Color::Red))
            .finish()
            .eprint(("input", Source::from(input)))
            .unwrap();
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind<'a> {
    #[error("Cannot use `{operator}` with {left} and {right}")]
    InvalidOperands {
        operator: InfixOperator,
        left: ObjectKind,
        right: ObjectKind,
    },
    #[error("Cannot use `{operator}` with {operand}")]
    InvalidOperand {
        operator: PrefixOperator,
        operand: ObjectKind,
    },
    #[error("Undefined variable `{0}`")]
    UndefinedVariable(&'a str),
    #[error("Cannot call a non-function")]
    NotAFunction,
    #[error("Expected {expected} argument{}, got {got}", if *expected > 1 { "s" } else { "" })]
    WrongNumberOfArguments { expected: usize, got: usize },
    #[error("Cannot index into a non-collection")]
    NotACollection,
    #[error("Cannot index an array with a non-integer")]
    NotAnIndex,
    #[error("Out of bounds: index was {i} but length was {len}")]
    OutOfBounds { i: i64, len: usize },
    #[error("{0} cannot be used as a map key")]
    InvalidKey(ObjectKind),
    #[error("Length greater than i64::MAX")]
    TooLongForLen,
    #[error("Cannot get length of {0}")]
    BadTypeForLen(ObjectKind),
}

pub type Result<'a, T, E = Vec<Error<'a>>> = std::result::Result<T, E>;

#[derive(Default, Debug, Clone)]
pub struct Environment<'a> {
    outer: Option<Box<Self>>,
    values: HashMap<&'a str, Object<'a>>,
}

impl<'a> Environment<'a> {
    pub fn eval_program(&mut self, program: Program<'a>) -> Result<'a, Object<'a>> {
        self.eval_statements(program.statements)
    }

    fn eval_statements(&mut self, statements: Vec<Statement<'a>>) -> Result<'a, Object<'a>> {
        if statements.is_empty() {
            return Ok(Object::Null);
        }

        let mut errors = Vec::new();
        let mut last = Object::Null;

        for statement in statements {
            match self.eval_statement(statement) {
                Ok(Object::Return(value)) => return Ok(*value),
                Ok(out) => last = out,
                Err(e) => errors.extend(e),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(last)
        }
    }

    fn eval_statement(&mut self, statement: Statement<'a>) -> Result<'a, Object<'a>> {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr, None),
            Statement::Return { value, .. } => {
                Ok(Object::Return(Box::new(self.eval_expression(value, None)?)))
            }
            Statement::Let { name, value, .. } => {
                let str = name.value;
                let value = self.eval_expression(value, Some(name))?;
                self.values.insert(str, value);
                Ok(Object::Null)
            }
        }
    }

    fn eval_expression(
        &mut self,
        expr: Expression<'a>,
        ident: Option<Identifier<'a>>,
    ) -> Result<'a, Object<'a>> {
        let span = expr.span();
        match expr {
            Expression::Integer { value, .. } => Ok(Object::Integer(value)),
            Expression::Boolean { value, .. } => Ok(Object::Boolean(value)),
            Expression::Prefix {
                operator, operand, ..
            } => match operator {
                PrefixOperator::Not => Ok(Object::Boolean(
                    !self.eval_expression(*operand, None)?.truthy(),
                )),
                PrefixOperator::Neg => match self.eval_expression(*operand, None)? {
                    Object::Integer(i) => Ok(Object::Integer(-i)),
                    e => Err(vec![Error {
                        span,
                        kind: ErrorKind::InvalidOperand {
                            operator,
                            operand: e.into(),
                        },
                    }]),
                },
            },
            Expression::Infix {
                operator,
                left,
                right,
            } => {
                match (
                    operator,
                    self.eval_expression(*left, None)?,
                    self.eval_expression(*right, None)?,
                ) {
                    (InfixOperator::Add, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Integer(left + right))
                    }
                    (InfixOperator::Add, Object::String(left), Object::String(right)) => {
                        Ok(Object::String(left + &right))
                    }
                    (InfixOperator::Sub, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Integer(left - right))
                    }
                    (InfixOperator::Mul, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Integer(left * right))
                    }
                    (InfixOperator::Div, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Integer(left / right))
                    }
                    (InfixOperator::LT, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Boolean(left < right))
                    }
                    (InfixOperator::GT, Object::Integer(left), Object::Integer(right)) => {
                        Ok(Object::Boolean(left > right))
                    }
                    (InfixOperator::Eq, left, right)
                        if !matches!(&left, Object::Function { .. })
                            && !matches!(&right, Object::Function { .. }) =>
                    {
                        Ok(Object::Boolean(left == right))
                    }
                    (InfixOperator::Neq, left, right)
                        if !matches!(&left, Object::Function { .. })
                            && !matches!(&right, Object::Function { .. }) =>
                    {
                        Ok(Object::Boolean(left != right))
                    }
                    (operator, left, right) => Err(vec![Error {
                        span,
                        kind: ErrorKind::InvalidOperands {
                            operator,
                            left: left.into(),
                            right: right.into(),
                        },
                    }]),
                }
            }
            Expression::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                if self.eval_expression(*condition, None)?.truthy() {
                    self.eval_block_statement(consequence)
                } else if let Some(alternative) = alternative {
                    self.eval_block_statement(alternative)
                } else {
                    Ok(Object::Null)
                }
            }
            Expression::Identifier(Identifier { value, token }) => {
                self.find_variable(value).ok_or_else(|| {
                    vec![Error {
                        span: token.span,
                        kind: ErrorKind::UndefinedVariable(value),
                    }]
                })
            }
            Expression::Function {
                parameters, body, ..
            } => Ok(Object::Function(Rc::new(FunctionObject {
                this: ident,
                parameters,
                body,
                env: Self {
                    outer: Some(Box::new(self.clone())),
                    values: HashMap::new(),
                },
            }))),
            Expression::Call {
                open,
                function,
                arguments,
                close,
            } => {
                let args_span = Span {
                    start: open.span.start,
                    end: close.span.end,
                };
                if let Expression::Identifier(Identifier { value, .. }) = *function
                    && let Some(intrinsic) = lookup_intrinsic(value)
                {
                    return intrinsic(
                        &arguments
                            .into_iter()
                            .map(|a| self.eval_expression(a, None))
                            .collect::<Result<'a, Vec<_>>>()?,
                        args_span,
                    );
                }

                let Object::Function(function) = self.eval_expression(*function, None)? else {
                    return Err(vec![Error {
                        span,
                        kind: ErrorKind::NotAFunction,
                    }]);
                };

                if arguments.len() != function.parameters.len() {
                    return Err(vec![Error {
                        span: Span {
                            start: open.span.start,
                            end: close.span.end,
                        },
                        kind: ErrorKind::WrongNumberOfArguments {
                            expected: function.parameters.len(),
                            got: arguments.len(),
                        },
                    }]);
                }

                let mut call_env = function.env.clone();
                for (argument, param) in arguments
                    .into_iter()
                    .map(|a| self.eval_expression(a, None))
                    .zip(&function.parameters)
                {
                    call_env.values.insert(param.value, argument?);
                }
                if let Some(this) = &function.this {
                    call_env
                        .values
                        .insert(this.value, Object::Function(function.clone()));
                }

                call_env.eval_block_statement(function.body.clone())
            }
            Expression::Null(_) => Ok(Object::Null),
            Expression::String { value, .. } => Ok(Object::String(value.to_string())),
            Expression::Array { elements, .. } => Ok(Object::Array(
                elements
                    .into_iter()
                    .map(|e| self.eval_expression(e, None))
                    .collect::<Result<'a, Vec<_>>>()?,
            )),
            Expression::Index {
                collection, index, ..
            } => match self.eval_expression(*collection, None)? {
                Object::Array(mut array) => {
                    let span = index.span();

                    let Object::Integer(index) = self.eval_expression(*index, None)? else {
                        return Err(vec![Error {
                            span,
                            kind: ErrorKind::NotAnIndex,
                        }]);
                    };

                    if index < 0 || index >= array.len() as i64 {
                        return Err(vec![Error {
                            span,
                            kind: ErrorKind::OutOfBounds {
                                i: index,
                                len: array.len(),
                            },
                        }]);
                    }

                    Ok(array.remove(index as usize))
                }
                Object::Map(map) => {
                    let index = self.eval_expression(*index, None)?;
                    if matches!(index, Object::Function { .. } | Object::Map(_)) {
                        Err(vec![Error {
                            span,
                            kind: ErrorKind::InvalidKey(index.into()),
                        }])
                    } else {
                        Ok(map.get(&index).cloned().unwrap_or(Object::Null))
                    }
                }
                _ => Err(vec![Error {
                    span,
                    kind: ErrorKind::NotACollection,
                }]),
            },
            Expression::Map { elements, .. } => {
                let mut map = Map::default();

                for (key, value) in elements {
                    let key_span = key.span();
                    let key = self.eval_expression(key, None)?;
                    if matches!(key, Object::Function { .. } | Object::Map(_)) {
                        return Err(vec![Error {
                            span: key_span,
                            kind: ErrorKind::InvalidKey(key.into()),
                        }]);
                    }
                    map.insert(key, self.eval_expression(value, None)?);
                }

                Ok(Object::Map(map))
            }
        }
    }

    fn eval_block_statement(&self, block: BlockStatement<'a>) -> Result<'a, Object<'a>> {
        let mut env = Environment {
            outer: Some(Box::new(self.clone())),
            values: HashMap::new(),
        };

        env.eval_statements(block.statements)
    }

    fn find_variable(&self, name: &'a str) -> Option<Object<'a>> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.outer.as_ref()?.find_variable(name))
    }
}
