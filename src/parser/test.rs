#![cfg(test)]

use crate::ast::{Identifier, LetStatement, Program, ReturnStatement, traits::Node};

#[test]
fn let_statements() {
    let input = r#"
        let x = 5;
        let y = 10;
        let foobar = 838383;
    "#;

    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => {
            panic!("Failed to parse program: {e}");
        }
    };

    // Subtract 2 for the start and end newlines in the input
    assert_eq!(program.statements.len(), input.lines().count() - 2);

    let test_idents = ["x", "y", "foobar"];

    for (ident, statement) in test_idents.iter().zip(program.statements) {
        assert_eq!(statement.token_literal(), "let");

        let let_statement = statement
            .downcast_ref::<LetStatement<Identifier>>()
            .unwrap();

        assert_eq!(let_statement.name.value(), *ident);
        assert_eq!(let_statement.name.token_literal(), *ident);
    }
}

#[test]
fn return_statements() {
    let input = r#"
        return 5;
        return 10;
        return 993322;
    "#;

    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => {
            panic!("Failed to parse program: {e}");
        }
    };

    assert_eq!(program.statements.len(), input.lines().count() - 2);
    for statement in program.statements {
        let return_statement = statement
            .downcast_ref::<ReturnStatement<Identifier>>()
            .unwrap();
        assert_eq!(return_statement.token_literal(), "return");
    }
}
