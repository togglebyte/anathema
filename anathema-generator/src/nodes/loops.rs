use std::sync::Arc;

use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, Value};

use crate::expression::{EvaluationContext, FromContext};
use crate::{Expression, NodeKind, Nodes, NodeId};

enum State {
    LoadValue,
    ProduceNode,
}

pub struct LoopState<Output: FromContext> {
    scope: ScopeId,
    collection: ValueRef<Value<Output::Value>>,
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
        collection: ValueRef<Value<Output::Value>>,
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

    fn load_value(&mut self, bucket: &BucketRef<'_, Output::Value>, parent: &NodeId) -> Option<Result<(), Output::Err>> {
        let value_read = bucket.read();
        let collection = value_read.getv2::<List<_>>(self.collection)?;

        // No more items to produce
        if self.value_index == collection.len() {
            return None;
        }

        let value = *collection.iter().skip(self.value_index).next()?;
        bucket.scope_value(self.binding, value, self.scope);

        self.value_index += 1;

        for expr in &*self.expressions {
            let node = match expr.to_node(&EvaluationContext::new(bucket, self.scope), parent.child(self.nodes.len())) {
                Ok(node) => self.nodes.push(node),
                Err(e) => return Some(Err(e)),
            };
        }

        Some(Ok(()))
    }

    pub(super) fn next(&mut self, bucket: &BucketRef<'_, Output::Value>, parent: &NodeId) -> Option<Result<&mut Output, Output::Err>> {
        if self.node_index == self.nodes.len() {
            self.load_value(bucket, parent)?;
        }

        let nodes = self.nodes.inner[self.node_index..].iter_mut();

        for node in nodes {
            match &mut node.kind {
                NodeKind::Single(value, _) => {
                    self.node_index += 1;
                    return Some(Ok(value));
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
