use std::rc::Rc;

use crate::{
    ast::{BlockStatement, Identifier},
    eval::Environment,
};

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(ObjectKind), derive(strum::Display))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object<'a> {
    Integer(i64),
    Boolean(bool),
    #[strum_discriminants(strum(serialize = "Null"))]
    Return(Box<Self>),
    Function(Rc<Function<'a>>),
    String(String),
    Null,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub this: Option<Identifier<'a>>,
    pub parameters: Vec<Identifier<'a>>,
    pub body: BlockStatement<'a>,
    pub env: Environment<'a>,
}

impl PartialEq for Function<'_> {
    fn eq(&self, _other: &Self) -> bool {
        unreachable!()
    }
}

impl Eq for Function<'_> {}

impl Object<'_> {
    pub fn truthy(&self) -> bool {
        !matches!(self, Object::Null | Object::Boolean(false))
    }
}

impl std::fmt::Display for Object<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => i.fmt(f),
            Self::Boolean(b) => b.fmt(f),
            Self::Null | Object::Return(_) => write!(f, "null"),
            Self::Function(function) => {
                write!(f, "<function")?;
                if let Some(this) = &function.this {
                    write!(f, " {this}")?;
                }
                write!(f, ">")
            }
            Self::String(s) => s.fmt(f),
        }
    }
}
