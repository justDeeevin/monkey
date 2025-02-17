mod test;

use crate::{
    ast::{
        BooleanLiteral, ExpressionStatement, IntegerLiteral, PrefixExpression, Program,
        traits::{Node, Statement},
    },
    object::{Boolean, Integer, Null, traits::Object},
};

const TRUE: Boolean = Boolean { value: true };
const FALSE: Boolean = Boolean { value: false };

pub fn eval(root: &dyn Node) -> Box<dyn Object> {
    if let Some(program) = root.downcast_ref::<Program>() {
        eval_statements(&program.statements)
    } else if let Some(expr) = root.downcast_ref::<ExpressionStatement>() {
        eval(expr.expression.as_ref())
    } else if let Some(int) = root.downcast_ref::<IntegerLiteral>() {
        Box::new(Integer { value: int.value() })
    } else if let Some(bool) = root.downcast_ref::<BooleanLiteral>() {
        Box::new(Boolean {
            value: bool.value(),
        })
    } else if let Some(prefix) = root.downcast_ref::<PrefixExpression>() {
        let right = eval(prefix.right.as_ref());
        eval_prefix(prefix.operator, right)
    } else {
        todo!()
    }
}

fn eval_statements(statements: &[Box<dyn Statement>]) -> Box<dyn Object> {
    let mut out: Box<dyn Object> = Box::new(Null);
    for statement in statements {
        out = eval(statement.as_ref());
    }
    out
}

fn eval_prefix(operator: char, right: Box<dyn Object>) -> Box<dyn Object> {
    match operator {
        '!' => Box::new(eval_bang(right)),
        '-' => eval_neg(right),
        _ => Box::new(Null),
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

fn eval_neg(right: Box<dyn Object>) -> Box<dyn Object> {
    let Some(int) = right.downcast_ref::<Integer>() else {
        return Box::new(Null);
    };
    Box::new(Integer { value: -int.value })
}
