use std::sync::Arc;

use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, ValueV2};

use crate::expression::{EvaluationContext, FromContext};
use crate::{Expression, Node, Nodes};

pub struct LoopState<Output: FromContext> {
    scope: ScopeId,
    collection: ValueRef<ValueV2<Output::Value>>,
    expressions: Arc<[Expression<Output::Ctx>]>,
    binding: PathId,
    expression_index: usize,
    value_index: usize,
    nodes: Nodes<Output>,
}

impl<Output: FromContext> LoopState<Output> {
    pub(crate) fn new(
        scope: ScopeId,
        binding: PathId,
        collection: ValueRef<ValueV2<Output::Value>>,
        expressions: Arc<[Expression<Output::Ctx>]>,
    ) -> Self {
        Self {
            scope,
            binding,
            collection,
            expressions,
            expression_index: 0,
            value_index: 0,
            nodes: Nodes::empty(),
        }
    }

    pub(super) fn generate_next(
        &mut self,
        bucket: &BucketRef<'_, Output::Value>,
    ) -> Option<&mut Node<Output>> {
        let collection = bucket.getv2::<List<_>>(self.collection)?;

        // No more items to produce
        if self.expression_index == self.expressions.len() && self.value_index == collection.len() {
            return None;
        }

        // First expression, load up new value into the scope
        if self.expression_index == 0 {
            let value = collection[self.value_index];
            bucket.scope_value(self.binding, value, self.scope);
        }

        // Last expression done, reset
        if self.expression_index == self.expressions.len() {
            self.expression_index = 0;
            self.value_index += 1;
        }

        let expression_index = self.expression_index;
        self.expression_index += 1;
        let expr = &self.expressions[expression_index];
        let node = expr.to_node(&EvaluationContext::new(bucket, self.scope))?;
        self.nodes.push(node);
        self.nodes.inner.last_mut()
    }

    pub(super) fn last(&mut self) -> Option<&mut Output> {
        self.nodes.inner.last_mut().and_then(Node::get_node)
    }
}
