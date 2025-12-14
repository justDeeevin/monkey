use crate::{ast::*, token::*};

#[test]
fn display() {
    let program = Program {
        statements: vec![Statement::Let {
            let_token: Token {
                kind: TokenKind::Let,
                literal: "let",
                span: (0..3).into(),
            },
            name: Identifier {
                token: Token {
                    kind: TokenKind::Ident,
                    literal: "myVar",
                    span: (4..9).into(),
                },
                value: "myVar",
            },
            value: Expression::Identifier(Identifier {
                token: Token {
                    kind: TokenKind::Ident,
                    literal: "anotherVar",
                    span: (10..20).into(),
                },
                value: "anotherVar",
            }),
        }],
    };

    assert_eq!(program.to_string().trim(), "let myVar = (anotherVar);");
}
