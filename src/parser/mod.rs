use crate::{
    ast::*,
    lexer::Lexer,
    token::{Span, Token, TokenKind},
};

#[cfg(test)]
mod test;

#[derive(thiserror::Error, Debug)]
#[error("{kind}")]
pub struct Error {
    span: Span,
    kind: ErrorKind,
}

impl Error {
    pub fn report(&self, input: &str) {
        use ariadne::{Color, Label, Report, ReportKind, Source};

        Report::build(ReportKind::Error, self.span)
            .with_message(self.to_string())
            .with_label(Label::new(self.span).with_color(Color::Red))
            .finish()
            .eprint(("input", Source::from(input)))
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
    #[error("Failed to parse integer literal.")]
    ParseInt(
        #[from]
        #[source]
        std::num::ParseIntError,
    ),
}

pub type Result<'a, T, E = Error> = std::result::Result<T, E>;

pub struct Parser<'a> {
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

    fn parse_program(mut self) -> Result<'a, Program<'a>, Vec<Error>> {
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

    fn parse_statement(&mut self) -> Result<'a, Statement<'a>, Error> {
        let error = |found| Error {
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

        let out = match token.kind {
            TokenKind::Let => Statement::Let(self.parse_let_statement(token)?),
            TokenKind::Return => Statement::Return(self.parse_return_statement(token)?),
            _ => Statement::Expression(self.parse_expression_statement(token)?),
        };

        if self.peek_is(TokenKind::Semicolon) {
            self.next_token();
        }

        Ok(out)
    }

    fn parse_expression_statement(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        let expr = self.parse_expression(Some(token), ExpressionKind::Base)?;

        Ok(expr)
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        self.peek.as_ref().map(|t| t.kind) == Some(kind)
    }

    fn parse_expression(
        &mut self,
        token: Option<Token<'a>>,
        kind: ExpressionKind,
    ) -> Result<'a, Expression<'a>> {
        let Some(token) = token.or_else(|| self.next_token().take()) else {
            return Err(self.unexpected_eof());
        };

        let Some(prefix_parser) = token.kind.prefix_parse() else {
            return Err(Error {
                span: token.span,
                kind: ErrorKind::Unexpected {
                    expected: "expression".to_string(),
                    found: Some(token.kind),
                },
            });
        };

        let mut left = prefix_parser(self, token)?;

        while self
            .peek
            .as_ref()
            .is_some_and(|t| t.kind != TokenKind::Semicolon)
            && kind < self.peek_expr_kind()?
        {
            let Some(peek) = self.peek.take() else {
                return Ok(left);
            };
            if !peek.kind.is_infix() && peek.kind != TokenKind::LParen {
                return Ok(left);
            };

            self.next_token();

            if peek.kind == TokenKind::LParen {
                left = self.parse_call(left)?;
            } else {
                left = self.parse_infix(peek, left)?;
            }
        }

        Ok(left)
    }

    fn parse_call(&mut self, function: Expression<'a>) -> Result<'a, Expression<'a>> {
        let arguments = self.parse_call_arguments()?;
        Ok(Expression::Call(Call {
            function: Box::new(function),
            arguments,
            close: self.expect_next(TokenKind::RParen)?,
        }))
    }

    fn parse_call_arguments(&mut self) -> Result<'a, Vec<Expression<'a>>> {
        let mut arguments = Vec::new();

        if self.peek_is(TokenKind::RParen) {
            self.next_token();
            return Ok(arguments);
        }

        arguments.push(self.parse_expression(None, ExpressionKind::Base)?);

        while self.peek_is(TokenKind::Comma) {
            self.next_token();
            arguments.push(self.parse_expression(None, ExpressionKind::Base)?);
        }

        Ok(arguments)
    }

    fn unexpected_eof(&self) -> Error {
        Error {
            span: Span {
                start: self.lexer.input.len(),
                end: self.lexer.input.len(),
            },
            kind: ErrorKind::Unexpected {
                expected: "expression".to_string(),
                found: None,
            },
        }
    }

    fn peek_expr_kind(&self) -> Result<'a, ExpressionKind> {
        self.peek
            .as_ref()
            .map(|t| ExpressionKind::from(t.kind))
            .ok_or_else(|| self.unexpected_eof())
    }

    fn parse_let_statement(&mut self, token: Token<'a>) -> Result<'a, Let<'a>> {
        let name_token = self.expect_next(TokenKind::Ident)?;
        let name = Identifier {
            value: name_token.literal,
            token: name_token,
        };
        self.expect_next(TokenKind::Assign)?;
        Ok(Let {
            value: self.parse_expression(None, ExpressionKind::Base)?,
            let_token: token,
            name,
        })
    }

    fn parse_return_statement(&mut self, token: Token<'a>) -> Result<'a, Return<'a>> {
        Ok(Return {
            return_token: token,
            value: self.parse_expression(None, ExpressionKind::Base)?,
        })
    }

    pub fn parse_identifier(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        Ok(Expression::Identifier(Identifier {
            value: token.literal,
            token,
        }))
    }

    pub fn parse_integer(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        let value = match token.literal.parse() {
            Ok(value) => value,
            Err(e) => {
                return Err(Error {
                    span: token.span,
                    kind: ErrorKind::ParseInt(e),
                });
            }
        };

        Ok(Expression::Integer(Integer { value, token }))
    }

    pub fn parse_prefix(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        Ok(Expression::Prefix(Prefix {
            operator: match token.kind {
                TokenKind::Not => PrefixOperator::Not,
                TokenKind::Minus => PrefixOperator::Neg,
                _ => unreachable!(),
            },
            operand: Box::new(self.parse_expression(None, ExpressionKind::Prefix)?),
            op_token: token,
        }))
    }

    fn parse_infix(
        &mut self,
        token: Token<'a>,
        left: Expression<'a>,
    ) -> Result<'a, Expression<'a>> {
        let kind = ExpressionKind::from(token.kind);
        let right = self.parse_expression(None, kind)?;
        Ok(Expression::Infix(Infix {
            left: Box::new(left),
            operator: match token.kind {
                TokenKind::Plus => InfixOperator::Add,
                TokenKind::Minus => InfixOperator::Sub,
                TokenKind::Mul => InfixOperator::Mul,
                TokenKind::Div => InfixOperator::Div,
                TokenKind::Eq => InfixOperator::Eq,
                TokenKind::Neq => InfixOperator::Neq,
                TokenKind::LT => InfixOperator::LT,
                TokenKind::GT => InfixOperator::GT,
                _ => unreachable!(),
            },
            right: Box::new(right),
        }))
    }

    pub fn parse_boolean(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        Ok(Expression::Boolean(Boolean {
            value: match token.literal {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            },
            token,
        }))
    }

    pub fn parse_grouped_expression(&mut self, _token: Token<'a>) -> Result<'a, Expression<'a>> {
        let expr = self.parse_expression(None, ExpressionKind::Base)?;

        self.expect_next(TokenKind::RParen)?;

        Ok(expr)
    }

    pub fn parse_if(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        self.expect_next(TokenKind::LParen)?;

        let condition = self.parse_expression(None, ExpressionKind::Base)?;

        self.expect_next(TokenKind::RParen)?;

        let lbrace = self.expect_next(TokenKind::LBrace)?;
        let consequence = self.parse_block_statement(lbrace)?;
        let mut alternative = None;

        if self.peek_is(TokenKind::Else) {
            self.next_token();
            let lbrace = self.expect_next(TokenKind::LBrace)?;
            alternative = Some(self.parse_block_statement(lbrace)?);
        }

        Ok(Expression::If(If {
            if_token: token,
            condition: Box::new(condition),
            consequence,
            alternative,
        }))
    }

    fn parse_block_statement(&mut self, open: Token<'a>) -> Result<'a, BlockStatement<'a>> {
        let mut statements = Vec::new();

        while self
            .peek
            .as_ref()
            .is_some_and(|t| t.kind != TokenKind::RBrace)
        {
            self.next_token();
            statements.push(self.parse_statement()?);
        }

        Ok(BlockStatement {
            open,
            statements,
            close: self.expect_next(TokenKind::RBrace)?,
        })
    }

    pub fn parse_function(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        self.expect_next(TokenKind::LParen)?;
        let parameters = self.parse_function_parameters()?;
        let lbrace = self.expect_next(TokenKind::LBrace)?;
        let body = self.parse_block_statement(lbrace)?;
        Ok(Expression::Function(Function {
            fn_token: token,
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<'a, Vec<Identifier<'a>>> {
        let mut identifiers = Vec::new();

        if self.peek_is(TokenKind::RParen) {
            self.next_token();
            return Ok(identifiers);
        }

        let first = self.expect_next(TokenKind::Ident)?;
        identifiers.push(Identifier {
            value: first.literal,
            token: first,
        });

        while self.peek_is(TokenKind::Comma) {
            self.next_token();
            let next = self.expect_next(TokenKind::Ident)?;
            identifiers.push(Identifier {
                value: next.literal,
                token: next,
            });
        }

        self.expect_next(TokenKind::RParen)?;

        Ok(identifiers)
    }
}

pub fn parse(input: &str) -> Result<Program, Vec<Error>> {
    Parser::new(input).parse_program()
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ExpressionKind {
    Base,
    Equal,
    Cmp,
    Sum,
    Product,
    Prefix,
    Call,
}

impl From<TokenKind> for ExpressionKind {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Eq | TokenKind::Neq => ExpressionKind::Equal,
            TokenKind::LT | TokenKind::GT => ExpressionKind::Cmp,
            TokenKind::Plus | TokenKind::Minus => ExpressionKind::Sum,
            TokenKind::Mul | TokenKind::Div => ExpressionKind::Product,
            TokenKind::LParen => ExpressionKind::Call,
            _ => ExpressionKind::Base,
        }
    }
}
