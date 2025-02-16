#![cfg(test)]

use crate::{
    ast::{
        ExpressionStatement, Identifier, IntegerLiteral, LetStatement, Program, ReturnStatement,
        traits::Node,
    },
    token::Token,
};

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
            .downcast_ref::<LetStatement>()
            .expect("Could not downcast to let statement");

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
            .downcast_ref::<ReturnStatement>()
            .expect("Could not downcast to return statement");
        assert_eq!(return_statement.token_literal(), "return");
    }
}

#[test]
fn format() {
    let program = Program::new(vec![Box::new(LetStatement {
        token: Token::word("let"),
        name: Identifier::new("myVar"),
        value: Box::new(Identifier::new("anotherVar")),
    })]);

    assert_eq!(program.to_string(), "let myVar = anotherVar;");
}

#[test]
fn ident_expr() {
    let input = "foobar;";
    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => {
            panic!("Failed to parse program: {e}");
        }
    };
    assert_eq!(program.statements.len(), 1);

    let ident = program
        .statements
        .first()
        .unwrap()
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<Identifier>()
        .expect("Could not downcast to identifier");

    assert_eq!(ident.value(), "foobar");
    assert_eq!(ident.token_literal(), "foobar");
}

#[test]
fn int_literal() {
    let input = "5;";
    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => {
            panic!("Failed to parse program: {e}");
        }
    };
    assert_eq!(program.statements.len(), 1);

    let int = program
        .statements
        .first()
        .unwrap()
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<IntegerLiteral>()
        .expect("Could not downcast to identifier");

    assert_eq!(int.value(), 5);
    assert_eq!(int.token_literal(), "5");
}
