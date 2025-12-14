use crate::{object::Object, token::Span};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    /// Push a constant at the given index onto the stack.
    Constant(usize),
    Add,
}

#[derive(Debug, Clone)]
pub struct SpannedObject<'a> {
    pub object: Rc<Object<'a>>,
    pub span: Span,
}

impl PartialEq for SpannedObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl PartialEq<Object<'_>> for SpannedObject<'_> {
    fn eq(&self, other: &Object<'_>) -> bool {
        self.object.as_ref() == other
    }
}

#[derive(Debug)]
pub struct Program<'a> {
    pub ops: Rc<[Op]>,
    pub constants: Rc<[SpannedObject<'a>]>,
}
