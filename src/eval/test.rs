#![cfg(test)]

use super::eval;
use crate::{
    ast::Integer as Int,
    object::{Boolean, Integer, traits::Object},
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
        .expect("Could not downcast to integer object");
    assert_eq!(int.value, expected);
}

#[test]
fn bool() {
    let tests = [("true", true), ("false", false)];
    for (input, expected) in tests {
        let eval = eval(&new_program(input, 1));
        test_bool(eval.as_ref(), expected);
    }
}

fn test_bool(object: &dyn Object, expected: bool) {
    let bool = object
        .downcast_ref::<Boolean>()
        .expect("Could not downcast to integer object");
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
