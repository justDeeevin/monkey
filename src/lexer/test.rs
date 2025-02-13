#![cfg(test)]

use super::Lexer;
use crate::token::TokenKind::*;

#[test]
fn test_next() {
    let input = r#"
        let five = 5;
        let ten = 10;
        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);
    "#;

    let tests = [
        (Let, "let"),
        (Ident, "five"),
        (Assign, "="),
        (Int, "5"),
        (Semi, ";"),
        (Let, "let"),
        (Ident, "ten"),
        (Assign, "="),
        (Int, "10"),
        (Semi, ";"),
        (Let, "let"),
        (Ident, "add"),
        (Assign, "="),
        (Fn, "fn"),
        (LParen, "("),
        (Ident, "x"),
        (Comma, ","),
        (Ident, "y"),
        (RParen, ")"),
        (LBrace, "{"),
        (Ident, "x"),
        (Plus, "+"),
        (Ident, "y"),
        (Semi, ";"),
        (RBrace, "}"),
        (Semi, ";"),
        (Let, "let"),
        (Ident, "result"),
        (Assign, "="),
        (Ident, "add"),
        (LParen, "("),
        (Ident, "five"),
        (Comma, ","),
        (Ident, "ten"),
        (RParen, ")"),
        (Semi, ";"),
    ];

    let lexer = Lexer::new(input);

    for (item, ((kind, literal), token)) in tests.iter().zip(lexer).enumerate() {
        assert_eq!(&token.kind, kind);
        assert_eq!(token.literal, (*literal).into());
    }
}
