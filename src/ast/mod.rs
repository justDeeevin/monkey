pub mod traits;
use traits::*;

use crate::token::{Token, TokenKind};

use std::{fmt::Display, rc::Rc};

// This seems good for now.
pub type Integer = i64;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Program {
    pub fn new(statements: Vec<Box<dyn Statement>>) -> Self {
        Self { statements }
    }
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

#[derive(Debug)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Box<dyn Expression>,
}

impl Display for ExpressionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};", self.expression)
    }
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
impl Statement for ExpressionStatement {}

#[derive(Debug)]
pub struct IntegerLiteral {
    token: Token,
    value: Integer,
}

impl IntegerLiteral {
    pub fn new(value: Integer) -> Self {
        let token = Token {
            kind: TokenKind::Int,
            literal: value.to_string().into(),
        };
        Self { token, value }
    }

    pub fn value(&self) -> Integer {
        self.value
    }

    pub fn from_token(token: Token) -> Result<Self, std::num::ParseIntError> {
        let value = token.literal.parse()?;
        Ok(Self { token, value })
    }
}

impl Display for IntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for IntegerLiteral {}

#[derive(Debug)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: char,
    pub right: Box<dyn Expression>,
}

impl Display for PrefixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}{})", self.operator, self.right)
    }
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for PrefixExpression {}

#[derive(Debug)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<dyn Expression>,
    pub operator: Rc<str>,
    pub right: Box<dyn Expression>,
}

impl Display for InfixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)
    }
}

impl Node for InfixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for InfixExpression {}
