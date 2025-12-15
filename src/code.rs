use crate::{ast::InfixOperator, object::Object, token::Span};
use std::rc::Rc;

#[derive(Debug, strum::EnumDiscriminants, Clone, Copy)]
#[strum_discriminants(name(OpKind))]
pub enum Op {
    /// Push a constant at the given index onto the stack.
    Constant(usize),
    True(Span),
    False(Span),
    Add,
    Sub,
    Mul,
    Div,
    Pop,
    Eq,
    Neq,
    GT,
    Neg(Span),
    Not(Span),
    JumpIfNot(usize),
    Jump(usize),
    Panic,
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        OpKind::from(self) == OpKind::from(other)
    }
}

impl Eq for Op {}

impl From<InfixOperator> for Op {
    fn from(value: InfixOperator) -> Self {
        match value {
            InfixOperator::Add => Op::Add,
            InfixOperator::Sub => Op::Sub,
            InfixOperator::Mul => Op::Mul,
            InfixOperator::Div => Op::Div,
            InfixOperator::Eq => Op::Eq,
            InfixOperator::Neq => Op::Neq,
            InfixOperator::LT | InfixOperator::GT => Op::GT,
        }
    }
}

impl TryFrom<Op> for InfixOperator {
    type Error = ();

    fn try_from(value: Op) -> Result<Self, Self::Error> {
        Ok(match value {
            Op::Add => InfixOperator::Add,
            Op::Sub => InfixOperator::Sub,
            Op::Mul => InfixOperator::Mul,
            Op::Div => InfixOperator::Div,
            Op::Eq => InfixOperator::Eq,
            Op::Neq => InfixOperator::Neq,
            Op::GT => InfixOperator::GT,
            _ => return Err(()),
        })
    }
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
