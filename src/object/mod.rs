pub mod traits;

use std::fmt::Display;

use crate::ast::Integer as Int;
use traits::Object;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
