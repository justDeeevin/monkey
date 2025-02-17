mod test;

use crate::{
    ast::{
        BlockStatement, BooleanLiteral, ExpressionStatement, Identifier, IfExpression,
        InfixExpression, IntegerLiteral, LetStatement, PrefixExpression, Program, ReturnStatement,
        traits::{Node, Statement},
    },
    object::{Boolean, Environment, Integer, Null, ReturnValue, traits::Object},
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

pub fn eval(root: &dyn Node, env: &mut Environment) -> Result<Box<dyn Object>> {
    if let Some(program) = root.downcast_ref::<Program>() {
        eval_program(&program.statements, env)
    } else if let Some(expr) = root.downcast_ref::<ExpressionStatement>() {
        eval(expr.expression.as_ref(), env)
    } else if let Some(int) = root.downcast_ref::<IntegerLiteral>() {
        Ok(Box::new(Integer { value: int.value() }))
    } else if let Some(bool) = root.downcast_ref::<BooleanLiteral>() {
        Ok(Box::new(Boolean {
            value: bool.value(),
        }))
    } else if let Some(prefix) = root.downcast_ref::<PrefixExpression>() {
        let right = eval(prefix.right.as_ref(), env)?;
        eval_prefix(prefix.operator, right)
    } else if let Some(infix) = root.downcast_ref::<InfixExpression>() {
        let left = eval(infix.left.as_ref(), env)?;
        let right = eval(infix.right.as_ref(), env)?;
        eval_infix(&infix.operator, left, right)
    } else if let Some(block) = root.downcast_ref::<BlockStatement>() {
        eval_block(&block.statements, env)
    } else if let Some(if_expr) = root.downcast_ref::<IfExpression>() {
        eval_if(if_expr, env)
    } else if let Some(return_statement) = root.downcast_ref::<ReturnStatement>() {
        Ok(Box::new(ReturnValue {
            value: eval(return_statement.value.as_ref(), env)?,
        }))
    } else if let Some(let_statement) = root.downcast_ref::<LetStatement>() {
        let value = eval(let_statement.value.as_ref(), env)?;
        env.insert(let_statement.name.value().into(), value);
        Ok(Box::new(Null))
    } else if let Some(identifier) = root.downcast_ref::<Identifier>() {
        eval_ident(identifier, env)
    } else {
        todo!()
    }
}

fn eval_ident(identifier: &Identifier, env: &mut Environment) -> Result<Box<dyn Object>> {
    if let Some(value) = env.get(identifier.value()) {
        Ok(value.clone())
    } else {
        Err(EvalError::NotFound(identifier.value().into()))
    }
}

fn eval_block(statements: &[Box<dyn Statement>], env: &mut Environment) -> Result<Box<dyn Object>> {
    let mut out = None;
    let mut errors = Vec::new();
    for statement in statements {
        match eval(statement.as_ref(), env) {
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

fn eval_program(
    statements: &[Box<dyn Statement>],
    env: &mut Environment,
) -> Result<Box<dyn Object>> {
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

fn eval_if(if_expr: &IfExpression, env: &mut Environment) -> Result<Box<dyn Object>> {
    let cond = eval(if_expr.cond.as_ref(), env)?;
    if cond.truthy() {
        eval(&if_expr.cons, env)
    } else if let Some(alt) = if_expr.alternative.as_ref() {
        eval(alt, env)
    } else {
        Ok(Box::new(Null))
    }
}
