use crate::{
    ast::{
        Expression, InfixOperator, Node, PrefixOperator, Program as Ast, Statement, StatementKind,
    },
    code::{Op, Program, SpannedObject},
    object::{CompiledFunction, Object},
    token::Span,
};

#[cfg(test)]
pub mod test;

#[derive(Default)]
struct Scope<'a> {
    ops: Vec<Op<'a>>,
}

#[derive(Default)]
pub struct Compiler<'a> {
    constants: Vec<SpannedObject<'a>>,
    scopes: Vec<Scope<'a>>,
}

impl<'a> Compiler<'a> {
    fn current_scope(&mut self) -> &mut Scope<'a> {
        self.scopes.last_mut().unwrap()
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn leave_scope(&mut self) -> Vec<Op<'a>> {
        self.scopes.pop().unwrap().ops
    }

    fn add_constant(&mut self, value: impl Into<Object<'a>>, span: Span) {
        self.constants.push(SpannedObject {
            object: value.into(),
            span,
        });
        let const_i = self.constants.len() - 1;
        self.current_scope().ops.push(Op::Constant(const_i));
    }

    pub fn compile(&mut self, program: Ast<'a>) -> Program<'a> {
        self.scopes.push(Scope::default());
        self.compile_statements(program.statements);
        self.current_scope().ops.pop();

        Program {
            ops: self.scopes.pop().unwrap().ops.into(),
            constants: self.constants.clone().into(),
        }
    }

    fn compile_statements(&mut self, statements: Vec<Statement<'a>>) {
        let mut last_kind = None;
        for statement in statements {
            last_kind = Some(StatementKind::from(&statement));
            self.compile_statement(statement);
            if last_kind == Some(StatementKind::Return) {
                break;
            }
        }
        if last_kind == Some(StatementKind::Expression) {
            self.current_scope().ops.pop();
        }
        if last_kind.is_none_or(|k| k == StatementKind::Let) {
            self.current_scope().ops.push(Op::Return);
        } else {
            self.current_scope().ops.push(Op::ReturnValue);
        }
    }

    fn compile_statement(&mut self, statement: Statement<'a>) {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.current_scope().ops.push(Op::Pop)
            }
            Statement::Let { name, value, .. } => {
                self.compile_expression(value);
                self.current_scope().ops.push(Op::Bind(name.value));
            }
            Statement::Return { value, .. } => {
                self.compile_expression(value);
            }
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
                self.current_scope().ops.push(operator.into());
            }
            Expression::Integer { value, token } => {
                self.add_constant(value, token.span);
            }
            Expression::Boolean { value, token } => {
                self.current_scope().ops.push(if value {
                    Op::True(token.span)
                } else {
                    Op::False(token.span)
                });
            }
            Expression::Prefix {
                operator, operand, ..
            } => {
                self.compile_expression(*operand);
                self.current_scope().ops.push(match operator {
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

                self.current_scope().ops.push(Op::Panic);
                let jin_pos = self.current_scope().ops.len() - 1;

                self.compile_statements(consequence.statements);

                if let Some(alternative) = alternative {
                    self.current_scope().ops.push(Op::Panic);
                    let jump_pos = self.current_scope().ops.len() - 1;

                    self.current_scope().ops[jin_pos] =
                        Op::JumpIfNot(self.current_scope().ops.len());

                    self.compile_statements(alternative.statements);
                    self.current_scope().ops[jump_pos] = Op::Jump(self.current_scope().ops.len());
                } else {
                    self.current_scope().ops[jin_pos] =
                        Op::JumpIfNot(self.current_scope().ops.len());
                }
            }
            Expression::Null(token) => {
                self.current_scope().ops.push(Op::Null(token.span));
            }
            Expression::Identifier(name) => {
                self.current_scope().ops.push(Op::Get {
                    name: name.value,
                    span: name.span(),
                });
            }
            Expression::String { value, token } => {
                self.add_constant(value, token.span);
            }
            Expression::Array {
                elements,
                open,
                close,
            } => {
                let size = elements.len();
                for element in elements.into_iter().rev() {
                    self.compile_expression(element);
                }
                self.current_scope().ops.push(Op::Array {
                    size,
                    span: open.span.join(close.span),
                })
            }
            Expression::Map {
                elements,
                open,
                close,
            } => {
                let size = elements.len();
                for (key, value) in elements.into_iter().rev() {
                    self.compile_expression(value);
                    self.compile_expression(key);
                }
                self.current_scope().ops.push(Op::Map {
                    size,
                    span: open.span.join(close.span),
                })
            }
            Expression::Index {
                collection,
                index,
                close,
            } => {
                let span = collection.span().join(close.span);
                self.compile_expression(*collection);
                self.compile_expression(*index);
                self.current_scope().ops.push(Op::Index(span));
            }
            Expression::Function {
                body,
                fn_token,
                parameters,
            } => {
                self.enter_scope();
                let span = fn_token.span.join(body.span());
                self.compile_statements(body.statements);
                let ops = self.leave_scope();
                self.add_constant(
                    CompiledFunction {
                        ops: ops.into(),
                        params: parameters.into_iter().map(|p| p.value).collect(),
                    },
                    span,
                );
            }
            Expression::Call {
                function,
                close,
                arguments,
                open,
            } => {
                let span = function.span().join(close.span);
                for argument in arguments {
                    self.compile_expression(argument);
                }
                self.compile_expression(*function);
                self.current_scope().ops.push(Op::Call {
                    call_span: span,
                    args_span: open.span.join(close.span),
                });
            }
        }
    }
}
