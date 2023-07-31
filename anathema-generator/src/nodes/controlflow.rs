use std::sync::Arc;

use anathema_values::{BucketRef, ScopeId};

use crate::expression::{ControlFlow, FromContext, EvaluationContext};
use crate::{Node, Nodes};

pub struct ControlFlows<Output: FromContext> {
    flows: Arc<[ControlFlow<Output::Ctx>]>,
    scope: Option<ScopeId>,
    nodes: Nodes<Output>,
    selected_flow: Option<usize>,
    node_index: usize,
}

impl<Output: FromContext> ControlFlows<Output> {
    pub fn new(flows: Arc<[ControlFlow<Output::Ctx>]>, scope: Option<ScopeId>) -> Self {
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
    ) -> Option<&mut Output> {
        match self.selected_flow {
            None => {
                let flow_index = self.eval(bucket, self.scope)?;
                self.selected_flow = Some(flow_index);
                for expr in &*self.flows[flow_index].body {
                    let node = expr.to_node(&EvaluationContext::new(bucket, self.scope))?;
                    self.nodes.push(node);
                }
                return self.next(bucket);
            }
            Some(index) => {
                for node in self.nodes.inner[self.node_index..].iter_mut() {
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
            }
        }
        None
    }

    // pub(super) fn last(&mut self) -> Option<&mut Output> {
    //     self.nodes.inner.last_mut().and_then(Node::get_output)
    // }

    fn eval(&mut self, bucket: &BucketRef<'_, Output::Value>, scope: Option<ScopeId>) -> Option<usize> {
        for (index, flow) in self.flows.iter().enumerate() {
            if flow.cond.eval(bucket, scope) {
                return Some(index);
            }
        }
        None
    }
}
