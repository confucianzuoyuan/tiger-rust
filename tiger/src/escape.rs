use std::rc::Rc;

use ast::{
    Declaration, DeclarationWithPos, Expr, ExprWithPos, FuncDeclaration, Operator, Var, VarWithPos,
};
use position::WithPos;
use symbol::{Strings, Symbols};

pub struct DepthEscape {
    depth: u32,
    pub escape: bool,
}

pub type EscapeEnv = Symbols<DepthEscape>;

struct EscapeFinder {
    env: EscapeEnv,
}

impl EscapeFinder {
    fn new(strings: Rc<Strings>) -> Self {
        Self {
            env: Symbols::new(strings),
        }
    }

    fn visit_binary_op(&mut self, left: &ExprWithPos, right: &ExprWithPos, depth: u32) {
        self.visit_exp(left, depth);
        self.visit_exp(right, depth);
    }

    fn visit_dec(&mut self, declaration: &DeclarationWithPos, depth: u32) {
        match declaration.node {
            Declaration::Function(ref declarations) => {
                for &WithPos {
                    node:
                        FuncDeclaration {
                            ref params,
                            ref body,
                            ..
                        },
                    ..
                } in declarations
                {
                    for param in params {
                        self.env.enter(
                            param.node.name,
                            DepthEscape {
                                depth,
                                escape: false,
                            },
                        );
                    }
                    self.visit_exp(body, depth + 1);
                }
            }
            Declaration::Type(_) => (),
            Declaration::VariableDeclaration { ref init, name, .. } => {
                self.visit_exp(init, depth + 1);
                self.env.enter(
                    name,
                    DepthEscape {
                        depth,
                        escape: false,
                    },
                );
            }
        }
    }

    fn visit_exp(&mut self, expr: &ExprWithPos, depth: u32) {
        match expr.node {
            Expr::Array {
                ref init, ref size, ..
            } => {
                self.visit_exp(size, depth);
                self.visit_exp(init, depth);
            }
            Expr::Assign { ref expr, ref var } => {
                self.visit_var(var, depth);
                self.visit_exp(expr, depth);
            }
            Expr::Break => {}
            Expr::Call { ref args, .. } => {
                for arg in args {
                    self.visit_exp(arg, depth);
                }
            }
            Expr::If {
                ref else_,
                ref test,
                ref then,
            } => {
                self.visit_exp(test, depth);
                self.visit_exp(then, depth);
                if let Some(ref else_) = *else_ {
                    self.visit_exp(&else_, depth);
                }
            }
            Expr::Int { .. } => (),
            Expr::Let {
                ref body,
                ref declarations,
            } => {
                for declaration in declarations {
                    self.visit_dec(declaration, depth);
                }
                self.visit_exp(body, depth);
            }
            Expr::Nil => (),
            Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Plus,
                        ..
                    },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Minus,
                        ..
                    },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Times,
                        ..
                    },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper: WithPos {
                    node: Operator::Lt, ..
                },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper: WithPos {
                    node: Operator::Gt, ..
                },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::And,
                        ..
                    },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper: WithPos {
                    node: Operator::Or, ..
                },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper: WithPos {
                    node: Operator::Ge, ..
                },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper: WithPos {
                    node: Operator::Le, ..
                },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Divide,
                        ..
                    },
                ref right,
            } => self.visit_binary_op(left, right, depth),
            Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Equal,
                        ..
                    },
                ref right,
            }
            | Expr::Oper {
                ref left,
                oper:
                    WithPos {
                        node: Operator::Neq,
                        ..
                    },
                ref right,
            } => {
                self.visit_exp(left, depth);
                self.visit_exp(right, depth);
            }
            Expr::Record { ref fields, .. } => {
                for field in fields {
                    self.visit_exp(&field.node.expr, depth);
                }
            }
            Expr::Sequence(ref exprs) => {
                if let Some((last_expr, exprs)) = exprs.split_last() {
                    for expr in exprs {
                        self.visit_exp(expr, depth);
                    }
                    self.visit_exp(last_expr, depth)
                }
            }
            Expr::Str { .. } => (),
            Expr::Variable(ref var) => self.visit_var(var, depth),
            Expr::While { ref body, ref test } => {
                self.visit_exp(test, depth);
                self.visit_exp(body, depth);
            }
        }
    }

    fn visit_var(&mut self, var: &VarWithPos, depth: u32) {
        match var.node {
            Var::Field { ref ident, .. } | Var::Simple { ref ident } => {
                if let Some(ref mut var) = self.env.look_mut(ident.node) {
                    if depth > var.depth {
                        var.escape = true;
                    }
                }
            }
            Var::Subscript { ref expr, ref this } => {
                self.visit_var(this, depth);
                self.visit_exp(expr, depth);
            }
        }
    }
}

pub fn find_escapes(exp: &ExprWithPos, strings: Rc<Strings>) -> EscapeEnv {
    let mut finder = EscapeFinder::new(strings);
    finder.visit_exp(exp, 0);
    finder.env
}
