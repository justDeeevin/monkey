mod test;

use std::str::FromStr;

use crate::{
    ast::{
        BooleanLiteral, ExpressionStatement, Identifier, InfixExpression, IntegerLiteral,
        IntegerLiteralConstructionError, LetStatement, PrefixExpression, Program, ReturnStatement,
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
        if self.peek_ref()?.kind == Semi {
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

        while self.peek_ref()?.kind != Semi && kind < self.peek_ref()?.kind.into() {
            self.next_token();
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, ParseError> {
        let token = self.current_clone()?;

        self.expect_peek(Ident)?;

        let name = Identifier::try_from(self.current_clone()?).unwrap();

        self.expect_peek(Assign)?;

        self.skip_to_semi()?;

        Ok(LetStatement {
            name,
            value: Box::new(Identifier::new("foo")),
            token,
        })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, ParseError> {
        let out = ReturnStatement {
            token: self.current_clone()?,
            value: Box::new(Identifier::new("foo")),
        };

        self.skip_to_semi()?;

        Ok(out)
    }

    fn skip_to_semi(&mut self) -> Result<(), ParseError> {
        while self.current_ref()?.kind != Semi {
            self.next_token();
        }

        Ok(())
    }

    fn expect_peek(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let Some(peek) = &self.peek else {
            return Err(ParseError::Eof);
        };
        if peek.kind != expected {
            return Err(ParseError::Unexpected {
                given: peek.clone(),
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
            True | False => Some(Box::new(BooleanLiteral::try_from(current).unwrap())),
            LParen => {
                self.next_token();
                let exp = self.parse_expression(ExpressionKind::Lowest)?;
                self.expect_peek(RParen)?;
                Some(exp)
            }
            _ => None,
        })
    }

    fn parse_infix(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, ParseError> {
        let current = self.current_clone()?;
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
                Err(err) => {
                    errors.push(err);
                    if let Err(e) = parser.skip_to_semi() {
                        errors.push(e);
                    };
                }
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
pub struct ProgramError {
    errors: Vec<ParseError>,
}

impl ProgramError {
    pub fn new(errors: Vec<ParseError>) -> Self {
        Self { errors }
    }
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
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
}

impl From<TokenKind> for ExpressionKind {
    fn from(value: TokenKind) -> Self {
        match value {
            Equal | NotEqual => Self::Equal,
            Less | Greater => Self::LessGreater,
            Plus | Minus => Self::Sum,
            Mult | Div => Self::Product,
            Ident => Self::Call,
            _ => Self::Lowest,
        }
    }
}
