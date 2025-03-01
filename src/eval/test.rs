#![cfg(test)]

use std::collections::HashMap;

use super::{EvalError, eval};
use crate::{
    ast::Integer as Int,
    eval::{FALSE, TRUE},
    object::{
        Array, Boolean, Function, Hash, Integer, Null, Scope, String as StringObject,
        traits::Object,
    },
    parser::test::new_program,
};

fn new_eval(input: &str, expected_statements: usize) -> Box<dyn Object> {
    let program = new_program(input, expected_statements);
    match eval(&program, &mut Scope::empty()) {
        Ok(eval) => eval,
        Err(e) => {
            panic!("Failed to eval program: {e}");
        }
    }
}

#[test]
fn int() {
    let tests = [
        ("5", 5),
        ("10", 10),
        ("-5", -5),
        ("-10", -10),
        ("5 + 5 + 5 + 5 - 10", 10),
        ("2 * 2 * 2 * 2 * 2", 32),
        ("-50 + 100 - 50", 0),
        ("5 * 2 + 10", 20),
        ("5 + 2 * 10", 25),
        ("20 + 2 * -10", 0),
        ("50 / 2 * 2 + 10", 60),
        ("2 * (5 + 10)", 30),
        ("3 * 3 * 3 + 10", 37),
        ("3 * (3 * 3) + 10", 37),
        ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
    ];
    for (input, expected) in tests {
        let eval = new_eval(input, 1);
        test_int(eval.as_ref(), expected);
    }
}

fn test_int(object: &dyn Object, expected: Int) {
    let int = object
        .downcast_ref::<Integer>()
        .unwrap_or_else(|| panic!("Could not downcast to integer object, got {:?}", object));
    assert_eq!(int.value, expected);
}

#[test]
fn bool() {
    let tests = [
        ("true", true),
        ("false", false),
        ("1 < 2", true),
        ("1 > 2", false),
        ("1 < 1", false),
        ("1 > 1", false),
        ("1 == 1", true),
        ("1 != 1", false),
        ("1 == 2", false),
        ("1 != 2", true),
        ("true == true", true),
        ("false == false", true),
        ("true == false", false),
        ("true != false", true),
        ("false != true", true),
        ("(1 < 2) == true", true),
        ("(1 < 2) == false", false),
        ("(1 > 2) == true", false),
        ("(1 > 2) == false", true),
    ];
    for (input, expected) in tests {
        let eval = new_eval(input, 1);
        test_bool(eval.as_ref(), expected);
    }
}

fn test_bool(object: &dyn Object, expected: bool) {
    let bool = object
        .downcast_ref::<Boolean>()
        .unwrap_or_else(|| panic!("Could not downcast to boolean object, got {:?}", object));
    assert_eq!(bool.value, expected);
}

#[test]
fn bang() {
    let tests = [
        ("!true", false),
        ("!false", true),
        ("!5", false),
        ("!!true", true),
        ("!!false", false),
        ("!!5", true),
    ];

    for (input, expected) in tests {
        let eval = new_eval(input, 1);
        test_bool(eval.as_ref(), expected);
    }
}

#[test]
fn if_else() {
    let tests = [
        ("if (true) {10}", Some(10)),
        ("if (false) {10}", None),
        ("if (1) {10}", Some(10)),
        ("if (1 < 2) {10}", Some(10)),
        ("if (1 > 2) {10}", None),
        ("if (1 > 2) {10} else {20}", Some(20)),
        ("if (1 < 2) {10} else {20}", Some(10)),
    ];

    for (input, expected) in tests {
        let eval = new_eval(input, 1);
        if let Some(expected) = expected {
            test_int(eval.as_ref(), expected);
        } else {
            test_null(eval.as_ref());
        }
    }
}

fn test_null(object: &dyn Object) {
    object
        .downcast_ref::<Null>()
        .unwrap_or_else(|| panic!("Could not downcast to null object, got {:?}", object));
}

#[test]
fn return_statements() {
    let tests = [
        (1, "return 10;", 10),
        (2, "return 10; 9;", 10),
        (2, "return 2 * 5; 9;", 10),
        (3, "9; return 2 * 5; 9;", 10),
        (
            1,
            r#"
            if (10 > 1) {
                if (10 > 1) {
                    return 10;
                }

                return 1;
            }
        "#,
            10,
        ),
    ];

    for (n_statements, input, expected) in tests {
        let eval = new_eval(input, n_statements);
        test_int(eval.as_ref(), expected);
    }
}

#[test]
fn let_statements() {
    let tests = [
        (2, "let a = 5; a;", 5),
        (2, "let a = 5 * 5; a;", 25),
        (3, "let a = 5; let b = a; b;", 5),
        (4, "let a = 5; let b = a; let c = a + b + 5; c;", 15),
    ];

    for (n_statements, input, expected) in tests {
        let eval = new_eval(input, n_statements);
        test_int(eval.as_ref(), expected);
    }
}

#[test]
fn function() {
    let input = "fn(x) { x + 2; };";
    let eval = new_eval(input, 1);
    let function = eval
        .downcast_ref::<Function>()
        .unwrap_or_else(|| panic!("Could not downcast to function, got {:?}", eval));
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0].value(), "x");
    assert_eq!(function.body.to_string(), "{ (x + 2); }");
}

#[test]
fn call() {
    let tests = [
        (2, "let identity = fn(x) { x; }; identity(5);", 5),
        (2, "let identity = fn(x) { return x; }; identity(5);", 5),
        (2, "let double = fn(x) { x * 2; }; double(5);", 10),
        (2, "let add = fn(x, y) { x + y; }; add(5, 5);", 10),
        (
            2,
            "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
            20,
        ),
        (1, "fn(x) { x; }(5)", 5),
    ];

    for (n_statements, input, expected) in tests {
        let eval = new_eval(input, n_statements);
        test_int(eval.as_ref(), expected);
    }
}

#[test]
fn string() {
    let input = "\"droddyrox\"";
    let eval = new_eval(input, 1);
    let string = eval
        .downcast_ref::<StringObject>()
        .expect("Could not downcast to string");
    assert_eq!(string.value.as_ref(), "droddyrox");
}

#[test]
fn concat_string() {
    let input = "\"droddy\" + \"rox\"";
    let eval = new_eval(input, 1);
    let string = eval
        .downcast_ref::<StringObject>()
        .expect("Could not downcast to string");
    assert_eq!(string.value.as_ref(), "droddyrox");
}

#[test]
fn len() {
    let tests = [
        ("len(\"\")", Ok(0)),
        ("len(\"four\")", Ok(4)),
        ("len(\"droddyrox\")", Ok(9)),
        (
            "len(1)",
            Err(EvalError::BadType {
                expected: "string".to_string(),
                got: "integer".to_string(),
            }
            .to_string()),
        ),
        (
            "len(\"one\", \"two\")",
            Err(EvalError::BadArity {
                expected: 1,
                got: 2,
            }
            .to_string()),
        ),
    ];

    for (input, expected) in tests {
        let program = new_program(input, 1);
        let mut scope = Scope::new();
        let eval = eval(&program, &mut scope);
        match expected {
            Ok(expected) => {
                let len = match eval {
                    Ok(len) => len,
                    Err(e) => {
                        panic!("Error evaluating: {}", e);
                    }
                };
                test_int(len.as_ref(), expected);
            }

            Err(e) => {
                let Err(EvalError::Many(errors)) = eval else {
                    panic!("Expected errors, got {:?}", eval);
                };
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].to_string(), e);
            }
        }
    }
}

#[test]
fn array() {
    let input = "[1,2 * 2, 3 + 3]";
    let eval = new_eval(input, 1);
    let array = eval
        .downcast_ref::<Array>()
        .expect("Could not downcast to array");
    assert_eq!(array.elements.len(), 3);
    test_int(array.elements[0].as_ref(), 1);
    test_int(array.elements[1].as_ref(), 4);
    test_int(array.elements[2].as_ref(), 6);
}

#[test]
fn index() {
    let tests = [
        (1, "[1, 2, 3][0]", Some(1)),
        (1, "[1, 2, 3][1]", Some(2)),
        (1, "[1, 2, 3][2]", Some(3)),
        (2, "let i = 0; [1][i];", Some(1)),
        (1, "[1, 2, 3][1 + 1];", Some(3)),
        (2, "let myArray = [1, 2, 3]; myArray[2];", Some(3)),
        (
            2,
            "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
            Some(6),
        ),
        (
            3,
            "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            Some(2),
        ),
        (1, "[1, 2, 3][3]", None),
        (1, "[1, 2, 3][-1]", None),
    ];

    for (n_statements, input, expected) in tests {
        let eval = new_eval(input, n_statements);
        if let Some(expected) = expected {
            test_int(eval.as_ref(), expected);
        } else {
            test_null(eval.as_ref());
        }
    }
}

#[test]
fn hash() {
    let input = r#"
        let two = "two";
        {
            "one": 10 - 9,
            two: 1 + 1,
            "thr" + "ee": 6 / 2,
            4: 4,
            true: 5,
            false: 6,
        }
    "#;
    let expected: HashMap<Box<dyn Object>, Int> =
        HashMap::from_iter::<[(Box<dyn Object>, Int); 6]>([
            (
                Box::new(StringObject {
                    value: "one".into(),
                }),
                1,
            ),
            (
                Box::new(StringObject {
                    value: "two".into(),
                }),
                2,
            ),
            (
                Box::new(StringObject {
                    value: "three".into(),
                }),
                3,
            ),
            (Box::new(Integer { value: 4 }), 4),
            (Box::new(TRUE), 5),
            (Box::new(FALSE), 6),
        ]);
    let eval = new_eval(input, 2);
    let hash = eval
        .downcast_ref::<Hash>()
        .expect("Could not downcast to hash");
    assert_eq!(hash.pairs.len(), expected.len());
    for (expected_k, expected_v) in expected {
        let v = hash.pairs.get(&expected_k).expect("No value found for key");
        test_int(v.as_ref(), expected_v);
    }
}

#[test]
fn index_hash() {
    let tests = [
        (1, "{\"foo\": 5}[\"foo\"]", Some(5)),
        (1, "{\"foo\": 5}[\"bar\"]", None),
        (2, "let key = \"foo\"; {\"foo\": 5}[key]", Some(5)),
        (1, "{}[\"foo\"]", None),
        (1, "{5: 5}[5]", Some(5)),
        (1, "{true: 5}[true]", Some(5)),
        (1, "{false: 5}[false]", Some(5)),
    ];
    for (n_statements, input, expected) in tests {
        let eval = new_eval(input, n_statements);
        match expected {
            Some(expected) => {
                test_int(eval.as_ref(), expected);
            }
            None => {
                test_null(eval.as_ref());
            }
        }
    }
}
