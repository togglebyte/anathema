use std::sync::Arc;

use anathema_values::{BucketRef, ScopeId, Truthy, Value, ValueRef};

use crate::expression::{ControlFlowExpr, EvaluationContext, FromContext};
use crate::{Expression, NodeId, NodeKind, Nodes};

enum Cond<Val> {
    If(ValueRef<Value<Val>>),
    Else(Option<ValueRef<Value<Val>>>),
}

impl<Val: Truthy> Cond<Val> {
    pub(crate) fn eval(&self, bucket: &BucketRef<'_, Val>, scope: Option<ScopeId>) -> bool {
        match self {
            Self::If(val) | Self::Else(Some(val)) => {
                bucket.get(*val).map(|val| val.is_true()).unwrap_or(false)
            }
            Self::Else(None) => true,
        }
    }
}

pub(crate) enum ControlFlow<Val> {
    If(ValueRef<Value<Val>>),
    Else(Option<ValueRef<Value<Val>>>),
}

impl<Val: Truthy> ControlFlow<Val> {
    fn eval(&self, bucket: &BucketRef<'_, Val>) -> bool {
        match self {
            Self::If(val_ref) | Self::Else(Some(val_ref)) => bucket
                .get(*val_ref)
                .map(|val| val.is_true())
                .unwrap_or(false),
            Self::Else(None) => true,
        }
    }
}

pub struct ControlFlows<Output: FromContext> {
    flows: Vec<(ControlFlow<Output::Value>, Arc<[Expression<Output>]>)>,
    scope: Option<ScopeId>,
    pub(crate) nodes: Nodes<Output>,
    selected_flow: Option<usize>,
    node_index: usize,
}

impl<Output: FromContext> ControlFlows<Output> {
    pub(crate) fn new(
        flows: Vec<(ControlFlow<Output::Value>, Arc<[Expression<Output>]>)>,
        scope: Option<ScopeId>,
    ) -> Self {
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
                let flow_index = self.eval(bucket)?;
                self.selected_flow = Some(flow_index);

                for expression in &*self.flows[flow_index].1 {
                    match expression.to_node(
                        &EvaluationContext::new(bucket, self.scope),
                        parent.child(self.nodes.len()),
                    ) {
                        Ok(node) => self.nodes.push(node),
                        Err(e) => return Some(Err(e)),
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
                        },
                        NodeKind::ControlFlow(flows) => match flows.next(bucket, &node.id) {
                            last @ Some(_) => return last,
                            None => self.node_index += 1,
                        },
                    }
                }
            }
        }
        None
    }

    // Evaluate the condition that is true for this control flow.
    // The index can then be used to select the truthy branch
    fn eval(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<usize> {
        for (index, (flow, _)) in self.flows.iter().enumerate() {
            if flow.eval(bucket) {
                return Some(index);
            }
        }
        None
    }
}
