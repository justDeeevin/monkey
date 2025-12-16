use crate::{code::Op, object::CompiledFunction, token::Span};
use std::rc::Rc;

pub struct Frame<'a> {
    function: Rc<CompiledFunction<'a>>,
    pub ip: usize,
    pub call_span: Span,
}

impl<'a> Frame<'a> {
    pub fn new(function: Rc<CompiledFunction<'a>>, call_span: Span) -> Self {
        Self {
            function,
            ip: 0,
            call_span,
        }
    }

    pub fn ops(&self) -> &[Op<'a>] {
        &self.function.ops
    }
}
