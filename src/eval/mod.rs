mod test;

use crate::{
    ast::{
        BlockStatement, BooleanLiteral, CallExpression, ExpressionStatement, FunctionLiteral,
        Identifier, IfExpression, InfixExpression, IntegerLiteral, LetStatement, PrefixExpression,
        Program, ReturnStatement, StringLiteral,
        traits::{Expression, Node, Statement},
    },
    object::{
        Boolean, Function, Integer, Null, ReturnValue, Scope, String as StringObject,
        traits::Object,
    },
};
use std::rc::Rc;

const TRUE: Boolean = Boolean { value: true };
const FALSE: Boolean = Boolean { value: false };

type Result<T> = std::result::Result<T, EvalError>;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("{0}")]
    Many(EvalErrorList),
    #[error("Operator {operator} cannot be used between {left} and {right}")]
    BadInfix {
        operator: Rc<str>,
        left: Box<dyn Object>,
        right: Box<dyn Object>,
    },
    #[error("Prefix operator {operator} cannot be used with {right}")]
    BadPrefix {
        operator: char,
        right: Box<dyn Object>,
    },
    #[error("Identifier {0} not found")]
    NotFound(Rc<str>),
    #[error("Cannot call {0}")]
    CannotCall(Box<dyn Object>),
}

#[derive(Debug, thiserror::Error)]
pub struct EvalErrorList(Vec<EvalError>);

impl std::fmt::Display for EvalErrorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            writeln!(f, "{error}")?;
        }
        Ok(())
    }
}

macro_rules! match_type {
    ($object:expr; $($name:ident as $type:ty => $do:expr),+) => {
        $(
            if let Some($name) = $object.downcast_ref::<$type>() {
                return $do;
            }
        )*
    }
}

pub fn eval(root: &dyn Node, scope: &mut Scope) -> Result<Box<dyn Object>> {
    match_type! {
        root;

        program as Program => eval_program(&program.statements, scope),
        expr as ExpressionStatement => eval(expr.expression.as_ref(), scope),
        int as IntegerLiteral  => Ok(Box::new(Integer { value: int.value() })),
        bool as BooleanLiteral => Ok(Box::new(Boolean { value: bool.value() })),
        prefix as PrefixExpression  => {
            let right = eval(prefix.right.as_ref(), scope)?;
            eval_prefix(prefix.operator, right)
        },
        infix as InfixExpression => {
            let left = eval(infix.left.as_ref(), scope)?;
            let right = eval(infix.right.as_ref(), scope)?;
            eval_infix(&infix.operator, left, right)
        },
        block as BlockStatement => eval_block(&block.statements, scope),
        if_expr as IfExpression => eval_if(if_expr, scope),
        return_statement as ReturnStatement => Ok(Box::new(ReturnValue {
            value: eval(return_statement.value.as_ref(), scope)?,
        })),
        let_statement as LetStatement => {
            let mut value = eval(let_statement.value.as_ref(), scope)?;
            if let Some(function) = value.downcast_mut::<Function>() {
                function.name = Some(let_statement.name.clone());
            }
            scope.insert(let_statement.name.value().into(), value);
            Ok(Box::new(Null))
        },
        ident as Identifier => eval_ident(ident, scope),
        function as FunctionLiteral => Ok(Box::new(Function {
            name: None,
            parameters: function.parameters.clone(),
            body: function.body.clone(),
            scope: scope.clone(),
        })),
        call as CallExpression => {
            let function = eval(call.function.as_ref(), scope)?;
            let args = eval_expressions(&call.arguments, scope)?;
            call_function(function, &args)
        },
        string as StringLiteral => Ok(Box::new(StringObject {
            value: string.rc()
        }))
    };
    Ok(Box::new(Null))
}

fn call_function(function: Box<dyn Object>, args: &[Box<dyn Object>]) -> Result<Box<dyn Object>> {
    let mut function = function
        .downcast::<Function>()
        .map_err(EvalError::CannotCall)?;
    function.scope.extend(
        function
            .parameters
            .iter()
            .zip(args)
            .map(|(i, a)| (i.value().into(), a.clone())),
    );
    if let Some(name) = function.name.clone() {
        function.scope.insert(name.value().into(), function.clone());
    }
    let eval = eval(&function.body, &mut function.scope.clone())?;
    match eval.downcast::<ReturnValue>() {
        Ok(return_value) => Ok(return_value.value),
        Err(eval) => Ok(eval),
    }
}

fn eval_expressions(
    expressions: &[Box<dyn Expression>],
    env: &mut Scope,
) -> Result<Vec<Box<dyn Object>>> {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    for expression in expressions {
        match eval(expression.as_ref(), env) {
            Ok(value) => out.push(value),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        Err(EvalError::Many(EvalErrorList(errors)))
    } else {
        Ok(out)
    }
}

fn eval_ident(identifier: &Identifier, env: &mut Scope) -> Result<Box<dyn Object>> {
    if let Some(value) = env.get(identifier.value()) {
        Ok(value.clone())
    } else {
        Err(EvalError::NotFound(identifier.value().into()))
    }
}

fn eval_block(statements: &[Box<dyn Statement>], scope: &mut Scope) -> Result<Box<dyn Object>> {
    let mut out = None;
    let mut errors = Vec::new();
    // Create a new scope for the block so variables are not leaked
    let mut scope = scope.clone();
    for statement in statements {
        match eval(statement.as_ref(), &mut scope) {
            Ok(value) => out = Some(value),
            Err(e) => errors.push(e),
        }
        if out
            .as_ref()
            .map(|o| o.downcast_ref::<ReturnValue>().is_some())
            == Some(true)
        {
            return Ok(out.unwrap());
        }
    }
    if !errors.is_empty() {
        Err(EvalError::Many(EvalErrorList(errors)))
    } else {
        Ok(out.unwrap_or_else(|| Box::new(Null)))
    }
}

fn eval_program(statements: &[Box<dyn Statement>], env: &mut Scope) -> Result<Box<dyn Object>> {
    let mut out: Box<dyn Object> = Box::new(Null);
    let mut errors = Vec::new();
    for statement in statements {
        match eval(statement.as_ref(), env) {
            Ok(value) => out = value,
            Err(e) => errors.push(e),
        }
        if out.downcast_ref::<ReturnValue>().is_some() {
            return Ok(out.downcast::<ReturnValue>().unwrap().value);
        }
    }
    if !errors.is_empty() {
        Err(EvalError::Many(EvalErrorList(errors)))
    } else {
        Ok(out)
    }
}

fn eval_prefix(operator: char, right: Box<dyn Object>) -> Result<Box<dyn Object>> {
    match operator {
        '!' => Ok(Box::new(eval_bang(right))),
        '-' => eval_neg(right),
        // Probably...
        _ => unreachable!(),
    }
}

fn eval_bang(right: Box<dyn Object>) -> Boolean {
    if let Some(bool) = right.downcast_ref::<Boolean>() {
        Boolean { value: !bool.value }
    } else if right.downcast_ref::<Null>().is_some() {
        TRUE
    } else {
        FALSE
    }
}

fn eval_neg(right: Box<dyn Object>) -> Result<Box<dyn Object>> {
    let Some(int) = right.downcast_ref::<Integer>() else {
        return Err(EvalError::BadPrefix {
            operator: '-',
            right,
        });
    };
    Ok(Box::new(Integer { value: -int.value }))
}

fn eval_infix(
    operator: impl AsRef<str>,
    left: Box<dyn Object>,
    right: Box<dyn Object>,
) -> Result<Box<dyn Object>> {
    if left.downcast_ref::<Integer>().is_some() && right.downcast_ref::<Integer>().is_some() {
        eval_int_infix(
            operator,
            left.downcast().unwrap(),
            right.downcast().unwrap(),
        )
    } else if left.downcast_ref::<Boolean>().is_some() && right.downcast_ref::<Boolean>().is_some()
    {
        let (left, right) = (
            left.downcast::<Boolean>().unwrap(),
            right.downcast::<Boolean>().unwrap(),
        );
        match operator.as_ref() {
            "==" => Ok(Box::new(Boolean {
                value: left.value == right.value,
            })),
            "!=" => Ok(Box::new(Boolean {
                value: left.value != right.value,
            })),
            _ => Err(EvalError::BadInfix {
                operator: operator.as_ref().into(),
                left,
                right,
            }),
        }
    } else if left.downcast_ref::<Null>().is_some() && right.downcast_ref::<Null>().is_some() {
        Ok(Box::new(TRUE))
    } else if let (Some(left), Some(right), "+") = (
        left.downcast_ref::<StringObject>(),
        right.downcast_ref::<StringObject>(),
        operator.as_ref(),
    ) {
        Ok(Box::new(StringObject {
            value: Rc::from(format!("{}{}", left.value, right.value)),
        }))
    } else if operator.as_ref() == "==" {
        Ok(Box::new(FALSE))
    } else if operator.as_ref() == "!=" {
        Ok(Box::new(TRUE))
    } else {
        Err(EvalError::BadInfix {
            operator: operator.as_ref().into(),
            left,
            right,
        })
    }
}

fn eval_int_infix(
    operator: impl AsRef<str>,
    left: Box<Integer>,
    right: Box<Integer>,
) -> Result<Box<dyn Object>> {
    Ok(match operator.as_ref() {
        "+" => Box::new(Integer {
            value: left.value + right.value,
        }),
        "-" => Box::new(Integer {
            value: left.value - right.value,
        }),
        "*" => Box::new(Integer {
            value: left.value * right.value,
        }),
        "/" => Box::new(Integer {
            value: left.value / right.value,
        }),
        "<" => Box::new(Boolean {
            value: left.value < right.value,
        }),
        ">" => Box::new(Boolean {
            value: left.value > right.value,
        }),
        "==" => Box::new(Boolean {
            value: left.value == right.value,
        }),
        "!=" => Box::new(Boolean {
            value: left.value != right.value,
        }),
        // should be, at least...
        _ => {
            return Err(EvalError::BadInfix {
                operator: operator.as_ref().into(),
                left,
                right,
            });
        }
    })
}

fn eval_if(if_expr: &IfExpression, env: &mut Scope) -> Result<Box<dyn Object>> {
    let cond = eval(if_expr.cond.as_ref(), env)?;
    if cond.truthy() {
        eval(&if_expr.cons, env)
    } else if let Some(alt) = if_expr.alternative.as_ref() {
        eval(alt, env)
    } else {
        Ok(Box::new(Null))
    }
}
