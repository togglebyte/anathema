use anathema_templates::Expression;

use crate::context::ResolverCtx;
use crate::expression::{Kind, ResolvedExpr};

pub struct Resolver<'a, 'frame, 'bp> {
    ctx: &'a ResolverCtx<'frame, 'bp>,
}

impl<'a, 'frame, 'bp> Resolver<'a, 'frame, 'bp> {
    pub fn new(ctx: &'a ResolverCtx<'frame, 'bp>) -> Self {
        Self { ctx }
    }

    fn lookup(&self, ident: &str) -> ResolvedExpr<'bp> {
        match ident {
            "state" => {
                let Some(state_id) = self.ctx.scope.get_state() else { return ResolvedExpr::Null };
                let Some(state) = self.ctx.states.get(state_id) else { return ResolvedExpr::Null };
                let value = state.reference();
                value.into()
            }
            "attributes" => {
                let Some(component) = self.ctx.scope.get_attributes() else { return ResolvedExpr::Null };
                ResolvedExpr::Attributes(component)
            }
            ident => match self.ctx.scope.lookup(ident) {
                Some(value) => value,
                None => {
                    let Some(id) = self.ctx.variables.global_lookup(ident) else { return ResolvedExpr::Null };
                    let expr = self.ctx.expressions.get(id);
                    self.resolve(expr)
                }
            },
        }
    }

    pub(crate) fn resolve(&self, expr: &'bp Expression) -> ResolvedExpr<'bp> {
        match expr {
            Expression::Primitive(primitive) => ResolvedExpr::from(*primitive),
            Expression::Variable(var) => match self.ctx.variables.load(*var).map(|id| self.ctx.expressions.get(id)) {
                Some(expr) => self.resolve(expr),
                None => ResolvedExpr::Null,
            },
            Expression::Str(s) => ResolvedExpr::Str(Kind::Static(s)),
            Expression::List(vec) => ResolvedExpr::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Map(map) => ResolvedExpr::Map(map.iter().map(|(k, e)| (k.as_str(), self.resolve(e))).collect()),
            Expression::TextSegments(vec) => ResolvedExpr::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Not(expr) => ResolvedExpr::Not(self.resolve(expr).into()),
            Expression::Negative(expr) => ResolvedExpr::Negative(self.resolve(expr).into()),
            Expression::Equality(lhs, rhs, equality) => {
                let lhs = self.resolve(lhs);
                let rhs = self.resolve(rhs);
                ResolvedExpr::Equality(lhs.into(), rhs.into(), *equality)
            }
            Expression::LogicalOp(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                ResolvedExpr::LogicalOp(lhs, rhs, *op)
            }
            Expression::Op(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                ResolvedExpr::Op(lhs, rhs, *op)
            }
            Expression::Either(first, second) => {
                ResolvedExpr::Either(self.resolve(first).into(), self.resolve(second).into())
            }
            Expression::Ident(ident) => self.lookup(ident),
            Expression::Index(source, index) => {
                let source = self.resolve(source);
                let index = self.resolve(index);
                ResolvedExpr::Index(source.into(), index.into())
            }
            Expression::Range(from, to) => {
                let from = self.resolve(from);
                let to = self.resolve(to);
                ResolvedExpr::Range(from.into(), to.into())
            }
            Expression::Call { fun, args } => {
                match &**fun {
                    // function(args)
                    Expression::Ident(fun) => match self.ctx.lookup_function(fun) {
                        Some(fun_ptr) => {
                            let args = args.iter().map(|arg| self.resolve(arg)).collect::<Box<_>>();
                            ResolvedExpr::Call { fun_ptr, args }
                        }
                        None => ResolvedExpr::Null,
                    },
                    // some.value.function(args)
                    Expression::Index(lhs, rhs) => {
                        let first_arg = self.resolve(lhs);
                        let Expression::Str(fun) = &**rhs else { return ResolvedExpr::Null };
                        match self.ctx.lookup_function(fun) {
                            Some(fun_ptr) => {
                                let args = std::iter::once(first_arg)
                                    .chain(args.iter().map(|arg| self.resolve(arg)))
                                    .collect::<Box<_>>();
                                ResolvedExpr::Call { fun_ptr, args }
                            }
                            None => ResolvedExpr::Null,
                        }
                    }
                    _ => ResolvedExpr::Null,
                }
            }
        }
    }
}
