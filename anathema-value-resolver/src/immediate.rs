use anathema_templates::Expression;

use crate::context::ResolverCtx;
use crate::expression::{Kind, ValueExpr};

pub struct Resolver<'a, 'frame, 'bp> {
    ctx: &'a ResolverCtx<'frame, 'bp>,
}

impl<'a, 'frame, 'bp> Resolver<'a, 'frame, 'bp> {
    pub fn new(ctx: &'a ResolverCtx<'frame, 'bp>) -> Self {
        Self { ctx }
    }

    fn lookup(&self, ident: &str) -> ValueExpr<'bp> {
        match ident {
            "state" => {
                // TODO: filthy unwraps all over this function
                let state_id = self.ctx.scope.get_state().unwrap();
                // TODO: There is yet to be a requirement for a state in the root
                //       so this unwrap can't become an expect until that's in place
                let state = self.ctx.states.get(state_id).unwrap();
                let value = state.reference();
                value.into()
            }
            "attributes" => {
                let component = self.ctx.scope.get_attributes().unwrap();
                ValueExpr::Attributes(component)
            }
            ident => match self.ctx.scope.lookup(ident) {
                Some(value) => value,
                None => {
                    let Some(expr) = self.ctx.globals.get(ident) else { return ValueExpr::Null };
                    self.resolve(expr)
                }
            },
        }
    }

    pub(crate) fn resolve(&self, expr: &'bp Expression) -> ValueExpr<'bp> {
        match expr {
            Expression::Primitive(primitive) => ValueExpr::from(*primitive),
            Expression::Str(s) => ValueExpr::Str(Kind::Static(s)),
            Expression::List(vec) => ValueExpr::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Map(map) => ValueExpr::Map(map.iter().map(|(k, e)| (k.as_str(), self.resolve(e))).collect()),
            Expression::TextSegments(vec) => ValueExpr::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Not(expr) => ValueExpr::Not(self.resolve(expr).into()),
            Expression::Negative(expr) => ValueExpr::Negative(self.resolve(expr).into()),
            Expression::Equality(lhs, rhs, equality) => {
                let lhs = self.resolve(lhs);
                let rhs = self.resolve(rhs);
                ValueExpr::Equality(lhs.into(), rhs.into(), *equality)
            }
            Expression::LogicalOp(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                ValueExpr::LogicalOp(lhs, rhs, *op)
            }
            Expression::Op(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                ValueExpr::Op(lhs, rhs, *op)
            }
            Expression::Either(first, second) => {
                ValueExpr::Either(self.resolve(first).into(), self.resolve(second).into())
            }
            Expression::Ident(ident) => self.lookup(ident),
            Expression::Index(source, index) => {
                let source = self.resolve(source);
                let index = self.resolve(index);
                ValueExpr::Index(source.into(), index.into())
            }
            Expression::Call { fun, args } => {
                match &**fun {
                    // function(args)
                    Expression::Ident(fun) => match self.ctx.lookup_function(fun) {
                        Some(fun_ptr) => {
                            let args = args.iter().map(|arg| self.resolve(arg)).collect::<Box<_>>();
                            ValueExpr::Call { fun_ptr, args }
                        }
                        None => ValueExpr::Null,
                    },
                    // some.value.function(args)
                    Expression::Index(lhs, rhs) => {
                        let first_arg = self.resolve(lhs);
                        let Expression::Str(fun) = &**rhs else { return ValueExpr::Null };
                        match self.ctx.lookup_function(fun) {
                            Some(fun_ptr) => {
                                let args = std::iter::once(first_arg)
                                    .chain(args.iter().map(|arg| self.resolve(arg)))
                                    .collect::<Box<_>>();
                                ValueExpr::Call { fun_ptr, args }
                            }
                            None => ValueExpr::Null,
                        }
                    }
                    _ => ValueExpr::Null,
                }
            }
        }
    }
}
