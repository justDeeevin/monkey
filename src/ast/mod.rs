pub mod traits;

use traits::*;

use crate::{
    parser::ParseError,
    token::{Token, TokenKind},
};

use std::{fmt::Display, rc::Rc};

// This seems good for now.
pub type Integer = i64;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

impl TryFrom<Token> for Identifier {
    type Error = ParseError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        if token.kind != TokenKind::Ident {
            return Err(ParseError::Unexpected {
                given: token,
                expected: TokenKind::Ident,
            });
        }

        Ok(Self {
            value: token.literal.clone(),
            token,
        })
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

#[derive(Debug, thiserror::Error)]
pub enum IntegerLiteralConstructionError {
    #[error("Failed to parse int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Given Token was not of kind Int")]
    NonInt,
}

impl TryFrom<Token> for IntegerLiteral {
    type Error = IntegerLiteralConstructionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.kind != TokenKind::Int {
            return Err(IntegerLiteralConstructionError::NonInt);
        }

        Ok(Self {
            value: value.literal.parse()?,
            token: value,
        })
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    token: Token,
    value: bool,
}

impl BooleanLiteral {
    pub fn new(value: bool) -> Self {
        Self {
            token: Token::word(value.to_string()),
            value,
        }
    }

    pub fn value(&self) -> bool {
        self.value
    }
}

impl TryFrom<Token> for BooleanLiteral {
    type Error = ParseError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let value = match &token.kind {
            TokenKind::True => true,
            TokenKind::False => false,
            _ => {
                return Err(ParseError::Unexpected {
                    given: token,
                    expected: TokenKind::True,
                });
            }
        };

        Ok(Self { token, value })
    }
}

impl Display for BooleanLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token.literal)
    }
}

impl Node for BooleanLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for BooleanLiteral {}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub token: Token,
    pub cond: Box<dyn Expression>,
    pub cons: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.cond, self.cons)?;

        if let Some(alternative) = &self.alternative {
            write!(f, " else {alternative}")?;
        }

        Ok(())
    }
}

impl Node for IfExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for IfExpression {}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Box<dyn Statement>>,
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for statement in &self.statements {
            write!(f, "{statement}")?;
        }
        write!(f, " }}")
    }
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Statement for BlockStatement {}

#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl Display for FunctionLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.token_literal())?;
        for param in self.parameters.iter().take(self.parameters.len() - 1) {
            write!(f, "{param}, ")?;
        }
        write!(
            f,
            "{}) {}",
            self.parameters
                .last()
                .map(|i| i.to_string())
                .unwrap_or_default(),
            self.body
        )
    }
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for FunctionLiteral {}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<dyn Expression>,
    pub arguments: Vec<Box<dyn Expression>>,
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.function)?;
        for arg in self.arguments.iter().take(self.arguments.len() - 1) {
            write!(f, "{arg}, ")?;
        }
        write!(
            f,
            "{})",
            self.arguments
                .last()
                .map(|e| e.to_string())
                .unwrap_or_default()
        )
    }
}

impl Node for CallExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
impl Expression for CallExpression {}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    token: Token,
    value: Rc<str>,
}

impl StringLiteral {
    pub fn new(value: Rc<str>) -> Self {
        Self {
            token: Token::word(value.as_ref()),
            value,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn rc(&self) -> Rc<str> {
        self.value.clone()
    }
}

impl TryFrom<Token> for StringLiteral {
    type Error = ParseError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        if token.kind != TokenKind::String {
            return Err(ParseError::Unexpected {
                given: token,
                expected: TokenKind::String,
            });
        }

        Ok(Self {
            value: token.literal.clone(),
            token,
        })
    }
}

impl Display for StringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl Node for StringLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

impl Expression for StringLiteral {}
