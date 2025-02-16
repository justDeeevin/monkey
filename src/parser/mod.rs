mod test;

use std::str::FromStr;

use crate::{
    ast::{
        ExpressionStatement, Identifier, IntegerLiteral, LetStatement, Program, ReturnStatement,
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

    fn parse_statement(&mut self) -> Result<Box<dyn Statement>, ParseError> {
        let current = self.current.clone().ok_or(ParseError::Eof)?;
        Ok(match current.kind {
            Let => Box::new(self.parse_let_statement()?),
            Return => Box::new(self.parse_return_statement()?),
            Ident | Int | True | False | Minus | LParen | If => {
                Box::new(self.parse_expression_statement()?)
            }
            _ => {
                return Err(ParseError::NoStatement {
                    given: current.clone(),
                });
            }
        })
    }

    fn parse_expression_statement(&mut self) -> Result<ExpressionStatement, ParseError> {
        let out = ExpressionStatement {
            token: self.current.clone().ok_or(ParseError::Eof)?,
            expression: self.parse_expression(ExpressionKind::Lowest)?,
        };
        if self.peek.as_ref().ok_or(ParseError::Eof)?.kind == Semi {
            self.next_token();
        }
        Ok(out)
    }

    fn parse_expression(
        &mut self,
        kind: ExpressionKind,
    ) -> Result<Box<dyn Expression>, ParseError> {
        if let Some(expr) = self.parse_prefix()? {
            return Ok(expr);
        }

        todo!()
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, ParseError> {
        let token = self.current.clone().ok_or(ParseError::Eof)?;

        self.expect_peek(Ident)?;

        let name = Identifier::from_token(self.current.clone().ok_or(ParseError::Eof)?);

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
            token: self.current.clone().ok_or(ParseError::Eof)?,
            value: Box::new(Identifier::new("foo")),
        };

        self.skip_to_semi()?;

        Ok(out)
    }

    fn skip_to_semi(&mut self) -> Result<(), ParseError> {
        while self.current.as_ref().ok_or(ParseError::Eof)?.kind != Semi {
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
        let current = self.current.clone().ok_or(ParseError::Eof)?;
        Ok(match current.kind {
            Ident => Some(Box::new(Identifier::from_token(current))),
            Int => Some(Box::new(IntegerLiteral::from_token(current)?)),
            _ => todo!(),
        })
    }

    fn parse_infix(&mut self, left: &dyn Expression) -> Result<Box<dyn Expression>, ParseError> {
        todo!()
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
    #[error("Unexpected token: {given} (expected start of statement)")]
    NoStatement { given: Token },
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
