use crate::{
    eval::{Error, ErrorKind, Result},
    object::Object,
    token::Span,
};

#[allow(clippy::type_complexity, reason = "idc")]
pub fn lookup_intrinsic<'a>(
    name: &'a str,
) -> Option<fn(&[Object<'a>], Span) -> Result<'a, Object<'a>>> {
    match name {
        "print" => Some(print),
        "len" => Some(len),
        "first" => Some(first),
        "last" => Some(last),
        "rest" => Some(rest),
        "push" => Some(push),
        _ => None,
    }
}

fn print<'a>(args: &[Object<'a>], _args_span: Span) -> Result<'a, Object<'a>> {
    for arg in args {
        print!("{arg} ");
    }
    println!();
    Ok(Object::Null)
}

fn len<'a>(args: &[Object<'a>], args_span: Span) -> Result<'a, Object<'a>> {
    let [arg] = args else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 1,
                got: args.len(),
            },
        });
    };

    let arg_span = Span {
        start: args_span.start + 1,
        end: args_span.end - 1,
    };

    let too_long_error = Error {
        span: arg_span,
        kind: ErrorKind::TooLongForLen,
    };

    match arg {
        Object::Array(a) => {
            if a.len() > i64::MAX as usize {
                Err(too_long_error)
            } else {
                Ok(Object::Integer(a.len() as i64))
            }
        }
        Object::Map(m) => {
            if m.len() > i64::MAX as usize {
                Err(too_long_error)
            } else {
                Ok(Object::Integer(m.len() as i64))
            }
        }
        Object::String(s) => {
            if s.len() > i64::MAX as usize {
                Err(too_long_error)
            } else {
                Ok(Object::Integer(s.len() as i64))
            }
        }
        arg => Err(Error {
            span: arg_span,
            kind: ErrorKind::BadTypeForLen(arg.into()),
        }),
    }
}

fn first<'a>(args: &[Object<'a>], args_span: Span) -> Result<'a, Object<'a>> {
    let [arg] = args else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 1,
                got: args.len(),
            },
        });
    };
    let Object::Array(a) = arg else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::NotAnArray,
        });
    };
    Ok(a.first().cloned().unwrap_or(Object::Null))
}

fn last<'a>(args: &[Object<'a>], args_span: Span) -> Result<'a, Object<'a>> {
    let [arg] = args else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 1,
                got: args.len(),
            },
        });
    };
    let Object::Array(a) = arg else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::NotAnArray,
        });
    };
    Ok(a.last().cloned().unwrap_or(Object::Null))
}

fn rest<'a>(args: &[Object<'a>], args_span: Span) -> Result<'a, Object<'a>> {
    let [arg] = args else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 1,
                got: args.len(),
            },
        });
    };
    let Object::Array(a) = arg else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::NotAnArray,
        });
    };
    Ok(Object::Array(a.iter().skip(1).cloned().collect()))
}

fn push<'a>(args: &[Object<'a>], args_span: Span) -> Result<'a, Object<'a>> {
    let [arg, value] = args else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::WrongNumberOfArguments {
                expected: 2,
                got: args.len(),
            },
        });
    };

    let Object::Array(mut a) = arg.clone() else {
        return Err(Error {
            span: args_span,
            kind: ErrorKind::NotAnArray,
        });
    };

    a.push(value.clone());

    Ok(Object::Array(a))
}
