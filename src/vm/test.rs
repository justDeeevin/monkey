use crate::{compiler::test::get_program, object::Object, vm::VM};

#[test]
fn integer_arithmetic() {
    let tests = [
        ("1", Object::Integer(1)),
        ("2", Object::Integer(2)),
        ("1 + 2", Object::Integer(3)),
    ];

    check(&tests);
}

fn check(cases: &[(&str, Object)]) {
    for (input, expected) in cases {
        match VM::new(get_program(input)).run() {
            Err(errors) => {
                panic!(
                    "failed to run program:{}",
                    errors
                        .into_iter()
                        .fold(String::new(), |acc, e| format!("{acc}\n{e}"))
                );
            }
            Ok(out) => assert_eq!(out.as_ref(), expected),
        }
    }
}
