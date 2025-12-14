use crate::{eval::Result, object::Object};

#[allow(clippy::type_complexity, reason = "idc")]
pub fn lookup_intrinsic<'a>(name: &'a str) -> Option<fn(&[Object<'a>]) -> Result<'a, Object<'a>>> {
    match name {
        "print" => Some(print),
        _ => None,
    }
}

fn print<'a>(args: &[Object<'a>]) -> Result<'a, Object<'a>> {
    for arg in args {
        print!("{arg} ");
    }
    println!();
    Ok(Object::Null)
}
