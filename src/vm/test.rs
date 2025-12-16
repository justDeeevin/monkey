use crate::{compiler::test::get_program, object::Object, vm::VM};

#[test]
fn integer_arithmetic() {
    let tests = [("1", 1.into()), ("2", 2.into()), ("1 + 2", 3.into())];

    check(&tests);
}

fn check(cases: &[(&str, Object)]) {
    for (input, expected) in cases {
        match VM::new(get_program(input)).run() {
            Err(e) => {
                panic!("failed to run program: {e}",);
            }
            Ok(out) => assert_eq!(out, *expected),
        }
    }
}
