use crate::object::{Integer, String as StringObject, traits::Object};

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
