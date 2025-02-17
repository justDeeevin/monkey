#![cfg(test)]

use super::eval;
use crate::{
    ast::Integer as Int,
    object::{Boolean, Integer, Null, traits::Object},
    parser::test::new_program,
};

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
        let eval = eval(&new_program(input, 1));
        test_int(eval.as_ref(), expected);
    }
}

fn test_int(object: &dyn Object, expected: Int) {
    let int = object
        .downcast_ref::<Integer>()
        .expect(format!("Could not downcast to integer object, got {:?}", object).as_str());
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
        let eval = eval(&new_program(input, 1));
        test_bool(eval.as_ref(), expected);
    }
}

fn test_bool(object: &dyn Object, expected: bool) {
    let bool = object
        .downcast_ref::<Boolean>()
        .expect(format!("Could not downcast to boolean object, got {:?}", object).as_str());
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
        let eval = eval(&new_program(input, 1));
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
        let eval = eval(&new_program(input, 1));
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
        .expect(format!("Could not downcast to null object, got {:?}", object).as_str());
}
