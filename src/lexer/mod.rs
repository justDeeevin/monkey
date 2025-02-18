use std::rc::Rc;

mod test;
use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Rc<str>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub fn new(input: impl AsRef<str>) -> Self {
        let input = input.as_ref();
        Self {
            input: input.into(),
            position: 0,
            read_position: 1,
            ch: input.chars().next(),
        }
    }

    fn read_char(&mut self) {
        self.ch = self.input.chars().nth(self.read_position);
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek(&mut self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.ch.unwrap_or('\0').is_whitespace() {
            self.read_char();
        }
        let ch = self.ch?;

        // past-tense read
        let (out, read) = if ch.is_ascii_digit() {
            let position = self.position;
            while self.ch.unwrap_or('\0').is_ascii_digit() {
                self.read_char();
            }
            (
                Token {
                    kind: TokenKind::Int,
                    literal: self.input[position..self.position].into(),
                },
                true,
            )
        } else if ch.is_alphabetic() || ch == '_' {
            let position = self.position;
            while self.ch.unwrap_or('\0').is_alphanumeric() || self.ch.unwrap_or('\0') == '_' {
                self.read_char();
            }
            (Token::word(&self.input[position..self.position]), true)
        } else if ch == '"' {
            let position = self.position + 1;
            loop {
                self.read_char();
                let Some(ch) = self.ch else {
                    break;
                };
                // TODO: escape sequences
                if ch == '"' {
                    break;
                }
            }
            (
                Token {
                    kind: TokenKind::String,
                    literal: self.input[position..self.position].into(),
                },
                false,
            )
        } else {
            let tok = Token::special(ch, self.peek().unwrap_or('\0'));
            if tok.kind.is_double() {
                self.read_char();
            }
            (tok, false)
        };

        if !read {
            self.read_char();
        }

        Some(out)
    }
}
