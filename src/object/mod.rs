pub mod traits;

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::ast::{BlockStatement, Identifier, Integer as Int};
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

pub type Scope = HashMap<Rc<str>, Box<dyn Object>>;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<Identifier>,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub scope: Scope,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn(")?;
        for param in self.parameters.iter().take(self.parameters.len() - 1) {
            write!(f, "{param}, ")?;
        }

        write!(
            f,
            "{}) {{\n {}\n}}",
            self.parameters
                .last()
                .map(|i| i.to_string())
                .unwrap_or_default(),
            self.body,
        )
    }
}

impl Object for Function {
    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct String {
    pub value: Rc<str>,
}

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl Object for String {
    fn truthy(&self) -> bool {
        true
    }
}
