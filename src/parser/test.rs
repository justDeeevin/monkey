use crate::{
    ast::{Node, Program, Statement, StatementKind},
    parser::parse,
};

fn get_program(input: &str) -> Program<'_> {
    parse(input).unwrap_or_else(|errors| {
        panic!(
            "failed to parse program:{}",
            errors
                .into_iter()
                .fold(String::new(), |acc, e| format!("{acc}\n{e}"))
        );
    })
}

fn test_let_statement(found: Statement<'_>, expected_name: &str) {
    assert_eq!(found.literal(), "let");

    let Statement::Let(found) = found else {
        panic!(
            "expected let statement, got {:?}",
            StatementKind::from(found)
        );
    };

    assert_eq!(found.name.value, expected_name);

    assert_eq!(found.name.literal(), expected_name);
}

#[test]
fn let_statements() {
    let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
"#;

    let program = get_program(input);

    assert_eq!(program.statements.len(), 3);

    let tests = ["x", "y", "foobar"];

    for (expected, found) in tests.into_iter().zip(program.statements) {
        test_let_statement(found, expected);
    }
}

#[test]
fn return_statements() {
    let input = r#"
return 5;
return 10;
return 993322;
"#;

    let program = get_program(input);
    assert_eq!(program.statements.len(), 3);

    for statement in program.statements {
        assert_eq!(statement.literal(), "return");
        assert_eq!(StatementKind::from(statement), StatementKind::Return);
    }
}
