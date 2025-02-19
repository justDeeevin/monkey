pub mod traits;

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    rc::Rc,
};

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
    fn type_name(&self) -> &'static str {
        "integer"
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
    fn type_name(&self) -> &'static str {
        "boolean"
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
    fn type_name(&self) -> &'static str {
        "null"
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
    fn type_name(&self) -> &'static str {
        self.value.type_name()
    }
}

#[derive(Debug, Clone)]
pub struct Scope(HashMap<Rc<str>, Box<dyn Object>>);

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        let mut hash = HashMap::new();
        hash.extend(crate::eval::intrinsics().0);
        Self(hash)
    }

    pub fn empty() -> Self {
        Self(HashMap::new())
    }
}

impl Deref for Scope {
    type Target = HashMap<Rc<str>, Box<dyn Object>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
    fn type_name(&self) -> &'static str {
        "function"
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
    fn type_name(&self) -> &'static str {
        "string"
    }
}

pub type IntrinsicFn = fn(&[Box<dyn Object>]) -> crate::eval::Result<Box<dyn Object>>;

#[derive(Debug, Clone)]
pub struct Intrinsic {
    pub function: IntrinsicFn,
}

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<intrinsic>")
    }
}

impl Object for Intrinsic {
    fn truthy(&self) -> bool {
        true
    }
    fn type_name(&self) -> &'static str {
        "function"
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Box<dyn Object>>,
}

impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for element in self.elements.iter().take(self.elements.len() - 1) {
            write!(f, "{element}, ")?;
        }
        write!(f, "{}]", self.elements.last().unwrap())
    }
}

impl Object for Array {
    fn truthy(&self) -> bool {
        true
    }
    fn type_name(&self) -> &'static str {
        "array"
    }
}
