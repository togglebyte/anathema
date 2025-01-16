use anathema_state::{PendingValue, Subscriber};
use anathema_strings::HStrings;
use anathema_templates::Expression;

use crate::context::ResolverCtx;
use crate::expression::ValueExpr;
use crate::value::{Value, ValueKind};
use crate::Resolver;

macro_rules! or_null {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => return CollectionExpr::Null,
        }
    };
}

#[derive(Debug)]
pub enum CollectionExpr<'bp> {
    List(&'bp [Expression]),
    DynList(PendingValue),
    Null,
}

impl<'bp> CollectionExpr<'bp> {
    fn len(&self) -> usize {
        match self {
            CollectionExpr::List(list) => list.len(),
            CollectionExpr::DynList(value) => {
                let Some(state) = value.as_state() else { return 0 };
                let Some(list) = state.as_any_list() else { return 0 };
                list.len()
            }
            CollectionExpr::Null => 0,
        }
    }
}

#[derive(Debug)]
pub struct Collection<'bp> {
    expr: CollectionExpr<'bp>,
    pub(crate) sub: Subscriber,
}

impl<'bp> Collection<'bp> {
    pub fn new(expr: CollectionExpr<'bp>, sub: Subscriber) -> Self {
        Self { expr, sub }
    }

    pub fn len(&self) -> usize {
        self.expr.len()
    }
}

pub(crate) struct CollectionResolver<'a, 'frame, 'bp> {
    ctx: &'a ResolverCtx<'frame, 'bp>,
}

impl<'a, 'frame, 'bp> CollectionResolver<'a, 'frame, 'bp> {
    pub fn new(ctx: &'a ResolverCtx<'frame, 'bp>) -> Self {
        Self { ctx }
    }

    fn lookup(&self, ident: &str) -> CollectionExpr<'bp> {
        match ident {
            "state" => {
                // TODO: filthy unwraps all over this function
                let state_id = self.ctx.scopes.get_state().unwrap();

                let state = self.ctx.states.get(state_id).unwrap();
                let value = state.reference();
                CollectionExpr::DynList(value)
            }
            "properties" => panic!(),
            scope => {
                // let Some(expr) = self.ctx.globals.get(scope) else { return ValueExpr::Null };
                // self.resolve(expr)
                panic!()
            }
        }
    }
}

impl<'a, 'frame, 'bp> Resolver<'bp> for CollectionResolver<'a, 'frame, 'bp> {
    type Output = CollectionExpr<'bp>;

    fn resolve(&self, expr: &'bp Expression) -> Self::Output {
        match expr {
            Expression::List(list) => CollectionExpr::List(list),
            Expression::Ident(ident) => self.lookup(ident),
            Expression::Either(first, second) => match self.resolve(first) {
                CollectionExpr::Null => self.resolve(second),
                collection => collection,
            },
            Expression::Index(src, index) => {
                let index = 1;
                match self.resolve(src) {
                    CollectionExpr::List(list) => self.resolve(&list[index]),
                    CollectionExpr::DynList(list) => {
                        let state = or_null!(list.as_state());
                        let list = or_null!(state.as_any_list());
                        let value = or_null!(list.lookup(index));
                        CollectionExpr::DynList(value)
                    }
                    CollectionExpr::Null => CollectionExpr::Null,
                }
            }
            Expression::Primitive(_)
            | Expression::Str(_)
            | Expression::Map(_)
            | Expression::TextSegments(_)
            | Expression::Not(_)
            | Expression::Negative(_)
            | Expression::Equality(..)
            | Expression::LogicalOp(..)
            | Expression::Op(..)
            | Expression::Call { .. } => CollectionExpr::Null,
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_state::List;
    use anathema_templates::expressions::{ident, index, list, num, strlit, sub};

    use super::*;
    use crate::testing::setup;

    #[test]
    fn nested_collection() {
        let expr = index(list([list([1, 2, 3])]), sub(num(1), num(1)));

        setup().finish(|mut test| {
            let value = test.eval_collection(&*expr);
            panic!("{value:?}")
        });
    }

    #[test]
    fn dyn_collection() {
        let expr = index(ident("state"), strlit("list"));
        setup().finish(|mut test| {
            let list: List<_> = List::from_iter([1, 2, 3]);
            test.set_state("list", list);
            let value = test.eval_collection(&*expr);
            assert_eq!(value.len(), 3);
        });
    }

    #[test]
    fn static_collection() {
        let expr = list([1, 2, 3]);

        setup().finish(|mut test| {
            let value = test.eval_collection(&*expr);
            assert_eq!(value.len(), 3);
        });
    }
}
