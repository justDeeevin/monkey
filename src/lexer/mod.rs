use std::rc::Rc;

mod test;
use crate::token::{Location, Span, Token};

pub struct Lexer {
    input: Rc<str>,
    position: usize,
    line: usize,
    column: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: impl AsRef<str>) -> Self {
        let input = input.as_ref();
        Self {
            input: input.into(),
            position: 0,
            line: 1,
            column: 2,
            read_position: 1,
            ch: input.chars().next().unwrap_or('\0'),
        }
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        if self.ch == '\n' {
            self.line += 1;
            self.column = 1;
        }
        self.read_position += 1;
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ch == '\0' {
            return None;
        }

        let loc = Location {
            line: self.line,
            column: self.column,
        };

        let lit = self.ch.to_string();

        self.read_char();

        Some(Token::new(lit, Span {
            start: loc.clone(),
            end: loc,
        }))
    }
}
