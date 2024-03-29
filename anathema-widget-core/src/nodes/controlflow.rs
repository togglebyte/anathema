use anathema_values::{Change, Context, DynValue, NextNodeId, NodeId, Value};

use crate::expressions::{ElseExpr, IfExpr};
use crate::{Nodes, WidgetContainer};

#[derive(Debug)]
pub struct IfElse<'e> {
    pub(super) if_node: If<'e>,
    pub(super) elses: Vec<Else<'e>>,
}

impl<'e> IfElse<'e> {
    pub(crate) fn new(
        if_expr: &'e IfExpr,
        elses: &'e [ElseExpr],
        context: &Context<'_, '_>,
        node_id: NodeId,
        next_node: NextNodeId,
    ) -> Self {
        let mut if_node = If {
            cond: bool::init_value(context, &node_id, &if_expr.cond),
            previous: false,
            body: Nodes::new(&if_expr.expressions, node_id.child(0)),
            node_id,
            next_node,
        };

        let mut elses = elses
            .iter()
            .map(|e| {
                let node_id = if_node.next_node.next(&if_node.node_id);
                Else {
                    cond: e
                        .cond
                        .as_ref()
                        .map(|expr| bool::init_value(context, &node_id, expr)),
                    previous: false,
                    body: Nodes::new(&e.expressions, node_id.child(0)),
                    node_id,
                }
            })
            .collect::<Vec<_>>();

        if_node.resolve(context);
        if_node.previous = if_node.cond.value_or_default();

        if !if_node.is_true() {
            for el in &mut elses {
                el.resolve(context);
                if el.is_true() {
                    break;
                }
            }
        }

        Self { if_node, elses }
    }

    pub(super) fn body_mut(&mut self) -> Option<&mut Nodes<'e>> {
        if self.if_node.is_true() {
            return Some(&mut self.if_node.body);
        }

        for el in &mut self.elses {
            if el.is_true() {
                return Some(&mut el.body);
            }
        }

        None
    }

    fn body(&self) -> Option<&Nodes<'e>> {
        if self.if_node.is_true() {
            return Some(&self.if_node.body);
        }

        for el in &self.elses {
            if el.is_true() {
                return Some(&el.body);
            }
        }

        None
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer<'e>, &mut Nodes<'e>)> + '_ {
        self.body_mut()
            .into_iter()
            .flat_map(|nodes| nodes.iter_mut())
    }

    pub(super) fn reset_cache(&mut self) {
        self.if_node.body.reset_cache();
        self.elses.iter_mut().for_each(|e| e.body.reset_cache());
    }

    pub(super) fn count(&self) -> usize {
        self.body().map(|nodes| nodes.count()).unwrap_or(0)
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: &Change, context: &Context<'_, '_>) {
        // If
        if self.if_node.node_id.contains(node_id) {
            if self.if_node.node_id.eq(node_id) {
                self.if_node.resolve(context);
                let current = self.if_node.cond.value_or_default();
                self.if_node.previous = current;
            } else {
                self.if_node.body.update(node_id, change, context);
            }
        }

        // Elses
        for e in &mut self.elses {
            if e.node_id.contains(node_id) {
                if e.node_id.eq(node_id) {
                    e.resolve(context);
                    let current = self.if_node.cond.value_or_default();
                    e.previous = current;
                } else {
                    e.body.update(node_id, change, context);
                }

                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct If<'e> {
    cond: Value<bool>,
    // Previous condition value
    previous: bool,
    pub(super) body: Nodes<'e>,
    node_id: NodeId,
    next_node: NextNodeId,
}

impl If<'_> {
    pub(super) fn is_true(&self) -> bool {
        self.cond.is_true()
    }

    fn resolve(&mut self, context: &Context<'_, '_>) {
        self.cond.resolve(context, &self.node_id);
    }
}

#[derive(Debug)]
pub struct Else<'e> {
    cond: Option<Value<bool>>,
    // Previous condition value
    previous: bool,
    pub(super) body: Nodes<'e>,
    node_id: NodeId,
}

impl Else<'_> {
    pub(super) fn is_true(&self) -> bool {
        match &self.cond {
            None => true,
            Some(cond) => cond.is_true(),
        }
    }

    fn resolve(&mut self, context: &Context<'_, '_>) {
        if let Some(c) = self.cond.as_mut() {
            c.resolve(context, &self.node_id)
        }
    }
}
