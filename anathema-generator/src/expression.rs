use std::sync::Arc;

use anathema_values::{AsSlice, StoreRef, List, Listen, PathId, ScopeId, Truthy, Container, ValueRef, ScopeValue};

use crate::nodes::{Node, NodeKind, Nodes};
use crate::nodes::controlflow::{ControlFlow, ControlFlows};
use crate::nodes::loops::LoopState;
use crate::{DataCtx, NodeId, ExpressionValue, ExpressionValues};

pub struct EvaluationContext<'a, Val> {
    store: &'a StoreRef<'a, Val>,
    scope: Option<ScopeId>,
}

impl<'a, Val> EvaluationContext<'a, Val> {
    pub fn new(bucket: &'a StoreRef<'a, Val>, scope: impl Into<Option<ScopeId>>) -> Self {
        Self {
            scope: scope.into(),
            store: bucket,
        }
    }
}

pub enum ControlFlowExpr<T> {
    If(ExpressionValue<T>),
    Else(Option<ExpressionValue<T>>),
}

pub enum Expression<Output: FromContext> {
    Node {
        context: Output::Ctx,
        children: Arc<[Expression<Output>]>,
        attributes: ExpressionValues<Output::Value>,
    },
    Loop {
        collection: ExpressionValue<Output::Value>,
        binding: PathId,
        body: Arc<[Expression<Output>]>,
    },
    ControlFlow(Vec<(ControlFlowExpr<Output::Value>, Arc<[Expression<Output>]>)>),
}

impl<Output: FromContext> Expression<Output> {
    pub(crate) fn to_node(
        &self,
        eval: &EvaluationContext<'_, Output::Value>,
        node_id: NodeId,
    ) -> Result<Node<Output>, Output::Err> {
        match self {
            Self::Node { context, children, attributes } => {
                let context = DataCtx::new(eval.store, &node_id, eval.scope, context, attributes);
                let output = Output::from_context(context)?;
                let nodes = children
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| expr.to_node(eval, node_id.child(i)))
                    .collect::<Result<_, Output::Err>>()?;
                Ok(NodeKind::Single(output, Nodes::new(nodes)).to_node(node_id))
            }
            Self::Loop {
                collection,
                binding,
                body,
            } => {
                let collection = collection.to_scope_value::<Output::Notifier>(eval.store, eval.scope, &node_id);
                let scope = eval.store.new_scope(eval.scope);
                let state = LoopState::new(scope, *binding, collection, body.clone());
                Ok(NodeKind::Collection(state).to_node(node_id))
            }
            Self::ControlFlow(flows) => {
                let mut node_flows = vec![];
                for (cond, expressions) in flows {
                    let cond = match cond {
                        ControlFlowExpr::If(val) => {
                            let val = val.to_scope_value::<Output::Notifier>(eval.store, eval.scope, &node_id);
                            ControlFlow::If(val)
                        }
                        ControlFlowExpr::Else(Some(val)) => {
                            let val = val.to_scope_value::<Output::Notifier>(eval.store, eval.scope, &node_id);
                            ControlFlow::Else(Some(val))
                        }
                        ControlFlowExpr::Else(None) => ControlFlow::Else(None),
                    };
                    node_flows.push((cond, expressions.clone()));
                }
                let flows = ControlFlows::new(node_flows, eval.scope);
                Ok(NodeKind::ControlFlow(flows).to_node(node_id))
            }
        }
    }
}

pub trait FromContext: Sized {
    type Ctx;
    type Value: Truthy + Clone;
    type Err;
    type Notifier: Listen<Key = NodeId, Value = Self::Value>;

    fn from_context(ctx: DataCtx<'_, Self>) -> Result<Self, Self::Err>;
}
