pub mod test;

use std::{collections::HashMap, str::FromStr};

use crate::{
    ast::{
        ArrayLiteral, BlockStatement, BooleanLiteral, CallExpression, ExpressionStatement,
        FunctionLiteral, HashLiteral, Identifier, IfExpression, IndexExpression, InfixExpression,
        IntegerLiteral, IntegerLiteralConstructionError, LetStatement, PrefixExpression, Program,
        ReturnStatement, StringLiteral,
        traits::{Expression, Statement},
    },
    lexer::Lexer,
    token::{
        Token,
        TokenKind::{self, *},
    },
};

pub struct Parser {
    lexer: Lexer,
    current: Option<Token>,
    peek: Option<Token>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut out = Self {
            lexer,
            current: None,
            peek: None,
        };

        out.next_token();
        out.next_token();
        out
    }

    fn next_token(&mut self) {
        self.current = self.peek.take();
        self.peek = self.lexer.next();
    }

    fn current_clone(&self) -> Result<Token, ParseError> {
        self.current.clone().ok_or(ParseError::Eof)
    }

    fn current_ref(&self) -> Result<&Token, ParseError> {
        self.current.as_ref().ok_or(ParseError::Eof)
    }

    fn peek_clone(&self) -> Result<Token, ParseError> {
        self.peek.clone().ok_or(ParseError::Eof)
    }

    fn peek_ref(&self) -> Result<&Token, ParseError> {
        self.peek.as_ref().ok_or(ParseError::Eof)
    }

    fn parse_statement(&mut self) -> Result<Box<dyn Statement>, ParseError> {
        let current = self.current_clone()?;
        Ok(match current.kind {
            Let => Box::new(self.parse_let_statement()?),
            Return => Box::new(self.parse_return_statement()?),
            _ => Box::new(self.parse_expression_statement()?),
        })
    }

    fn parse_expression_statement(&mut self) -> Result<ExpressionStatement, ParseError> {
        let out = ExpressionStatement {
            token: self.current_clone()?,
            expression: self.parse_expression(ExpressionKind::Lowest)?,
        };
        if self.peek.as_ref().map(|t| t.kind) == Some(Semi) {
            self.next_token();
        }
        Ok(out)
    }

    fn parse_expression(
        &mut self,
        kind: ExpressionKind,
    ) -> Result<Box<dyn Expression>, ParseError> {
        let Some(mut left) = self.parse_prefix()? else {
            return Err(ParseError::NoPrefix);
        };

        while let Some(peek) = &self.peek {
            if peek.kind != Semi && kind < peek.kind.into() {
                self.next_token();
                left = self.parse_infix(left)?;
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, ParseError> {
        let token = self.current_clone()?;

        self.expect_peek(Ident)?;

        let name = Identifier::try_from(self.current_clone()?).unwrap();

        self.expect_peek(Assign)?;

        self.next_token();

        let value = self.parse_expression(ExpressionKind::Lowest)?;

        if self.peek.as_ref().map(|t| t.kind) == Some(Semi) {
            self.next_token();
        }

        Ok(LetStatement { name, value, token })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, ParseError> {
        let token = self.current_clone()?;

        self.next_token();
        let value = self.parse_expression(ExpressionKind::Lowest)?;

        if self.peek.as_ref().map(|t| t.kind) == Some(Semi) {
            self.next_token();
        }

        Ok(ReturnStatement { token, value })
    }

    fn expect_peek(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let peek = self.peek_clone()?;
        if peek.kind != expected {
            return Err(ParseError::Unexpected {
                given: peek,
                expected,
            });
        }
        self.next_token();
        Ok(())
    }

    fn parse_prefix(&mut self) -> Result<Option<Box<dyn Expression>>, ParseError> {
        let current = self.current_clone()?;
        Ok(match current.kind {
            Ident => Some(Box::new(Identifier::try_from(current).unwrap())),
            Int => match IntegerLiteral::try_from(current) {
                Ok(lit) => Some(Box::new(lit)),
                Err(IntegerLiteralConstructionError::NonInt) => unreachable!(),
                Err(IntegerLiteralConstructionError::ParseInt(e)) => return Err(e.into()),
            },
            Not | Minus => {
                self.next_token();
                Some(Box::new(PrefixExpression {
                    operator: current.kind.to_string().chars().next().unwrap(),
                    token: current,
                    right: self.parse_expression(ExpressionKind::Prefix)?,
                }))
            }
            True | False => Some(Box::new(BooleanLiteral::try_from(current)?)),
            LParen => {
                self.next_token();
                let exp = self.parse_expression(ExpressionKind::Lowest)?;
                self.expect_peek(RParen)?;
                Some(exp)
            }
            If => {
                let token = self.current_clone()?;
                self.expect_peek(LParen)?;

                self.next_token();
                let cond = self.parse_expression(ExpressionKind::Lowest)?;
                self.expect_peek(RParen)?;
                self.expect_peek(LBrace)?;

                let cons = self.parse_block_statement()?;

                let alternative = if self.peek.as_ref().map(|t| t.kind) == Some(Else) {
                    self.next_token();

                    self.expect_peek(LBrace)?;

                    Some(self.parse_block_statement()?)
                } else {
                    None
                };

                Some(Box::new(IfExpression {
                    token,
                    cond,
                    cons,
                    alternative,
                }))
            }
            Fn => {
                let token = self.current_clone()?;
                self.expect_peek(LParen)?;
                let parameters = self.parse_fn_params()?;
                self.expect_peek(LBrace)?;
                let body = self.parse_block_statement()?;
                Some(Box::new(FunctionLiteral {
                    token,
                    parameters,
                    body,
                }))
            }
            String => Some(Box::new(StringLiteral::try_from(current)?)),
            LBracket => {
                let current = self.current_clone()?;
                let elements = self.parse_expression_list(RBracket)?;
                Some(Box::new(ArrayLiteral {
                    token: current,
                    elements,
                }))
            }
            LBrace => {
                let current = self.current_clone()?;
                let mut pairs = HashMap::new();
                while self.peek_ref()?.kind != RBrace {
                    self.next_token();
                    let key = self.parse_expression(ExpressionKind::Lowest)?;
                    self.expect_peek(Colon)?;
                    self.next_token();
                    let value = self.parse_expression(ExpressionKind::Lowest)?;
                    pairs.insert(key, value);
                    if self.peek_ref()?.kind != RBrace {
                        self.expect_peek(Comma)?;
                    }
                }
                self.expect_peek(RBrace)?;
                Some(Box::new(HashLiteral {
                    token: current,
                    pairs,
                }))
            }
            _ => None,
        })
    }

    fn parse_fn_params(&mut self) -> Result<Vec<Identifier>, ParseError> {
        let mut out = Vec::new();
        self.next_token();
        if self.current_ref()?.kind == RParen {
            return Ok(out);
        }
        let current = self.current_clone()?;
        out.push(Identifier::try_from(current.clone())?);
        while self.peek_ref()?.kind == Comma {
            self.next_token();
            self.next_token();
            out.push({
                let token = self.current_clone()?;
                Identifier::try_from(token.clone())?
            })
        }

        self.expect_peek(RParen)?;
        Ok(out)
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, ParseError> {
        let token = self.current_clone()?;
        let mut statements = Vec::new();

        self.next_token();
        while self.current_ref()?.kind != RBrace {
            statements.push(self.parse_statement()?);
            self.next_token();
        }

        Ok(BlockStatement { token, statements })
    }

    fn parse_infix(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, ParseError> {
        let current = self.current_clone()?;
        if current.kind == LParen {
            return Ok(Box::new(CallExpression {
                token: current,
                function: left,
                arguments: self.parse_expression_list(RParen)?,
            }));
        }
        if current.kind == LBracket {
            self.next_token();
            let index = self.parse_expression(ExpressionKind::Lowest)?;
            self.expect_peek(RBracket)?;
            return Ok(Box::new(IndexExpression {
                token: current,
                left,
                index,
            }));
        }
        let kind = ExpressionKind::from(current.kind);
        self.next_token();
        let right = self.parse_expression(kind)?;

        Ok(Box::new(InfixExpression {
            operator: current.literal.clone(),
            token: current,
            left,
            right,
        }))
    }

    fn parse_expression_list(
        &mut self,
        end: TokenKind,
    ) -> Result<Vec<Box<dyn Expression>>, ParseError> {
        let mut out = Vec::new();
        self.next_token();
        if self.current_ref()?.kind == end {
            return Ok(out);
        }

        out.push(self.parse_expression(ExpressionKind::Lowest)?);
        while self.peek_ref()?.kind == Comma {
            self.next_token();
            self.next_token();
            out.push(self.parse_expression(ExpressionKind::Lowest)?);
        }

        self.expect_peek(end)?;
        Ok(out)
    }
}

impl FromStr for Program {
    type Err = ProgramError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(Lexer::new(s));
        let mut statements: Vec<Box<dyn Statement>> = Vec::new();
        let mut errors = Vec::new();

        while parser.current.is_some() {
            match parser.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => errors.push(err),
            }
            parser.next_token();
        }

        if !errors.is_empty() {
            Err(ProgramError::new(errors))
        } else {
            Ok(Program::new(statements))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token: {given} (expected {expected})")]
    Unexpected { given: Token, expected: TokenKind },
    #[error("Unexpected EOF")]
    Eof,
    #[error("No prefix expression")]
    NoPrefix,
    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, thiserror::Error)]
pub struct ProgramError(Vec<ParseError>);

impl ProgramError {
    pub fn new(errors: Vec<ParseError>) -> Self {
        Self(errors)
    }
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            writeln!(f, "{error}")?;
        }
        Ok(())
    }
}

#[derive(PartialEq, PartialOrd)]
enum ExpressionKind {
    Lowest,
    Equal,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

impl From<TokenKind> for ExpressionKind {
    fn from(value: TokenKind) -> Self {
        match value {
            Equal | NotEqual => Self::Equal,
            Less | Greater => Self::LessGreater,
            Plus | Minus => Self::Sum,
            Mult | Div => Self::Product,
            LParen => Self::Call,
            LBracket => Self::Index,
            _ => Self::Lowest,
        }
    }
}
