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
