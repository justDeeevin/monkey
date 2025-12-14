use crate::{
    ast::InfixOperator,
    code::{Op, Program, SpannedObject},
    object::Object,
    token::Span,
};
use std::{mem::MaybeUninit, rc::Rc};

#[cfg(test)]
mod test;

struct Stack<T, const SIZE: usize = 2048> {
    data: [MaybeUninit<T>; SIZE],
    top: usize,
}

impl<T: std::fmt::Debug, const SIZE: usize> std::fmt::Debug for Stack<T, SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self.data
                    .iter()
                    .take(self.top)
                    .map(|x| unsafe { x.assume_init_ref() }),
            )
            .finish()
    }
}

impl<T, const SIZE: usize> Default for Stack<T, SIZE> {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| MaybeUninit::uninit()),
            top: 0,
        }
    }
}

impl<T, const SIZE: usize> Stack<T, SIZE> {
    pub fn push(&mut self, value: T) {
        self.data[self.top] = MaybeUninit::new(value);
        self.top += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.top == 0 {
            None
        } else {
            self.top -= 1;
            // SAFETY: We just checked that the top is not out of bounds.
            Some(unsafe { self.data[self.top].assume_init_read() })
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.top == 0 {
            None
        } else {
            // SAFETY: Top is always 1 greater than
            Some(unsafe { self.data[self.top - 1].assume_init_ref() })
        }
    }
}

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
    program: Program<'input>,
}

impl<'input> VM<'input> {
    pub fn new(program: Program<'input>) -> Self {
        Self {
            stack: Stack::default(),
            program,
        }
    }

    pub fn run(mut self) -> Result<'input, Rc<Object<'input>>> {
        let mut i = 0;
        while let Some(op) = self.program.ops.get(i) {
            match op {
                Op::Constant(value) => {
                    self.stack.push(self.program.constants[*value].clone());
                }
                Op::Add => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let span = Span {
                        start: left.span.start,
                        end: right.span.end,
                    };
                    match (left.object.as_ref(), right.object.as_ref()) {
                        (Object::Integer(left), Object::Integer(right)) => {
                            self.stack.push(SpannedObject {
                                object: Rc::new(Object::Integer(left + right)),
                                span,
                            });
                        }
                        (Object::String(left), Object::String(right)) => {
                            self.stack.push(SpannedObject {
                                object: Rc::new(Object::String(left.clone() + right)),
                                span,
                            });
                        }
                        (left, right) => {
                            return Err(vec![Error::Eval(crate::eval::Error {
                                span,
                                kind: crate::eval::ErrorKind::InvalidOperands {
                                    operator: InfixOperator::Add,
                                    left: left.into(),
                                    right: right.into(),
                                },
                            })]);
                        }
                    }
                }
            }

            i += 1;
        }

        Ok(self
            .stack
            .pop()
            .map(|o| o.object)
            .unwrap_or(Rc::new(Object::Null)))
    }

    fn pop(&mut self) -> Result<'input, SpannedObject<'input>> {
        self.stack.pop().ok_or(vec![Error::Underflow])
    }
}
