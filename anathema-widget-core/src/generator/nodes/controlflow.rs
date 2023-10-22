use std::rc::Rc;

use anathema_values::{Change, Context, ScopeValue, State, ValueExpr};

use crate::generator::expressions::Expression;
use crate::generator::{Else, If};
use crate::{Nodes, WidgetContainer};

#[derive(Debug)]
pub struct IfElse<'e> {
    pub(super) body: Nodes<'e>,
    if_expr: &'e If,
    elses: &'e [Else],
}

impl<'e> IfElse<'e> {
    pub(crate) fn new(body: Nodes<'e>, if_expr: &'e If, elses: &'e [Else]) -> Self {
        Self {
            body,
            if_expr,
            elses,
        }
    }

    pub(super) fn load_body<'val>(&mut self, context: &Context<'_, 'val>)
    where
        'e: 'val,
    {
        // First evaluate the collection, then the value inside the collection.
        // This could be more efficient by hanging on to the collection
        // via a ref (Rc?).
        let val = match self.if_expr.cond.eval_bool(context, None) {
            true => {},
            false => {}
        };
    }

    pub(super) fn reset_cache(&mut self) {
        self.body.reset_cache();
    }

    pub(super) fn count(&self) -> usize {
        self.body.count()
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes<'e>)> + '_ {
        self.body.iter_mut()
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        self.body.update(node_id, change, state)
    }
}
