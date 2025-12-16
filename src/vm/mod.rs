use crate::{
    ast::{InfixOperator, PrefixOperator},
    code::{Op, Program, Scope, ScopedObject, SpannedObject},
    eval::{Error as EvalError, ErrorKind},
    object::Object,
    token::Span,
};
use stack::Stack;
use std::{collections::HashMap, rc::Rc};

mod stack;
#[cfg(test)]
mod test;

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("Stack underflow")]
    Underflow,
    #[error("{0}")]
    Eval(crate::eval::Error<'a>),
}

impl<'a> Error<'a> {
    pub fn report(&self, input: &'a str) {
        match self {
            Error::Underflow => eprintln!("Stack underflow"),
            Error::Eval(e) => e.report(input),
        }
    }
}

pub type Result<'a, T, E = Vec<Error<'a>>> = std::result::Result<T, E>;

#[derive(Default)]
pub struct VM<'input> {
    stack: Stack<SpannedObject<'input>>,
    pub program: Program<'input>,
    symbols: HashMap<&'input str, ScopedObject<'input>>,
}

impl<'input> VM<'input> {
    pub fn new(program: Program<'input>) -> Self {
        Self {
            stack: Stack::default(),
            program,
            symbols: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<'input, Object<'input>> {
        let mut i = 0;
        while let Some(op) = self.program.ops.get(i) {
            match op {
                Op::Constant(value) => {
                    self.stack.push(self.program.constants[*value].clone());
                }
                Op::Pop => {
                    self.stack.pop();
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Eq | Op::Neq | Op::GT => {
                    self.execute_binary_op((*op).try_into().unwrap())?
                }
                Op::True(span) => self.stack.push(SpannedObject {
                    object: Object::Boolean(true),
                    span: *span,
                }),
                Op::False(span) => self.stack.push(SpannedObject {
                    object: Object::Boolean(false),
                    span: *span,
                }),
                Op::Neg(span) => {
                    let span = *span;
                    let value = self.pop()?;
                    let Object::Integer(value) = value.object else {
                        return Err(vec![Error::Eval(EvalError {
                            span: value.span,
                            kind: ErrorKind::InvalidOperand {
                                operator: PrefixOperator::Neg,
                                operand: value.object.into(),
                            },
                        })]);
                    };

                    self.stack.push(SpannedObject {
                        object: (-value).into(),
                        span,
                    });
                }
                Op::Not(span) => {
                    let span = *span;
                    let value = self.pop()?;
                    self.stack.push(SpannedObject {
                        object: Object::Boolean(!value.object.truthy()),
                        span,
                    })
                }
                Op::JumpIfNot(index) => {
                    let index = *index;
                    let value = self.pop()?;
                    if !value.object.truthy() {
                        i = index - 1;
                    }
                }
                Op::Jump(index) => {
                    i = *index - 1;
                }
                Op::Panic => {
                    panic!("Panicked! Why?");
                }
                Op::Null(span) => self.stack.push(SpannedObject {
                    object: Object::Null,
                    span: *span,
                }),
                Op::SetGlobal(name) => {
                    let name = *name;
                    let value = self.pop()?;
                    self.symbols.insert(
                        name,
                        ScopedObject {
                            object: value,
                            scope: Scope::Global,
                        },
                    );
                }
                Op::GetGlobal { name, span } => {
                    let Some(value) = self.symbols.get(name) else {
                        return Err(vec![Error::Eval(EvalError {
                            span: *span,
                            kind: ErrorKind::UndefinedVariable(name),
                        })]);
                    };
                    self.stack.push(value.object.clone());
                }
            }

            i += 1;
        }

        Ok(self.stack.pop().map(|o| o.object).unwrap_or(Object::Null))
    }

    fn execute_binary_op(&mut self, operator: InfixOperator) -> Result<'input, ()> {
        let right = self.pop()?;
        let left = self.pop()?;
        let span = left.span.join(right.span);
        let operator = if left.span.start > right.span.end {
            InfixOperator::LT
        } else {
            operator
        };
        match (operator, left.object, right.object) {
            (InfixOperator::Add, Object::Integer(left), Object::Integer(right)) => {
                self.stack.push(SpannedObject {
                    object: (left + right).into(),
                    span,
                });
            }
            (InfixOperator::Add, Object::String(left), Object::String(right)) => {
                self.stack.push(SpannedObject {
                    object: Object::String(left.clone() + &right),
                    span,
                });
            }
            (InfixOperator::Sub, Object::Integer(left), Object::Integer(right)) => {
                self.stack.push(SpannedObject {
                    object: (left - right).into(),
                    span,
                });
            }
            (InfixOperator::Mul, Object::Integer(left), Object::Integer(right)) => {
                self.stack.push(SpannedObject {
                    object: (left * right).into(),
                    span,
                });
            }
            (InfixOperator::Div, Object::Integer(left), Object::Integer(right)) => {
                if right == 0 {
                    return Err(vec![Error::Eval(EvalError {
                        span,
                        kind: ErrorKind::DivisionByZero,
                    })]);
                }

                self.stack.push(SpannedObject {
                    object: (left / right).into(),
                    span,
                });
            }
            (InfixOperator::Eq, left, right)
                if !matches!(&left, Object::Function { .. })
                    && !matches!(&right, Object::Function { .. }) =>
            {
                self.stack.push(SpannedObject {
                    object: (left == right).into(),
                    span,
                });
            }
            (InfixOperator::Neq, left, right)
                if !matches!(&left, Object::Function { .. })
                    && !matches!(&right, Object::Function { .. }) =>
            {
                self.stack.push(SpannedObject {
                    object: (left != right).into(),
                    span,
                });
            }
            (
                InfixOperator::GT | InfixOperator::LT,
                Object::Integer(left),
                Object::Integer(right),
            ) => {
                self.stack.push(SpannedObject {
                    object: (left > right).into(),
                    span,
                });
            }
            (_, left, right) => {
                return invalid_operands(operator, left, right, span).map(|_| ());
            }
        }

        Ok(())
    }

    fn pop(&mut self) -> Result<'input, SpannedObject<'input>> {
        self.stack.pop().ok_or(vec![Error::Underflow])
    }
}

fn invalid_operands<'a>(
    operator: InfixOperator,
    left: Object<'a>,
    right: Object<'a>,
    span: Span,
) -> Result<'a, Rc<Object<'a>>> {
    Err(vec![Error::Eval(EvalError {
        span,
        kind: ErrorKind::InvalidOperands {
            operator,
            left: left.into(),
            right: right.into(),
        },
    })])
}
