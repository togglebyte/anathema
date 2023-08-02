use std::sync::Arc;

use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, ValueV2};

use crate::expression::{EvaluationContext, FromContext};
use crate::{Expression, Node, Nodes};

enum State {
    LoadValue,
    ProduceNode,
}

pub struct LoopState<Output: FromContext> {
    scope: ScopeId,
    collection: ValueRef<ValueV2<Output::Value>>,
    expressions: Arc<[Expression<Output::Ctx, Output::Value>]>,
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
        collection: ValueRef<ValueV2<Output::Value>>,
        expressions: Arc<[Expression<Output::Ctx, Output::Value>]>,
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

    fn load_value(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<()> {
        let collection = bucket.getv2::<List<_>>(self.collection)?;

        // No more items to produce
        if self.value_index == collection.len() {
            return None;
        }

        self.value_index += 1;

        for expr in &*self.expressions {
            let node = expr.to_node(&EvaluationContext::new(bucket, self.scope))?;
            self.nodes.push(node);
        }

        Some(())
    }

    pub(super) fn next(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<&mut Output> {
        if self.node_index == self.nodes.len() {
            self.load_value(bucket)?;
        }

        let nodes = self.nodes.inner[self.node_index..].iter_mut();

        for node in nodes {
            match node {
                Node::Single(value, _) => {
                    self.node_index += 1;
                    return Some(value);
                }
                Node::Collection(nodes) => match nodes.next(bucket) {
                    last @ Some(_) => return last,
                    None => self.node_index += 1,
                }
                Node::ControlFlow(flows) => match flows.next(bucket) {
                    last @ Some(_) => return last,
                    None => self.node_index += 1,
                }
            }
        }

        None
    }
}
