use std::marker::PhantomData;

use anathema_state::{Number, Path, Subscriber, Type};
use anathema_strings::HStrings;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::{Expression, Primitive};

use crate::context::ResolverCtx;
use crate::null::Null;
use crate::scope::Lookup;
use crate::value::{Kind, Str, Value};
use crate::Resolver;

pub struct ImmediateResolver<'a, 'frame, 'bp> {
    ctx: &'a ResolverCtx<'frame, 'bp>,
    value_id: Subscriber,
}

impl<'a, 'frame, 'bp> ImmediateResolver<'a, 'frame, 'bp> {
    pub(super) fn new(ctx: &'a ResolverCtx<'frame, 'bp>, value_id: Subscriber) -> Self {
        Self { ctx, value_id }
    }

    fn lookup(&self, ident: &str) -> Value<'bp> {
        match ident {
            "state" => {
                let state_id = self.ctx.scopes.get_state();
                let state = self.ctx.states.get(state_id).unwrap();
                let value = state.reference();
                match value.type_info() {
                    Type::Int => Value::Number(Kind::Dyn {
                        source: value,
                        cache: panic!(),
                    }),
                    Type::Float => todo!(),
                    Type::Map => todo!(),
                    Type::List => todo!(),
                    Type::String => todo!(),
                    Type::Composite => todo!(),
                }
            }
            "properties" => panic!(),
            global => panic!(),
        }
    }

    fn resolve_index(&self, source: &'bp Expression, index: &'bp Expression) -> Value<'bp> {
        let source = self.resolve(source);
        let path = match index {
            Expression::Str(s) => Path::from(&**s),
            _ => panic!(),
            // or_null!(NumberResolver
            //     .resolve(index)
            //     .map(|n| Path::from(n.as_int() as usize))),
        };

        panic!()
    }
}

impl<'a, 'frame, 'bp> Resolver<'bp> for ImmediateResolver<'a, 'frame, 'bp> {
    type Output = Value<'bp>;

    fn resolve(&self, expr: &'bp Expression) -> Value<'bp> {
        match expr {
            Expression::Primitive(primitive) => Value::from(*primitive),
            Expression::Str(s) => Value::Str(s),
            Expression::List(vec) => Value::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Map(map) => Value::Map(map.iter().map(|(k, e)| (k.as_str(), self.resolve(e))).collect()),
            Expression::TextSegments(vec) => Value::List(vec.iter().map(|e| self.resolve(e)).collect()),
            Expression::Not(expr) => Value::Not(self.resolve(expr).into()),
            Expression::Negative(expr) => Value::Negative(self.resolve(expr).into()),
            Expression::Equality(lhs, rhs, equality) => {
                let lhs = self.resolve(lhs);
                let rhs = self.resolve(rhs);
                Value::Equality(lhs.into(), rhs.into(), *equality)
            }
            Expression::LogicalOp(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                Value::LogicalOp(lhs, rhs, *op)
            }
            Expression::Op(lhs, rhs, op) => {
                let lhs = self.resolve(lhs).into();
                let rhs = self.resolve(rhs).into();
                Value::Op(lhs, rhs, *op)
            }
            Expression::Either(first, second) => match self.resolve(first) {
                Value::Null => self.resolve(second),
                value => value,
            },
            Expression::Ident(ident) => self.lookup(ident),
            Expression::Index(source, index) => Value::Index(self.resolve(source).into(), self.resolve(index).into()),
            Expression::Call { fun, args } => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{AnyState, Map, State, States};
    use anathema_templates::expressions::{ident, index};
    use anathema_templates::Globals;

    use super::*;
    use crate::scope::Scope;

    struct TestCase {
        globals: Globals,
        scopes: Scope,
        states: States,
    }

    impl TestCase {
        fn eval<'bp>(&'bp self, expr: &'bp Expression) -> Value<'bp> {
            let ctx = ResolverCtx::new(&self.globals, &self.scopes, &self.states);
            let mut resolver = ImmediateResolver::new(&ctx, Subscriber::ZERO);
            resolver.resolve(expr)
        }
    }

    fn setup<T: State>(key: &str, value: T) -> TestCase {
        let globals = Globals::empty();
        let mut scopes = Scope::new();
        let mut states = States::new();

        let mut state = Map::empty();
        state.insert(key, value);
        let state = anathema_state::Value::<Box<dyn AnyState>>::new(Box::new(state));
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
        let expr = index(ident("state"), ident("value"));
        let test = setup("value", 123);
        test.eval(&*expr);
    }

    #[test]
    fn variable_keys() {
        // text state.map[state.key]
        // assert_eq!(expected, actual);
    }
}
