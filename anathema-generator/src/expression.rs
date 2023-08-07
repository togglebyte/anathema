use std::sync::Arc;

use anathema_values::{AsSlice, BucketRef, List, Listen, PathId, ScopeId, Truthy, Value, ValueRef};

use crate::nodes::controlflow::{ControlFlow, ControlFlows};
use crate::nodes::loops::LoopState;
use crate::{DataCtx, Node, NodeId, NodeKind, Nodes};

pub struct EvaluationContext<'a, Val> {
    bucket: &'a BucketRef<'a, Val>,
    scope: Option<ScopeId>,
}

impl<'a, Val> EvaluationContext<'a, Val> {
    pub fn new(bucket: &'a BucketRef<'a, Val>, scope: impl Into<Option<ScopeId>>) -> Self {
        Self {
            scope: scope.into(),
            bucket,
        }
    }
}

pub enum ControlFlowExpr {
    If(PathId),
    Else(Option<PathId>),
}

pub enum Expression<Output: FromContext> {
    Node {
        context: Output::Ctx,
        children: Arc<[Expression<Output>]>,
    },
    Loop {
        collection: PathId,
        binding: PathId,
        body: Arc<[Expression<Output>]>,
    },
    ControlFlow(Vec<(ControlFlowExpr, Arc<[Expression<Output>]>)>),
}

impl<Output: FromContext> Expression<Output> {
    pub fn to_node(
        &self,
        eval: &EvaluationContext<'_, Output::Value>,
        node_id: NodeId,
    ) -> Result<Node<Output>, Output::Err> {
        match self {
            Self::Node { context, children } => {
                let context = DataCtx::new(eval.bucket, &node_id, eval.scope, context);
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
                // TODO: unwrap!!!
                let collection = eval.bucket.by_path(*collection, eval.scope).unwrap();
                Output::Notifier::subscribe(collection, node_id.clone());

                let scope = eval.bucket.new_scope(eval.scope);
                let state = LoopState::new(scope, *binding, collection, body.clone());
                Ok(NodeKind::Collection(state).to_node(node_id))
            }
            Self::ControlFlow(flows) => {
                let mut node_flows = vec![];
                for (cond, expressions) in flows {
                    let cond = match cond {
                        ControlFlowExpr::If(path) => {
                            let val = eval.bucket.by_path(*path, eval.scope).unwrap();
                            Output::Notifier::subscribe(val, node_id.clone());
                            ControlFlow::If(val)
                        }
                        ControlFlowExpr::Else(Some(path)) => {
                            let val = eval.bucket.by_path(*path, eval.scope).unwrap();
                            Output::Notifier::subscribe(val, node_id.clone());
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
