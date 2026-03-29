use crate::{
    ast::Span,
    eval::{Error, ErrorKind, Result},
    value::Value,
};

pub type Intrinsic<'a> = fn(Span, Vec<Value<'a>>) -> Result<'a, Value<'a>>;

pub fn find_intrinsic(name: &str) -> Option<Intrinsic<'_>> {
    match name {
        "print" => Some(print),
        "dbg" => Some(dbg),
        _ => None,
    }
}

fn print<'a>(_call_span: Span, args: Vec<Value<'a>>) -> Result<'a, Value<'a>> {
    for arg in args {
        println!("{arg}");
    }
    Ok(Value::Null)
}

fn dbg<'a>(call_span: Span, args: Vec<Value<'a>>) -> Result<'a, Value<'a>> {
    if args.len() != 1 {
        return Err(Error {
            span: call_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 1,
                found: args.len(),
            },
        });
    }
    println!("{}", args[0]);
    Ok(args.into_iter().next().unwrap())
}
