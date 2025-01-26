#![cfg(test)]

use super::Lexer;
use crate::token::TokenKind;

#[test]
fn test_next() {
    let tests = [
        (TokenKind::Assign, "="),
        (TokenKind::Plus, "+"),
        (TokenKind::LParen, "("),
        (TokenKind::RParen, ")"),
        (TokenKind::LBrace, "{"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Comma, ","),
        (TokenKind::Semicolon, ";"),
    ];

    let lexer = Lexer::new("=+(){},;");

    for ((kind, literal), token) in tests.iter().zip(lexer) {
        assert_eq!(&token.kind, kind);
        assert_eq!(token.literal, (*literal).into());
    }
}
