use crate::object::{Integer, Null, String as StringObject, traits::Object};

use super::EvalError;

pub fn len(args: &[Box<dyn Object>]) -> super::Result<Box<dyn Object>> {
    if args.len() != 1 {
        return Err(EvalError::BadArity {
            expected: 1,
            got: args.len(),
        });
    }
    let Some(string) = args[0].downcast_ref::<StringObject>() else {
        return Err(EvalError::BadType {
            expected: "string".to_string(),
            got: args[0].type_name().to_string(),
        });
    };

    Ok(Box::new(Integer {
        value: string.value.len() as i64,
    }))
}

pub fn puts(args: &[Box<dyn Object>]) -> super::Result<Box<dyn Object>> {
    for arg in args {
        if let Some(string) = arg.downcast_ref::<StringObject>() {
            println!("{}", string.value);
        } else {
            println!("{}", arg);
        }
    }

    Ok(Box::new(Null))
}
