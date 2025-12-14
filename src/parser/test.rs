use crate::{ast::*, parser::parse};

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
    let Statement::Let(found) = found else {
        panic!(
            "expected let statement, got {:?}",
            StatementKind::from(found)
        );
    };

    assert_eq!(found.name.value, expected_name);
}

fn test_int_literal(expression: &Expression<'_>, expected_value: i64) {
    let Expression::Integer(integer) = expression else {
        panic!(
            "expected integer expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    assert_eq!(integer.value, expected_value);
}

fn test_ident(expression: &Expression<'_>, value: &str) {
    let Expression::Identifier(identifier) = expression else {
        panic!(
            "expected identifier expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    assert_eq!(identifier.value, value);
}

fn test_boolean(expression: &Expression<'_>, value: bool) {
    let Expression::Boolean(boolean) = expression else {
        panic!(
            "expected boolean expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    assert_eq!(boolean.value, value);
}

enum LiteralValue<'a> {
    Integer(i64),
    Identifier(&'a str),
    Boolean(bool),
}

fn test_literal_expr<'a>(expression: &Expression<'a>, expected_value: LiteralValue<'a>) {
    match expected_value {
        LiteralValue::Integer(expected_value) => test_int_literal(expression, expected_value),
        LiteralValue::Identifier(expected_value) => test_ident(expression, expected_value),
        LiteralValue::Boolean(expected_value) => test_boolean(expression, expected_value),
    }
}

fn test_infix_expr<'a>(
    expression: &Expression<'a>,
    left: LiteralValue<'a>,
    operator: InfixOperator,
    right: LiteralValue<'_>,
) {
    let Expression::Infix(infix) = expression else {
        panic!(
            "expected infix expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    test_literal_expr(&infix.left, left);
    assert_eq!(infix.operator, operator);
    test_literal_expr(&infix.right, right);
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
        assert_eq!(StatementKind::from(statement), StatementKind::Return);
    }
}

#[test]
fn ident_expr() {
    let input = "foobar;";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);

    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    let Expression::Identifier(ident) = expression else {
        panic!(
            "expected identifier expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    assert_eq!(ident.value, "foobar");
}

#[test]
fn int_literal_expr() {
    let input = "5;";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);
    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    test_literal_expr(expression, LiteralValue::Integer(5));
}

#[test]
fn prefix_expr() {
    let tests = [
        ("!5;", PrefixOperator::Not, 5),
        ("-15;", PrefixOperator::Neg, 15),
    ];

    for (expected_operator, expected_operand, program) in
        tests.into_iter().map(|(i, o, a)| (o, a, get_program(i)))
    {
        assert_eq!(program.statements.len(), 1);

        let Statement::Expression(expression) = program.statements.first().unwrap() else {
            panic!(
                "expected expression statement, got {:?}",
                StatementKind::from(program.statements.first().unwrap())
            );
        };

        let Expression::Prefix(prefix) = expression else {
            panic!(
                "expected prefix expression, got {:?}",
                ExpressionKind::from(expression)
            );
        };

        assert_eq!(prefix.operator, expected_operator);

        test_int_literal(&prefix.operand, expected_operand);
    }
}

#[test]
fn infix_expr() {
    let tests = [
        ("5 + 5;", InfixOperator::Add, 5, 5),
        ("5 - 5;", InfixOperator::Sub, 5, 5),
        ("5 * 5;", InfixOperator::Mul, 5, 5),
        ("5 / 5;", InfixOperator::Div, 5, 5),
        ("5 < 5;", InfixOperator::LT, 5, 5),
        ("5 > 5;", InfixOperator::GT, 5, 5),
        ("5 == 5;", InfixOperator::Eq, 5, 5),
        ("5 != 5;", InfixOperator::Neq, 5, 5),
    ];

    for (expected_operator, expected_left, expected_right, program) in tests
        .into_iter()
        .map(|(i, o, l, r)| (o, l, r, get_program(i)))
    {
        assert_eq!(program.statements.len(), 1);

        let Statement::Expression(expression) = program.statements.first().unwrap() else {
            panic!(
                "expected expression statement, got {:?}",
                StatementKind::from(program.statements.first().unwrap())
            )
        };

        test_infix_expr(
            expression,
            LiteralValue::Integer(expected_left),
            expected_operator,
            LiteralValue::Integer(expected_right),
        );
    }
}

#[test]
fn boolean_literal_expr() {
    let input = "true;";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);

    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    test_literal_expr(expression, LiteralValue::Boolean(true));
}

#[test]
fn if_expr() {
    let input = "if (x < y) { x }";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);

    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    let Expression::If(if_expr) = expression else {
        panic!(
            "expected if expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    test_infix_expr(
        &if_expr.condition,
        LiteralValue::Identifier("x"),
        InfixOperator::LT,
        LiteralValue::Identifier("y"),
    );

    assert_eq!(if_expr.consequence.statements.len(), 1);

    let Statement::Expression(consequence) = if_expr.consequence.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(if_expr.consequence.statements.first().unwrap())
        );
    };

    test_literal_expr(consequence, LiteralValue::Identifier("x"));

    assert!(if_expr.alternative.is_none());
}

#[test]
fn if_else_expr() {
    let input = "if (x < y) { x } else { y }";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);

    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    let Expression::If(if_expr) = expression else {
        panic!(
            "expected if expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    test_infix_expr(
        &if_expr.condition,
        LiteralValue::Identifier("x"),
        InfixOperator::LT,
        LiteralValue::Identifier("y"),
    );

    assert_eq!(if_expr.consequence.statements.len(), 1);

    let Statement::Expression(consequence) = if_expr.consequence.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(if_expr.consequence.statements.first().unwrap())
        );
    };

    test_literal_expr(consequence, LiteralValue::Identifier("x"));

    let Some(alternative) = &if_expr.alternative else {
        panic!("expected alternative");
    };

    assert_eq!(alternative.statements.len(), 1);

    let Statement::Expression(alternative) = alternative.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(alternative.statements.first().unwrap())
        );
    };

    test_literal_expr(alternative, LiteralValue::Identifier("y"));
}

#[test]
fn fn_expr() {
    let input = "fn(x, y) { x + y; }";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);

    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    let Expression::Function(function) = expression else {
        panic!(
            "expected function expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    assert_eq!(function.parameters.len(), 2);

    assert_eq!(function.parameters[0].value, "x");
    assert_eq!(function.parameters[1].value, "y");

    assert_eq!(function.body.statements.len(), 1);

    let Statement::Expression(body) = function.body.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(function.body.statements.first().unwrap())
        );
    };

    test_infix_expr(
        body,
        LiteralValue::Identifier("x"),
        InfixOperator::Add,
        LiteralValue::Identifier("y"),
    );
}

#[test]
fn fn_params() {
    let tests = [
        ("fn() {};", [].as_slice()),
        ("fn(x) {};", ["x"].as_slice()),
        ("fn(x, y, z) {};", ["x", "y", "z"].as_slice()),
    ];

    for (expected, program) in tests.into_iter().map(|(i, ps)| (ps, get_program(i))) {
        assert_eq!(program.statements.len(), 1);
        let Statement::Expression(expression) = program.statements.first().unwrap() else {
            panic!(
                "expected expression statement, got {:?}",
                StatementKind::from(program.statements.first().unwrap())
            );
        };

        let Expression::Function(function) = expression else {
            panic!(
                "expected function expression, got {:?}",
                ExpressionKind::from(expression)
            );
        };

        assert_eq!(function.parameters.len(), expected.len());

        for (expected, found) in expected.iter().zip(&function.parameters) {
            assert_eq!(*expected, found.value);
        }
    }
}

#[test]
fn call_expr() {
    let input = "add(1, 2 * 3, 4 + 5);";

    let program = get_program(input);

    assert_eq!(program.statements.len(), 1);
    let Statement::Expression(expression) = program.statements.first().unwrap() else {
        panic!(
            "expected expression statement, got {:?}",
            StatementKind::from(program.statements.first().unwrap())
        );
    };

    let Expression::Call(call) = expression else {
        panic!(
            "expected call expression, got {:?}",
            ExpressionKind::from(expression)
        );
    };

    test_ident(&call.function, "add");

    assert_eq!(call.arguments.len(), 3);

    test_literal_expr(&call.arguments[0], LiteralValue::Integer(1));
    test_infix_expr(
        &call.arguments[1],
        LiteralValue::Integer(2),
        InfixOperator::Mul,
        LiteralValue::Integer(3),
    );
    test_infix_expr(
        &call.arguments[2],
        LiteralValue::Integer(4),
        InfixOperator::Add,
        LiteralValue::Integer(5),
    );
}
