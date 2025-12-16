use crate::{
    ast::{InfixOperator, PrefixOperator},
    code::{Op, Program, SpannedObject},
    eval::{Error as EvalError, ErrorKind},
    object::{CompiledFunction, Object},
    token::Span,
};
use frame::Frame;
use itertools::Itertools;
use stack::Stack;
use std::{collections::HashMap, rc::Rc};

pub mod frame;
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

pub struct VM<'input> {
    stack: Stack<SpannedObject<'input>>,
    pub frames: Vec<Frame<'input>>,
    pub constants: Rc<[SpannedObject<'input>]>,
}

impl Default for VM<'_> {
    fn default() -> Self {
        Self::new(Program::default())
    }
}

impl<'input> VM<'input> {
    pub fn new(program: Program<'input>) -> Self {
        Self {
            stack: Stack::default(),
            frames: vec![Frame::new(
                Rc::new(CompiledFunction {
                    ops: program.ops,
                    params: Rc::new([]),
                }),
                Span::default(),
            )],
            constants: program.constants,
        }
    }

    fn current_frame(&self) -> &Frame<'input> {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut Frame<'input> {
        self.frames.last_mut().unwrap()
    }

    fn bind(&mut self, name: &'input str, value: SpannedObject<'input>) {
        self.current_frame_mut().locals.insert(name, value);
    }

    pub fn run(&mut self) -> Result<'input, Object<'input>> {
        while let Some(op) = self
            .current_frame()
            .ops()
            .get(self.current_frame().ip)
            .copied()
        {
            match op {
                Op::Constant(value) => {
                    self.stack.push(self.constants[value].clone());
                }
                Op::Pop => {
                    self.stack.pop();
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Eq | Op::Neq | Op::GT => {
                    self.execute_binary_op((op).try_into().unwrap())?
                }
                Op::True(span) => self.stack.push(SpannedObject {
                    object: Object::Boolean(true),
                    span,
                }),
                Op::False(span) => self.stack.push(SpannedObject {
                    object: Object::Boolean(false),
                    span,
                }),
                Op::Neg(span) => {
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
                    let value = self.pop()?;
                    self.stack.push(SpannedObject {
                        object: (!value.object.truthy()).into(),
                        span,
                    })
                }
                Op::JumpIfNot(index) => {
                    let value = self.pop()?;
                    if !value.object.truthy() {
                        self.current_frame_mut().ip = index - 1;
                    }
                }
                Op::Jump(index) => {
                    self.current_frame_mut().ip = index - 1;
                }
                Op::Panic => {
                    panic!("Panicked! Why?");
                }
                Op::Null(span) => self.stack.push(SpannedObject {
                    object: Object::Null,
                    span,
                }),
                Op::Bind(name) => {
                    let value = self.pop()?;
                    self.bind(name, value);
                }
                Op::Get { name, span } => {
                    let Some(value) = self.current_frame().locals.get(name) else {
                        return Err(vec![Error::Eval(EvalError {
                            span,
                            kind: ErrorKind::UndefinedVariable(name),
                        })]);
                    };
                    self.stack.push(SpannedObject {
                        object: value.object.clone(),
                        span,
                    });
                }
                Op::Array { size, span } => {
                    let object = self.stack.drain(size).map(|o| o.object).collect();
                    self.stack.push(SpannedObject { object, span })
                }
                Op::Map { size, span } => {
                    let map = self
                        .stack
                        .drain(size * 2)
                        .map(|o| o.object)
                        .tuples::<(_, _)>()
                        .collect::<HashMap<_, _>>();
                    if map.len() != size {
                        return Err(vec![Error::Underflow]);
                    }
                    self.stack.push(SpannedObject {
                        object: map.into(),
                        span,
                    });
                }
                Op::Index(span) => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    let object = match collection.object {
                        Object::Array(mut array) => {
                            let index_span = index.span;

                            let Object::Integer(index) = index.object else {
                                return Err(vec![Error::Eval(EvalError {
                                    span: index_span,
                                    kind: ErrorKind::NotAnIndex,
                                })]);
                            };

                            if index < 0 || index >= array.len() as i64 {
                                return Err(vec![Error::Eval(EvalError {
                                    span: index_span,
                                    kind: ErrorKind::OutOfBounds {
                                        i: index,
                                        len: array.len(),
                                    },
                                })]);
                            }

                            array.remove(index as usize)
                        }
                        Object::Map(mut map) => {
                            if matches!(index.object, Object::CompiledFunction(_) | Object::Map(_))
                            {
                                return Err(vec![Error::Eval(EvalError {
                                    span: index.span,
                                    kind: ErrorKind::InvalidKey(index.object.into()),
                                })]);
                            }

                            map.remove(&index.object).unwrap_or(Object::Null)
                        }
                        _ => {
                            return Err(vec![Error::Eval(EvalError {
                                span: collection.span,
                                kind: ErrorKind::NotACollection,
                            })]);
                        }
                    };

                    self.stack.push(SpannedObject { span, object });
                }
                Op::Call {
                    call_span,
                    args_span,
                } => {
                    let function = self.pop()?;
                    let Object::CompiledFunction(function) = function.object else {
                        return Err(vec![Error::Eval(EvalError {
                            span: function.span,
                            kind: ErrorKind::NotAFunction,
                        })]);
                    };
                    let params = function.params.clone();
                    let args = self.stack.drain(params.len()).collect::<Vec<_>>();
                    if args.len() != params.len() {
                        return Err(vec![Error::Eval(EvalError {
                            span: args_span,
                            kind: ErrorKind::WrongNumberOfArguments {
                                expected: params.len(),
                                got: args.len(),
                            },
                        })]);
                    }
                    self.frames.push(Frame::new(function, call_span));
                    for (param, value) in params.iter().rev().zip(args) {
                        self.bind(param, value);
                    }
                    continue;
                }
                Op::ReturnValue => {
                    let value = self.pop()?.object;
                    let Some(call_span) = self.frames.pop().map(|frame| frame.call_span) else {
                        return Err(vec![Error::Underflow]);
                    };
                    self.stack.push(SpannedObject {
                        span: call_span,
                        object: value,
                    });
                }
                Op::Return => {
                    let Some(call_span) = self.frames.pop().map(|frame| frame.call_span) else {
                        return Err(vec![Error::Underflow]);
                    };
                    self.stack.push(SpannedObject {
                        object: Object::Null,
                        span: call_span,
                    });
                }
            }

            self.current_frame_mut().ip += 1;
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
                if !matches!(&left, Object::CompiledFunction(_))
                    && !matches!(&right, Object::CompiledFunction(_)) =>
            {
                self.stack.push(SpannedObject {
                    object: (left == right).into(),
                    span,
                });
            }
            (InfixOperator::Neq, left, right)
                if !matches!(&left, Object::CompiledFunction(_))
                    && !matches!(&right, Object::CompiledFunction(_)) =>
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
