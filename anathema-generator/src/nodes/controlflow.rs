use std::sync::Arc;

use anathema_values::{BucketRef, ScopeId};

use crate::expression::{ControlFlow, FromContext, EvaluationContext};
use crate::{NodeKind, Nodes, NodeId};

pub struct ControlFlows<Output: FromContext> {
    flows: Arc<[ControlFlow<Output>]>,
    scope: Option<ScopeId>,
    pub(crate) nodes: Nodes<Output>,
    selected_flow: Option<usize>,
    node_index: usize,
}

impl<Output: FromContext> ControlFlows<Output> {
    pub fn new(flows: Arc<[ControlFlow<Output>]>, scope: Option<ScopeId>) -> Self {
        Self {
            flows,
            scope,
            nodes: Nodes::empty(),
            selected_flow: None,
            node_index: 0,
        }
    }

    pub(super) fn next(
        &mut self,
        bucket: &BucketRef<'_, Output::Value>,
        parent: &NodeId,
    ) -> Option<Result<&mut Output, Output::Err>> {
        match self.selected_flow {
            None => {
                let flow_index = self.eval(bucket, self.scope)?;
                self.selected_flow = Some(flow_index);
                for expr in &*self.flows[flow_index].body {
                    match expr.to_node(&EvaluationContext::new(bucket, self.scope), parent.child(self.nodes.len())) {
                        Ok(node) => self.nodes.push(node),
                        Err(e) => return Some(Err(e))
                    }
                }
                return self.next(bucket, parent);
            }
            Some(index) => {
                for node in self.nodes.inner[self.node_index..].iter_mut() {
                    match &mut node.kind {
                        NodeKind::Single(output, _) => {
                            self.node_index += 1;
                            return Some(Ok(output));
                        }
                        NodeKind::Collection(nodes) => match nodes.next(bucket, &node.id) {
                            last @ Some(_) => return last,
                            None => self.node_index += 1,
                        }
                        NodeKind::ControlFlow(flows) => match flows.next(bucket, &node.id) {
                            last @ Some(_) => return last,
                            None => self.node_index += 1,
                        }
                    }
                }
            }
        }
        None
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
