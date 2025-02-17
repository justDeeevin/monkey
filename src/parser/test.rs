#![cfg(test)]

use std::any::Any;

use crate::{
    ast::{
        BooleanLiteral, ExpressionStatement, Identifier, IfExpression, InfixExpression, Integer,
        IntegerLiteral, LetStatement, PrefixExpression, Program, ReturnStatement,
        traits::{Expression, Node},
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

    let ident = program.statements[0]
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

    let int = program.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<IntegerLiteral>()
        .expect("Could not downcast to identifier");

    assert_eq!(int.value(), 5);
    assert_eq!(int.token_literal(), "5");
}

#[test]
fn prefix_expr() {
    let integer_inputs = [("!5;", '!', 5), ("-15;", '-', 15)];

    for (input, operator, expected) in integer_inputs {
        let program = match input.parse::<Program>() {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to parse program: {e}");
            }
        };

        assert_eq!(program.statements.len(), 1);

        let prefix_expr = program.statements[0]
            .downcast_ref::<ExpressionStatement>()
            .expect("Could not downcast to expression statement")
            .expression
            .downcast_ref::<PrefixExpression>()
            .expect("Could not downcast to prefix expression");

        assert_eq!(prefix_expr.operator, operator);

        test_int_literal(prefix_expr.right.as_ref(), expected);
    }

    let boolean_inputs = [("!true;", '!', true), ("!false;", '!', false)];
    for (input, operator, expected) in boolean_inputs {
        let program = match input.parse::<Program>() {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to parse program: {e}");
            }
        };

        assert_eq!(program.statements.len(), 1);

        let prefix_expr = program.statements[0]
            .downcast_ref::<ExpressionStatement>()
            .expect("Could not downcast to expression statement")
            .expression
            .downcast_ref::<PrefixExpression>()
            .expect("Could not downcast to prefix expression");

        assert_eq!(prefix_expr.operator, operator);

        test_bool_literal(prefix_expr.right.as_ref(), expected);
    }
}

fn test_int_literal(expr: &dyn Expression, value: Integer) {
    let int = expr
        .downcast_ref::<IntegerLiteral>()
        .expect("Could not downcast to integer literal");
    assert_eq!(int.value(), value);
    assert_eq!(int.token_literal(), &value.to_string());
}

fn test_identifier(expr: &dyn Expression, value: &str) {
    let ident = expr
        .downcast_ref::<Identifier>()
        .expect("Could not downcast to identifier");
    assert_eq!(ident.value(), value);
    assert_eq!(ident.token_literal(), value);
}

fn test_literal(expr: &dyn Expression, value: &dyn Any) {
    if let Some(value) = value.downcast_ref::<Integer>() {
        test_int_literal(expr, *value);
    } else if let Some(value) = value.downcast_ref::<&str>() {
        test_identifier(expr, value);
    } else if let Some(value) = value.downcast_ref::<bool>() {
        test_bool_literal(expr, *value);
    } else {
        panic!("Unsupported literal type");
    }
}

fn test_bool_literal(expr: &dyn Expression, value: bool) {
    let bool_literal = expr
        .downcast_ref::<BooleanLiteral>()
        .expect("Could not downcast to boolean literal");
    assert_eq!(bool_literal.value(), value);
    assert_eq!(bool_literal.token_literal(), value.to_string().as_str());
}

fn test_infix(expr: &dyn Expression, left: &dyn Any, operator: &str, right: &dyn Any) {
    let infix = expr
        .downcast_ref::<InfixExpression>()
        .expect("Could not downcast to infix");
    test_literal(infix.left.as_ref(), left);
    assert_eq!(infix.operator.as_ref(), operator);
    test_literal(infix.right.as_ref(), right);
}

#[test]
fn infix_expr() {
    let integer_inputs = [
        ("5 + 5;", 5, "+", 5),
        ("5 - 5;", 5, "-", 5),
        ("5 * 5;", 5, "*", 5),
        ("5 / 5;", 5, "/", 5),
        ("5 == 5;", 5, "==", 5),
        ("5 != 5;", 5, "!=", 5),
        ("5 < 5;", 5, "<", 5),
        ("5 > 5;", 5, ">", 5),
    ];

    for (input, left, operator, right) in integer_inputs {
        let program = match input.parse::<Program>() {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to parse program: {e}");
            }
        };

        assert_eq!(program.statements.len(), 1);

        let expr = &program.statements[0]
            .downcast_ref::<ExpressionStatement>()
            .expect("Could not downcast to expression statement")
            .expression;
        test_infix(
            expr.as_ref(),
            &(left as Integer),
            operator,
            &(right as Integer),
        )
    }

    let boolean_inputs = [
        ("true == true;", true, "==", true),
        ("true != false;", true, "!=", false),
        ("false == false;", false, "==", false),
    ];

    for (input, left, operator, right) in boolean_inputs {
        let program = match input.parse::<Program>() {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to parse program: {e}");
            }
        };

        assert_eq!(program.statements.len(), 1);

        let expr = &program.statements[0]
            .downcast_ref::<ExpressionStatement>()
            .expect("Could not downcast to expression statement")
            .expression;
        test_infix(expr.as_ref(), &left, operator, &right)
    }
}

#[test]
fn pemdas() {
    let inputs = [
        ("-a * b;", "((-a) * b);"),
        ("!-a;", "(!(-a));"),
        ("a + b + c;", "((a + b) + c);"),
        ("a + b - c;", "((a + b) - c);"),
        ("a * b * c;", "((a * b) * c);"),
        ("a * b / c;", "((a * b) / c);"),
        ("a + b / c;", "(a + (b / c));"),
        ("a + b * c + d / e - f;", "(((a + (b * c)) + (d / e)) - f);"),
        ("3 + 4; -5 * 5;", "(3 + 4);((-5) * 5);"),
        ("5 > 4 == 3 < 4;", "((5 > 4) == (3 < 4));"),
        (" 5 < 4 != 3 > 4;", "((5 < 4) != (3 > 4));"),
        (
            "3 + 4 * 5 == 3 * 1 + 4 * 5;",
            "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)));",
        ),
        ("true;", "true;"),
        ("false;", "false;"),
        ("3 > 5 == false;", "((3 > 5) == false);"),
        ("3 < 5 == true;", "((3 < 5) == true);"),
        ("1 + (2 + 3) + 4;", "((1 + (2 + 3)) + 4);"),
        ("(5 + 5) * 2;", "((5 + 5) * 2);"),
        ("2 / (5 + 5);", "(2 / (5 + 5));"),
        ("-(5 + 5);", "(-(5 + 5));"),
        ("!(true == true);", "(!(true == true));"),
    ];

    for (input, expected) in inputs {
        let program = match input.parse::<Program>() {
            Ok(program) => program,
            Err(e) => {
                panic!("Failed to parse program: {e}");
            }
        };

        assert_eq!(program.to_string(), expected);
    }
}

#[test]
fn bool_literal() {
    let input = "true;";
    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => {
            panic!("Failed to parse program: {e}");
        }
    };
    assert_eq!(program.statements.len(), 1);

    let bool_literal = program.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<BooleanLiteral>()
        .expect("Could not downcast to boolean literal");

    assert!(bool_literal.value());
    assert_eq!(bool_literal.token_literal(), "true");
}

#[test]
fn if_expr() {
    let input = "if (x < y) { x };";
    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => panic!("Failed to parse program: {e}"),
    };

    assert_eq!(program.statements.len(), 1);
    let if_expr = program.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<IfExpression>()
        .expect("Could not downcast to if expression");

    test_infix(if_expr.cond.as_ref(), &"x", "<", &"y");

    assert_eq!(if_expr.cons.statements.len(), 1);
    let cons = if_expr.cons.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement");
    test_identifier(cons.expression.as_ref(), &"x");
    assert!(if_expr.alternative.is_none());
}

#[test]
fn if_else_expr() {
    let input = "if (x < y) { x } else { y };";
    let program = match input.parse::<Program>() {
        Ok(program) => program,
        Err(e) => panic!("Failed to parse program: {e}"),
    };

    assert_eq!(program.statements.len(), 1);

    let if_expr = program.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement")
        .expression
        .downcast_ref::<IfExpression>()
        .expect("Could not downcast to if expression");

    test_infix(if_expr.cond.as_ref(), &"x", "<", &"y");

    assert_eq!(if_expr.cons.statements.len(), 1);
    let cons = if_expr.cons.statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement");
    test_identifier(cons.expression.as_ref(), &"x");
    let alternative = if_expr
        .alternative
        .as_ref()
        .expect("No alternative")
        .statements[0]
        .downcast_ref::<ExpressionStatement>()
        .expect("Could not downcast to expression statement");
    test_identifier(alternative.expression.as_ref(), &"y");
}
