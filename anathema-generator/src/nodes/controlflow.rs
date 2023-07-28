use std::sync::Arc;

use anathema_values::{BucketRef, ScopeId};

use crate::expression::{ControlFlow, FromContext, EvaluationContext};
use crate::{Node, Nodes};

pub struct ControlFlows<Output: FromContext> {
    flows: Arc<[ControlFlow<Output::Ctx>]>,
    scope: Option<ScopeId>,
    nodes: Nodes<Output>,
    selected_flow: Option<usize>,
    expression_index: usize,
}

impl<Output: FromContext> ControlFlows<Output> {
    pub fn new(flows: Arc<[ControlFlow<Output::Ctx>]>, scope: Option<ScopeId>) -> Self {
        Self {
            flows,
            scope,
            nodes: Nodes::empty(),
            selected_flow: None,
            expression_index: 0,
        }
    }

    pub(super) fn generate_next(
        &mut self,
        bucket: &BucketRef<'_, Output::Value>,
    ) -> Option<&mut Node<Output>> {
        match self.selected_flow {
            None => {
                self.selected_flow = self.eval(bucket, self.scope);
                self.generate_next(bucket)
            }
            Some(index) => {
                let flow = &self.flows[index];

                if self.expression_index == flow.body.len() {
                    return None;
                }

                let expression_index = self.expression_index;
                self.expression_index += 1;
                let expr = &flow.body[expression_index];
                let node = expr.to_node(&EvaluationContext::new(bucket, self.scope))?;
                self.nodes.push(node);
                self.nodes.inner.last_mut()
            }
        }
    }

    pub(super) fn last(&mut self) -> Option<&mut Output> {
        self.nodes.inner.last_mut().and_then(Node::get_node)
    }

    fn eval(&mut self, bucket: &BucketRef<'_, Output::Value>, scope: Option<ScopeId>) -> Option<usize> {
        for (index, flow) in self.flows.iter().enumerate() {
            if flow.cond.eval(bucket, scope) {
                return Some(index);
            }
        }
        None
    }
}
