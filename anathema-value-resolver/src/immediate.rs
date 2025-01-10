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
                let state_id = self.ctx.scopes.get_state();
                let state = self.ctx.states.get(state_id).unwrap();
                let value = state.reference();
                match value.type_info() {
                    Type::Int => ValueExpr::Int(Kind::Dyn(value)),
                    Type::Float => todo!(),
                    Type::String => todo!(),
                    Type::Bool => todo!(),
                    Type::Char => todo!(),
                    Type::Map => ValueExpr::DynMap(value),
                    Type::List => todo!(),
                    Type::Composite => todo!(),
                }
            }
            "properties" => panic!(),
            global => panic!("{global}"),
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
            Expression::Either(first, second) => match self.resolve(first) {
                ValueExpr::Null => self.resolve(second),
                value => value,
            },
            Expression::Ident(ident) => self.lookup(ident),
            Expression::Index(source, index) => ValueExpr::Index(self.resolve(source).into(), self.resolve(index).into()),
            Expression::Call { fun, args } => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{AnyValue, Map, State, States};
    use anathema_templates::expressions::{ident, index, strlit};
    use anathema_templates::Globals;

    use super::*;
    use crate::scope::Scope;

    struct TestCase {
        globals: Globals,
        scopes: Scope,
        states: States,
    }

    impl TestCase {
        fn eval<'bp>(&'bp self, expr: &'bp Expression) -> ValueExpr<'bp> {
            let ctx = ResolverCtx::new(&self.globals, &self.scopes, &self.states);
            let mut resolver = ImmediateResolver::new(&ctx, Subscriber::ZERO);
            resolver.resolve(expr)
        }
    }

    fn setup<T: AnyValue>(key: &str, value: T) -> TestCase {
        let globals = Globals::empty();
        let mut scopes = Scope::new();
        let mut states = States::new();

        let mut state = Map::empty();
        state.insert(key, value);
        let state = anathema_state::Value::<Box<dyn AnyValue>>::new(Box::new(state));
        let state_id = states.insert(state);
        scopes.insert_state(state_id);

        TestCase {
            globals,
            scopes,
            states,
        }
    }

    #[test]
    fn eval_primitive() {
        let expr = index(ident("state"), strlit("value"));
        let test = setup("value", 123);
        test.eval(&*expr);
    }

    #[test]
    fn variable_keys() {
        // text state.map[state.key]
        // assert_eq!(expected, actual);
    }
}
