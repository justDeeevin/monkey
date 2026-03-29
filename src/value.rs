use crate::ast::{Block, Identifier};
use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};
use strum::{Display, EnumDiscriminants};

#[derive(Clone, EnumDiscriminants)]
#[strum_discriminants(name(Type), derive(Display))]
pub enum Value<'a> {
    Int(i64),
    Bool(bool),
    String(String),
    Array(Vec<Self>),
    Map(HashMap<Self, Self>),
    Null,
    Function(Rc<Function<'a>>),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => i.fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Array(a) => f.debug_list().entries(a.iter().map(DebugDisplay)).finish(),
            Self::Map(m) => f
                .debug_map()
                .entries(m.iter().map(|(k, v)| (DebugDisplay(k), DebugDisplay(v))))
                .finish(),
            Self::Null => write!(f, "null"),
            Self::Function(_) => write!(f, "<function>"),
        }
    }
}

struct DebugDisplay<'a, T: Display>(&'a T);

impl<T: Display> std::fmt::Debug for DebugDisplay<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Value<'_> {
    pub fn truthy(&self) -> bool {
        match self {
            Self::Int(1..)
            | Self::Bool(true)
            | Self::Array(_)
            | Self::Map(_)
            | Self::Function(_) => true,
            Self::String(s) => !s.is_empty(),
            _ => false,
        }
    }
}

pub struct Function<'a> {
    pub name: Option<Identifier<'a>>,
    pub parameters: Vec<Identifier<'a>>,
    pub body: Block<'a>,
}

impl Hash for Value<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(i) => i.hash(state),
            Self::Bool(b) => b.hash(state),
            Self::String(s) => s.hash(state),
            Self::Array(_) | Self::Map(_) | Self::Function(_) | Self::Null => {
                panic!("map key must be int, bool, or string")
            }
        }
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l), Self::Int(r)) => l == r,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Array(l), Self::Array(r)) => l == r,
            (Self::Map(l), Self::Map(r)) => l == r,
            (Self::Null, Self::Null) => true,
            _ => false,
        }
    }
}

impl Eq for Value<'_> {}
