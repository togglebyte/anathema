use std::marker::PhantomData;

use anathema_state::{Number, Path, Subscriber, Type};
use anathema_strings::HStrings;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::{Expression, Primitive};

use crate::context::ResolverCtx;
use crate::expression::{Kind, ValueExpr};
use crate::null::Null;
use crate::scope::Lookup;
use crate::Resolver;

pub struct ImmediateResolver<'a, 'frame, 'bp> {
    ctx: &'a ResolverCtx<'frame, 'bp>,
}

impl<'a, 'frame, 'bp> ImmediateResolver<'a, 'frame, 'bp> {
    pub fn new(ctx: &'a ResolverCtx<'frame, 'bp>) -> Self {
        Self { ctx }
    }

    fn lookup(&self, ident: &str) -> ValueExpr<'bp> {
        match ident {
            "state" => {
                // TODO: filthy unwraps all over this function
                let state_id = self.ctx.scopes.get_state().unwrap();
                // TODO: There is yet to be a requirement for a state in the root
                //       so this unwrap can't become an expect until that's in place
                let state = self.ctx.states.get(state_id).unwrap();
                let value = state.reference();
                match value.type_info() {
                    Type::Int => ValueExpr::Int(Kind::Dyn(value)),
                    Type::Float => ValueExpr::Float(Kind::Dyn(value)),
                    Type::String => ValueExpr::Str(Kind::Dyn(value)),
                    Type::Bool => todo!("write tests for these"),
                    Type::Char => todo!(),
                    Type::Map => ValueExpr::DynMap(value),
                    Type::List => todo!(),
                    Type::Composite => ValueExpr::DynMap(value),
                }
            }
            "properties" => panic!(),
            scope => {
                let Some(expr) = self.ctx.globals.get(scope) else { return ValueExpr::Null };
                self.resolve(expr)
            }
        }
    }
}

impl<'a, 'frame, 'bp> Resolver<'bp> for ImmediateResolver<'a, 'frame, 'bp> {
    type Output = ValueExpr<'bp>;

    fn resolve(&self, expr: &'bp Expression) -> ValueExpr<'bp> {
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
                ValueExpr::Index(self.resolve(source).into(), self.resolve(index).into())
            }
            Expression::Call { fun, args } => unimplemented!(),
        }
    }
}
