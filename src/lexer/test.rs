use crate::{lexer::Lexer, token::TokenKind};

#[test]
fn test_next() {
    let input = r#"let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
  return true;
} else {
  return false;
}

10 == 10;
10 != 9;
"#;

    let tests = [
        (TokenKind::Let, "let"),
        (TokenKind::Ident, "five"),
        (TokenKind::Assign, "="),
        (TokenKind::Int, "5"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Let, "let"),
        (TokenKind::Ident, "ten"),
        (TokenKind::Assign, "="),
        (TokenKind::Int, "10"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Let, "let"),
        (TokenKind::Ident, "add"),
        (TokenKind::Assign, "="),
        (TokenKind::Fn, "fn"),
        (TokenKind::LParen, "("),
        (TokenKind::Ident, "x"),
        (TokenKind::Comma, ","),
        (TokenKind::Ident, "y"),
        (TokenKind::RParen, ")"),
        (TokenKind::LBrace, "{"),
        (TokenKind::Ident, "x"),
        (TokenKind::Plus, "+"),
        (TokenKind::Ident, "y"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Let, "let"),
        (TokenKind::Ident, "result"),
        (TokenKind::Assign, "="),
        (TokenKind::Ident, "add"),
        (TokenKind::LParen, "("),
        (TokenKind::Ident, "five"),
        (TokenKind::Comma, ","),
        (TokenKind::Ident, "ten"),
        (TokenKind::RParen, ")"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Not, "!"),
        (TokenKind::Minus, "-"),
        (TokenKind::Div, "/"),
        (TokenKind::Mul, "*"),
        (TokenKind::Int, "5"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Int, "5"),
        (TokenKind::LT, "<"),
        (TokenKind::Int, "10"),
        (TokenKind::GT, ">"),
        (TokenKind::Int, "5"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::If, "if"),
        (TokenKind::LParen, "("),
        (TokenKind::Int, "5"),
        (TokenKind::LT, "<"),
        (TokenKind::Int, "10"),
        (TokenKind::RParen, ")"),
        (TokenKind::LBrace, "{"),
        (TokenKind::Return, "return"),
        (TokenKind::True, "true"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Else, "else"),
        (TokenKind::LBrace, "{"),
        (TokenKind::Return, "return"),
        (TokenKind::False, "false"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Int, "10"),
        (TokenKind::Eq, "=="),
        (TokenKind::Int, "10"),
        (TokenKind::Semicolon, ";"),
        (TokenKind::Int, "10"),
        (TokenKind::Neq, "!="),
        (TokenKind::Int, "9"),
        (TokenKind::Semicolon, ";"),
    ];

    for (i, ((expected_kind, expected_literal), token)) in
        tests.into_iter().zip(Lexer::new(input)).enumerate()
    {
        dbg!(i, &token);
        assert_eq!(expected_kind, token.kind);
        assert_eq!(expected_literal, token.literal);
    }
}
