pub mod traits;
use traits::*;

use crate::token::Token;

use std::{fmt::Display, rc::Rc};

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            write!(f, "{statement}")?;
        }
        Ok(())
    }
}

impl Node for Program {
    fn token_literal(&self) -> &str {
        self.statements
            .first()
            .map(|s| s.token_literal())
            .unwrap_or("")
    }
}

#[derive(Debug)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<dyn Expression>,
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {};", self.name, self.value)
    }
}

impl Node for LetStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
impl Statement for LetStatement {}

#[derive(Debug)]
pub struct Identifier {
    token: Token,
    value: Rc<str>,
}

impl Identifier {
    pub fn new(literal: impl AsRef<str>) -> Self {
        let token = Token::word(&literal);
        Self {
            token,
            value: literal.as_ref().into(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn from_token(token: Token) -> Self {
        Self {
            value: token.literal.clone(),
            token,
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
impl Expression for Identifier {}

#[derive(Debug)]
pub struct ReturnStatement {
    pub token: Token,
    pub value: Box<dyn Expression>,
}

impl Display for ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {};", self.value)
    }
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
impl Statement for ReturnStatement {}
