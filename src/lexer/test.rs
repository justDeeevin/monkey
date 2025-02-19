#![cfg(test)]

use super::Lexer;
use crate::token::TokenKind::*;

#[test]
fn lex() {
    let input = r#"
        let five = 5;
        let ten = 10;
        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;

        if (5 < 10) {
            return true;
        }
        else {
            return false;
        }

        10 == 10;
        10 != 9;
        "foobar"
        "foo bar"
        [1,2];
        {"foo": "bar"}
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
        (Not, "!"),
        (Minus, "-"),
        (Div, "/"),
        (Mult, "*"),
        (Int, "5"),
        (Semi, ";"),
        (Int, "5"),
        (Less, "<"),
        (Int, "10"),
        (Greater, ">"),
        (Int, "5"),
        (Semi, ";"),
        (If, "if"),
        (LParen, "("),
        (Int, "5"),
        (Less, "<"),
        (Int, "10"),
        (RParen, ")"),
        (LBrace, "{"),
        (Return, "return"),
        (True, "true"),
        (Semi, ";"),
        (RBrace, "}"),
        (Else, "else"),
        (LBrace, "{"),
        (Return, "return"),
        (False, "false"),
        (Semi, ";"),
        (RBrace, "}"),
        (Int, "10"),
        (Equal, "=="),
        (Int, "10"),
        (Semi, ";"),
        (Int, "10"),
        (NotEqual, "!="),
        (Int, "9"),
        (Semi, ";"),
        (String, "foobar"),
        (String, "foo bar"),
        (LBracket, "["),
        (Int, "1"),
        (Comma, ","),
        (Int, "2"),
        (RBracket, "]"),
        (Semi, ";"),
        (LBrace, "{"),
        (String, "foo"),
        (Colon, ":"),
        (String, "bar"),
        (RBrace, "}"),
    ];

    let lexer = Lexer::new(input);

    for ((kind, literal), token) in tests.iter().zip(lexer) {
        assert_eq!(&token.kind, kind);
        assert_eq!(token.literal, (*literal).into());
    }
}
