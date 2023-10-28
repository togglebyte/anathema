use std::rc::Rc;

use anathema_values::{Change, Context, NodeId, State, ValueExpr};

use crate::generator::expressions::Expression;
use crate::generator::{Else, If};
use crate::{Nodes, WidgetContainer};

#[derive(Debug)]
pub struct IfElse<'e> {
    pub(super) body: Option<Nodes<'e>>,
    if_expr: &'e If,
    elses: &'e [Else],
}

impl<'e> IfElse<'e> {
    pub(crate) fn new(if_expr: &'e If, elses: &'e [Else]) -> Self {
        Self {
            body: None,
            if_expr,
            elses,
        }
    }

    pub(super) fn load_body<'val>(
        &mut self,
        context: &Context<'_, 'val>,
        node_id: NodeId,
    )
    where
        'e: 'val,
    {
        panic!("deferred values")
        // match self.if_expr.cond.eval_bool(context) {
        //     true => {
        //         let body = Nodes::new(&self.if_expr.body, node_id);
        //         self.body = Some(body);
        //     }
        //     false => {
        //         for els in self.elses {
        //             match &els.cond {
        //                 Some(cond) if cond.eval_bool(context, None) => {}
        //                 None => {}
        //                 _ => continue,
        //             }

        //             let body = Nodes::new(&els.body, node_id);
        //             self.body = Some(body);
        //             break;
        //         }
        //     }
        // }
    }

    pub(super) fn reset_cache(&mut self) {
        self.body.as_mut().map(|b| b.reset_cache());
    }

    pub(super) fn count(&self) -> usize {
        self.body.as_ref().map(|b| b.count()).unwrap_or(0)
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes<'e>)> + '_ {
        self.body.iter_mut().map(|mut n| n.iter_mut()).flatten()
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        self.body.as_mut().map(|b| b.update(node_id, change, state));
    }
}
