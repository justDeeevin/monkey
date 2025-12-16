use crate::{
    ast::{BlockStatement, Identifier},
    code::Op,
    eval::Environment,
};
use std::{collections::HashMap, hash::Hash, rc::Rc};

#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(ObjectKind), derive(strum::Display))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Object<'a> {
    Integer(i64),
    Boolean(bool),
    #[strum_discriminants(strum(serialize = "Null"))]
    Return(Box<Self>),
    Function(Rc<Function<'a>>),
    CompiledFunction(Rc<CompiledFunction<'a>>),
    String(String),
    // Null could instead be represented as `Option::<Object<'a>>::None`, but that would require
    // the added `Option` tag. This saves space.
    Null,
    Array(Vec<Self>),
    Map(Map<'a>),
}

impl From<i64> for Object<'_> {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<bool> for Object<'_> {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<String> for Object<'_> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Object<'_> {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<'a> From<Vec<Object<'a>>> for Object<'a> {
    fn from(a: Vec<Object<'a>>) -> Self {
        Self::Array(a)
    }
}

impl<'a> FromIterator<Object<'a>> for Object<'a> {
    fn from_iter<T: IntoIterator<Item = Object<'a>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl<'a> From<HashMap<Object<'a>, Object<'a>>> for Object<'a> {
    fn from(m: HashMap<Object<'a>, Object<'a>>) -> Self {
        Self::Map(Map(m))
    }
}

impl<'a> FromIterator<(Object<'a>, Object<'a>)> for Object<'a> {
    fn from_iter<T: IntoIterator<Item = (Object<'a>, Object<'a>)>>(iter: T) -> Self {
        Self::Map(Map(iter.into_iter().collect()))
    }
}

impl<'a> From<CompiledFunction<'a>> for Object<'a> {
    fn from(value: CompiledFunction<'a>) -> Self {
        Self::CompiledFunction(Rc::new(value))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Map<'a>(pub HashMap<Object<'a>, Object<'a>>);

impl<'a> std::ops::Deref for Map<'a> {
    type Target = HashMap<Object<'a>, Object<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Map<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for Map<'_> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unreachable!()
    }
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

impl Hash for Function<'_> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct CompiledFunction<'a> {
    pub ops: Rc<[Op<'a>]>,
    pub params: Rc<[&'a str]>,
}

impl PartialEq for CompiledFunction<'_> {
    fn eq(&self, _other: &Self) -> bool {
        unreachable!()
    }
}

impl Eq for CompiledFunction<'_> {}

impl Hash for CompiledFunction<'_> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unreachable!()
    }
}

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
            Self::Array(a) => {
                write!(f, "[")?;
                if !a.is_empty() {
                    for element in a.iter().take(a.len() - 1) {
                        write!(f, "{}, ", element)?;
                    }
                    write!(f, "{}", a.last().unwrap())?;
                }
                write!(f, "]")
            }
            Self::Map(m) => {
                write!(f, "{{")?;
                if !m.is_empty() {
                    for (key, value) in m.iter().take(m.len() - 1) {
                        write!(f, "{key}: {value}, ")?;
                    }
                    let (key, value) = m.iter().last().unwrap();
                    write!(f, "{key}: {value}")?;
                }
                write!(f, "}}")
            }
            Self::CompiledFunction(fun) => f.debug_list().entries(fun.ops.as_ref()).finish(),
        }
    }
}
