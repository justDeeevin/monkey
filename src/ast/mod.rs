pub mod traits;
use traits::*;

use crate::token::Token;

use std::rc::Rc;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
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
pub struct LetStatement<E: Expression> {
    pub token: Token,
    pub name: Identifier,
    pub value: E,
}

impl<E: Expression> Node for LetStatement<E> {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
#[cfg(test)]
impl<E: Expression + 'static> Statement for LetStatement<E> {}
#[cfg(not(test))]
impl<E: Expression> Statement for LetStatement<E> {}

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

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for Identifier {}
