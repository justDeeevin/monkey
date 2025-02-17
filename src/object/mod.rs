pub mod traits;

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::ast::Integer as Int;
use traits::Object;

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: Int,
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Object for Integer {
    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub value: bool,
}

impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Object for Boolean {
    fn truthy(&self) -> bool {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Null;

impl Display for Null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

impl Object for Null {
    fn truthy(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct ReturnValue {
    pub value: Box<dyn Object>,
}

impl Display for ReturnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Object for ReturnValue {
    fn truthy(&self) -> bool {
        self.value.truthy()
    }
}

#[derive(Default, Clone)]
pub struct Environment(HashMap<Rc<str>, Box<dyn Object>>);

impl std::ops::Deref for Environment {
    type Target = HashMap<Rc<str>, Box<dyn Object>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Environment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
