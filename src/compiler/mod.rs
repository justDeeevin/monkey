use crate::{
    ast::{
        Expression, InfixOperator, Node, PrefixOperator, Program as Ast, Statement, StatementKind,
    },
    code::{Op, Program, SpannedObject},
};

#[cfg(test)]
pub mod test;

#[derive(Default)]
pub struct Compiler<'a> {
    constants: Vec<SpannedObject<'a>>,
    ops: Vec<Op<'a>>,
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
        let mut last_kind = None;
        for statement in statements {
            last_kind = Some(StatementKind::from(&statement));
            self.compile_statement(statement);
        }
        if last_kind != Some(StatementKind::Let) {
            self.ops.pop();
        }
    }

    fn compile_statement(&mut self, statement: Statement<'a>) {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.ops.push(Op::Pop)
            }
            Statement::Let { name, value, .. } => {
                self.compile_expression(value);
                self.ops.push(Op::SetGlobal(name.value));
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
                    object: value.into(),
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
            Expression::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                self.compile_expression(*condition);

                self.ops.push(Op::Panic);
                let jin_pos = self.ops.len() - 1;

                self.compile_statements(consequence.statements);

                if let Some(alternative) = alternative {
                    self.ops.push(Op::Panic);
                    let jump_pos = self.ops.len() - 1;

                    self.ops[jin_pos] = Op::JumpIfNot(self.ops.len());

                    self.compile_statements(alternative.statements);
                    self.ops[jump_pos] = Op::Jump(self.ops.len());
                } else {
                    self.ops[jin_pos] = Op::JumpIfNot(self.ops.len());
                }
            }
            Expression::Null(token) => {
                self.ops.push(Op::Null(token.span));
            }
            Expression::Identifier(name) => {
                self.ops.push(Op::GetGlobal {
                    name: name.value,
                    span: name.span(),
                });
            }
            _ => todo!(),
        }
    }
}
