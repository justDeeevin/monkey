use std::rc::Rc;

use crate::{
    ast::{Expression, Program as Ast, Statement},
    code::{Op, Program, SpannedObject},
    object::Object,
};

#[cfg(test)]
pub mod test;

#[derive(Default)]
pub struct Compiler<'a> {
    constants: Vec<SpannedObject<'a>>,
    ops: Vec<Op>,
}

impl<'a> Compiler<'a> {
    pub fn compile(program: Ast<'a>) -> Program<'a> {
        let mut compiler = Compiler::default();

        compiler.compile_statements(program.statements);

        Program {
            ops: compiler.ops.into(),
            constants: compiler.constants.into(),
        }
    }

    fn compile_statements(&mut self, statements: Vec<Statement<'a>>) {
        for statement in statements {
            self.compile_statement(statement);
        }
    }

    fn compile_statement(&mut self, statement: Statement<'a>) {
        match statement {
            Statement::Expression(expr) => self.compile_expression(expr),
            _ => todo!(),
        }
    }

    fn compile_expression(&mut self, expr: Expression<'a>) {
        match expr {
            Expression::Infix {
                left,
                operator: _,
                right,
            } => {
                self.compile_expression(*left);
                self.compile_expression(*right);
                self.ops.push(Op::Add);
            }
            Expression::Integer { value, token } => {
                self.constants.push(SpannedObject {
                    object: Rc::new(Object::Integer(value)),
                    span: token.span,
                });
                self.ops.push(Op::Constant(self.constants.len() - 1));
            }
            _ => todo!(),
        }
    }
}
