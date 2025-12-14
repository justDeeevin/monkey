use crate::{
    ast::*,
    eval::intrinsic::lookup_intrinsic,
    object::{Function as FunctionObject, Object, ObjectKind},
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
    #[error("Expected {expected} arguments, got {got}")]
    WrongNumberOfArguments { expected: usize, got: usize },
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

    fn eval_statements(&mut self, mut statements: Vec<Statement<'a>>) -> Result<'a, Object<'a>> {
        if statements.is_empty() {
            return Ok(Object::Null);
        }

        let mut errors = Vec::new();

        for statement in statements.drain(..statements.len() - 1) {
            match self.eval_statement(statement) {
                Ok(Object::Return(value)) => return Ok(*value),
                Ok(_) => {}
                Err(e) => errors.push(e),
            }
        }

        self.eval_statement(statements.pop().unwrap()).map_err(|e| {
            errors.push(e);
            errors.into_iter().flatten().collect()
        })
    }

    fn eval_statement(&mut self, statement: Statement<'a>) -> Result<'a, Object<'a>> {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr, None),
            Statement::Return(Return { value, .. }) => {
                Ok(Object::Return(Box::new(self.eval_expression(value, None)?)))
            }
            Statement::Let(Let { name, value, .. }) => {
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
        match expr {
            Expression::Integer(int) => Ok(Object::Integer(int.value)),
            Expression::Boolean(bool) => Ok(Object::Boolean(bool.value)),
            Expression::Prefix(Prefix {
                operator,
                operand,
                op_token,
            }) => match operator {
                PrefixOperator::Not => Ok(Object::Boolean(
                    !self.eval_expression(*operand, None)?.truthy(),
                )),
                PrefixOperator::Neg => {
                    let span = Span {
                        start: op_token.span.start,
                        end: operand.span().end,
                    };
                    match self.eval_expression(*operand, None)? {
                        Object::Integer(i) => Ok(Object::Integer(-i)),
                        e => Err(vec![Error {
                            span,
                            kind: ErrorKind::InvalidOperand {
                                operator,
                                operand: e.into(),
                            },
                        }]),
                    }
                }
            },
            Expression::Infix(Infix {
                operator,
                left,
                right,
            }) => {
                let span = Span {
                    start: left.span().start,
                    end: right.span().end,
                };
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
            Expression::If(If {
                condition,
                consequence,
                alternative,
                ..
            }) => {
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
            Expression::Function(Function {
                parameters, body, ..
            }) => Ok(Object::Function(Rc::new(FunctionObject {
                this: ident,
                parameters,
                body,
                env: Self {
                    outer: Some(Box::new(self.clone())),
                    values: HashMap::new(),
                },
            }))),
            Expression::Call(Call {
                open,
                function,
                arguments,
                close,
            }) => {
                let span = Span {
                    start: function.span().start,
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
            Expression::String(String { value, .. }) => Ok(Object::String(value.to_string())),
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
