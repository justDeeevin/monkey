use crate::{
    code::{Op, Program},
    compiler::Compiler,
    object::Object,
    parser::test::get_program as get_ast,
};

#[test]
fn integer_arithmetic() {
    let tests = [(
        "1 + 2",
        [Object::Integer(1), Object::Integer(2)].as_slice(),
        [Op::Constant(0), Op::Constant(1), Op::Add].as_slice(),
    )];

    check(&tests);
}

fn check(cases: &[(&str, &[Object], &[Op])]) {
    for (input, expected_constants, expected_ops) in cases {
        let program = get_program(input);
        assert_eq!(program.constants.as_ref(), *expected_constants);
        assert_eq!(program.ops.as_ref(), *expected_ops);
    }
}

pub fn get_program(input: &str) -> Program<'_> {
    let ast = get_ast(input);
    Compiler::compile(ast).unwrap_or_else(|errors| {
        panic!(
            "failed to compile program:{}",
            errors
                .into_iter()
                .fold(String::new(), |acc, e| format!("{acc}\n{e}"))
        )
    })
}
