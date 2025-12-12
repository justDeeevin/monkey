use crate::{
    ast::*,
    lexer::Lexer,
    token::{Span, Token, TokenKind},
};

#[cfg(test)]
mod test;

#[derive(thiserror::Error, Debug)]
#[error("{kind}")]
pub struct Error<'a> {
    input: &'a str,
    span: Span,
    kind: ErrorKind,
}

impl Error<'_> {
    pub fn report(&self) {
        use ariadne::{Color, Label, Report, ReportKind, Source};

        Report::build(ReportKind::Error, self.span)
            .with_message(self.to_string())
            .with_label(Label::new(self.span).with_color(Color::Red))
            .finish()
            .eprint(("input", Source::from(self.input)))
            .unwrap();
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Unexpected token. Expected {expected}, found {}.", found.map(|k| k.to_string()).unwrap_or_else(|| "end of input".to_string()))]
    Unexpected {
        expected: String,
        found: Option<TokenKind>,
    },
}

pub type Result<'a, T, E = Error<'a>> = std::result::Result<T, E>;

struct Parser<'a> {
    lexer: Lexer<'a>,

    current: Option<Token<'a>>,
    peek: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current = lexer.next();
        let peek = lexer.next();
        Self {
            lexer,
            current,
            peek,
        }
    }

    fn next_token(&mut self) -> &mut Option<Token<'a>> {
        self.current = self.peek.take();
        self.peek = self.lexer.next();
        &mut self.current
    }

    fn expect_next(&mut self, expected: TokenKind) -> Result<'a, Token<'a>> {
        let input = self.lexer.input;

        match self.next_token().take() {
            Some(token) if token.kind == expected => Ok(token),
            found => Err(Error {
                input,
                span: Span {
                    start: found.as_ref().map(|t| t.span.start).unwrap_or(input.len()),
                    end: input.len(),
                },
                kind: ErrorKind::Unexpected {
                    expected: expected.to_string(),
                    found: found.as_ref().map(|t| t.kind),
                },
            }),
        }
    }

    fn parse_program(mut self) -> Result<'a, Program<'a>, Vec<Error<'a>>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while self.current.is_some() {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => {
                    errors.push(err);
                    while self
                        .next_token()
                        .as_ref()
                        .is_some_and(|t| t.kind != TokenKind::Semicolon)
                    {}
                }
            }
            self.next_token();
        }

        if errors.is_empty() {
            Ok(Program { statements })
        } else {
            Err(errors)
        }
    }

    fn parse_statement(&mut self) -> Result<'a, Statement<'a>, Error<'a>> {
        let error = |found| Error {
            input: self.lexer.input,
            span: Span {
                start: self.lexer.input.len(),
                end: self.lexer.input.len(),
            },
            kind: ErrorKind::Unexpected {
                expected: "statement".to_string(),
                found,
            },
        };
        let Some(token) = self.current.take() else {
            return Err(error(None));
        };

        Ok(match token.kind {
            TokenKind::Let => Statement::Let(self.parse_let_statement(token)?),
            TokenKind::Return => Statement::Return(self.parse_return_statement(token)?),
            _ => todo!(),
        })
    }

    fn parse_let_statement(&mut self, token: Token<'a>) -> Result<'a, Let<'a>> {
        let name_token = self.expect_next(TokenKind::Ident)?;
        let name = Identifier {
            value: name_token.literal,
            token: name_token,
        };
        self.expect_next(TokenKind::Assign)?;
        while self
            .next_token()
            .as_ref()
            .is_some_and(|t| t.kind != TokenKind::Semicolon)
        {}

        Ok(Let {
            token,
            name,
            value: Expression::Temp,
        })
    }

    fn parse_return_statement(&mut self, token: Token<'a>) -> Result<'a, Return<'a>> {
        self.next_token();
        while self
            .next_token()
            .as_ref()
            .is_some_and(|t| t.kind != TokenKind::Semicolon)
        {}

        Ok(Return {
            token,
            value: Expression::Temp,
        })
    }
}

pub fn parse(input: &str) -> Result<Program, Vec<Error>> {
    Parser::new(input).parse_program()
}
