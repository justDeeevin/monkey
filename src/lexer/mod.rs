use crate::token::{Span, Token, TokenKind, lookup_keyword};

#[cfg(test)]
mod test;

pub struct Lexer<'a> {
    pub input: &'a str,
    pos: usize,
    next_pos: usize,
    char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let char = input.chars().next();
        let next_pos = if let Some(c) = char { c.len_utf8() } else { 0 };
        Self {
            input,
            pos: 0,
            next_pos,
            char,
        }
    }

    fn read_char(&mut self) {
        self.char = self.input[self.next_pos..].chars().next();
        self.pos = self.next_pos;
        let Some(c) = self.char else {
            return;
        };
        self.next_pos += c.len_utf8();
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.next_pos..].chars().next()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.char?.is_whitespace() {
            self.read_char();
        }

        let start = self.pos;

        let kind = match self.char? {
            '=' => {
                if let Some('=') = self.peek_char() {
                    self.read_char();
                    TokenKind::Eq
                } else {
                    TokenKind::Assign
                }
            }
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '!' => {
                if let Some('=') = self.peek_char() {
                    self.read_char();
                    TokenKind::Neq
                } else {
                    TokenKind::Not
                }
            }
            '/' => TokenKind::Div,
            '*' => TokenKind::Mul,
            '<' => TokenKind::LT,
            '>' => TokenKind::GT,
            ';' => TokenKind::Semicolon,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            ',' => TokenKind::Comma,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '"' => {
                while self.peek_char().is_some_and(|c| c != '"') {
                    self.read_char();
                }
                self.read_char();
                TokenKind::String
            }
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ':' => TokenKind::Colon,
            c if c.is_alphabetic() || c == '_' => {
                while self
                    .peek_char()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    self.read_char();
                }
                lookup_keyword(&self.input[start..self.next_pos]).unwrap_or(TokenKind::Ident)
            }
            c if c.is_ascii_digit() => {
                while self.peek_char().as_ref().is_some_and(char::is_ascii_digit) {
                    self.read_char();
                }
                TokenKind::Int
            }
            c => TokenKind::Illegal(c),
        };

        self.read_char();

        let end = if kind == TokenKind::String {
            self.pos - 1
        } else {
            self.pos
        };

        let literal_start = if kind == TokenKind::String {
            start + 1
        } else {
            start
        };

        Some(Token {
            kind,
            literal: &self.input[literal_start..end],
            span: Span {
                start,
                end: self.pos,
            },
        })
    }
}
