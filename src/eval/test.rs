use crate::{eval::Environment, object::Object};

fn test_eval(input: &str) -> Object<'_> {
    Environment::default()
        .eval_program(crate::parser::test::get_program(input))
        .unwrap_or_else(|errors| {
            panic!(
                "failed to evaluate program:{}",
                errors
                    .into_iter()
                    .fold(String::new(), |acc, e| format!("{acc}\n{e}"))
            )
        })
}

#[test]
fn integer() {
    let tests = [("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

    for (found, expected) in tests
        .into_iter()
        .map(|(input, expected)| (test_eval(input), expected))
    {
        assert_eq!(found, Object::Integer(expected));
    }
}

#[test]
fn boolean() {
    let tests = [("true", true), ("false", false)];

    for (found, expected) in tests
        .into_iter()
        .map(|(input, expected)| (test_eval(input), expected))
    {
        assert_eq!(found, Object::Boolean(expected));
    }
}

#[test]
fn not() {
    let tests = [
        ("!true", false),
        ("!false", true),
        ("!5", false),
        ("!!true", true),
        ("!!false", false),
        ("!!5", true),
    ];

    for (found, expected) in tests
        .into_iter()
        .map(|(input, expected)| (test_eval(input), expected))
    {
        assert_eq!(found, Object::Boolean(expected));
    }
}
