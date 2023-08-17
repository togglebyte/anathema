use std::sync::Arc;

use anathema_values::{Container, List, PathId, ScopeId, ScopeValue, StoreRef, Truthy, ValueRef};

use crate::{expression::{EvaluationContext, FromContext}, Expression, NodeId};

use super::{Nodes, NodeKind};

enum State {
    LoadValue,
    ProduceNode,
}

pub struct LoopState<Output: FromContext> {
    scope: ScopeId,
    collection: ScopeValue<Output::Value>,
    expressions: Arc<[Expression<Output>]>,
    binding: PathId,
    expression_index: usize,
    value_index: usize,
    node_index: usize,
    pub(super) nodes: Nodes<Output>,
}

impl<Output: FromContext> LoopState<Output> {
    pub(crate) fn new(
        scope: ScopeId,
        binding: PathId,
        collection: ScopeValue<Output::Value>,
        expressions: Arc<[Expression<Output>]>,
    ) -> Self {
        Self {
            scope,
            binding,
            collection,
            expressions,
            expression_index: 0,
            value_index: 0,
            node_index: 0,
            nodes: Nodes::empty(),
        }
    }

    fn load_value(
        &mut self,
        store: &StoreRef<'_, Output::Value>,
        parent: &NodeId,
    ) -> Option<Result<(), Output::Err>> {
        let value_read = store.read();

        // value = [1, 2, 3]
        // value = path_id
        // value = 1
        let value = match &self.collection {
            ScopeValue::Dyn(col) => value_read
                .getv2::<List<_>>(*col)?
                .iter()
                .skip(self.value_index)
                .next()
                .map(|val| ScopeValue::Dyn(*val))?,
            ScopeValue::List(list) => list.get(self.value_index).cloned()?,
            ScopeValue::Static(val) => return None,
        };

        self.value_index += 1;
        store.scope_value(self.binding, value, self.scope);

        for expr in &*self.expressions {
            let node = match expr.to_node(
                &EvaluationContext::new(store, self.scope),
                parent.child(self.nodes.len()),
            ) {
                Ok(node) => self.nodes.push(node),
                Err(e) => return Some(Err(e)),
            };
        }

        Some(Ok(()))
    }

    pub(super) fn next(
        &mut self,
        bucket: &StoreRef<'_, Output::Value>,
        parent: &NodeId,
    ) -> Option<Result<(&mut Output, &mut Nodes<Output>), Output::Err>> {
        if self.node_index == self.nodes.len() {
            self.load_value(bucket, parent)?;
        }

        let nodes = self.nodes.inner[self.node_index..].iter_mut();

        for node in nodes {
            match &mut node.kind {
                NodeKind::Single(value, children) => {
                    self.node_index += 1;
                    return Some(Ok((value, children)));
                }
                NodeKind::Collection(nodes) => match nodes.next(bucket, &node.id) {
                    last @ Some(_) => return last,
                    None => self.node_index += 1,
                },
                NodeKind::ControlFlow(flows) => match flows.next(bucket, &node.id) {
                    last @ Some(_) => return last,
                    None => self.node_index += 1,
                },
            }
        }

        None
    }
}
