use std::rc::Rc;

use crate::{
    ast::{Expression, InfixOperator, Node, PrefixOperator, Program as Ast, Statement},
    code::{Op, Program, SpannedObject},
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
        self.ops.pop();
    }

    fn compile_statement(&mut self, statement: Statement<'a>) {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.ops.push(Op::Pop)
            }
            _ => todo!(),
        }
    }

    fn compile_expression(&mut self, expr: Expression<'a>) {
        let span = expr.span();
        match expr {
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                if operator == InfixOperator::LT {
                    self.compile_expression(*right);
                    self.compile_expression(*left);
                } else {
                    self.compile_expression(*left);
                    self.compile_expression(*right);
                }
                self.ops.push(operator.into());
            }
            Expression::Integer { value, token } => {
                self.constants.push(SpannedObject {
                    object: Rc::new(value.into()),
                    span: token.span,
                });
                self.ops.push(Op::Constant(self.constants.len() - 1));
            }
            Expression::Boolean { value, token } => {
                self.ops.push(if value {
                    Op::True(token.span)
                } else {
                    Op::False(token.span)
                });
            }
            Expression::Prefix {
                operator, operand, ..
            } => {
                self.compile_expression(*operand);
                self.ops.push(match operator {
                    PrefixOperator::Not => Op::Not(span),
                    PrefixOperator::Neg => Op::Neg(span),
                });
            }
            _ => todo!(),
        }
    }
}
