mod test;

use std::str::FromStr;

use crate::{
    ast::{
        Identifier, LetStatement, Program,
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

    fn parse_statement(&mut self) -> Result<impl Statement + use<>, Error> {
        match self.current.as_ref().ok_or(Error::Eof)?.kind {
            Let => self.parse_let_statement(),
            _ => todo!(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement<impl Expression + use<>>, Error> {
        let token = self.current.clone().ok_or(Error::Eof)?;

        self.expect_peek(Ident)?;

        let name = Identifier::from_token(self.current.clone().ok_or(Error::Eof)?);

        self.expect_peek(Assign)?;

        while self.current.as_ref().ok_or(Error::Eof)?.kind != Semi {
            self.next_token();
        }

        Ok(LetStatement {
            name,
            value: Identifier::new("foo"),
            token,
        })
    }

    fn expect_peek(&mut self, expected: TokenKind) -> Result<(), Error> {
        let Some(peek) = &self.peek else {
            return Err(Error::Eof);
        };
        if peek.kind != expected {
            return Err(Error::Unexpected {
                given: peek.clone(),
                expected,
            });
        }
        self.next_token();
        Ok(())
    }
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(Lexer::new(s));
        let mut statements: Vec<Box<dyn Statement>> = Vec::new();

        while parser.current.is_some() {
            statements.push(Box::new(parser.parse_statement()?));
            parser.next_token();
        }

        Ok(Program { statements })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected token: {given} (expected {expected})")]
    Unexpected { given: Token, expected: TokenKind },
    #[error("Unexpected EOF")]
    Eof,
}
