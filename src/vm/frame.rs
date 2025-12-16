use crate::{
    code::{Op, SpannedObject},
    object::CompiledFunction,
    token::Span,
};
use std::{collections::HashMap, rc::Rc};

pub struct Frame<'a> {
    pub function: Rc<CompiledFunction<'a>>,
    pub ip: usize,
    pub call_span: Span,
    pub locals: HashMap<&'a str, SpannedObject<'a>>,
}

impl<'a> Frame<'a> {
    pub fn new(function: Rc<CompiledFunction<'a>>, call_span: Span) -> Self {
        Self {
            function,
            ip: 0,
            call_span,
            locals: HashMap::new(),
        }
    }

    pub fn ops(&self) -> &[Op<'a>] {
        &self.function.ops
    }
}
