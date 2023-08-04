use std::sync::Arc;

use anathema_values::{AsSlice, BucketRef, List, PathId, ScopeId, Truthy, ValueRef, Value};

use crate::nodes::controlflow::ControlFlows;
use crate::nodes::loops::LoopState;
use crate::{NodeKind, Node, Nodes, NodeId};

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

pub enum Cond {
    If(PathId),
    Else(Option<PathId>),
}

impl Cond {
    pub(crate) fn eval<Val: Truthy>(
        &self,
        bucket: &BucketRef<'_, Val>,
        scope: Option<ScopeId>,
    ) -> bool {
        match self {
            Self::If(path) | Self::Else(Some(path)) => bucket
                .by_path(*path, scope)
                .and_then(|val| bucket.get(val))
                .map(|val| val.is_true())
                .unwrap_or(false),
            Self::Else(None) => true,
        }
    }
}

pub struct ControlFlow<Output: FromContext> {
    pub cond: Cond,
    pub body: Arc<[Expression<Output>]>,
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
    ControlFlow(Arc<[ControlFlow<Output>]>),
}

impl<Output: FromContext> Expression<Output> {
    pub fn to_node(
        &self,
        eval: &EvaluationContext<'_, Output::Value>,
        node_id: NodeId,
    ) -> Result<Node<Output>, Output::Err> {
        match self {
            Self::Node { context, children } => {
                let output = Output::from_context(&context, eval.bucket)?;
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
                let collection = eval
                    .bucket
                    .by_path(*collection, eval.scope).unwrap();

                let scope = eval.bucket.new_scope(eval.scope);
                // TODO: Lookup the value, subscribe to the value, if the value does not exist
                // insert Value::Empty,
                let state = LoopState::new(scope, *binding, collection, body.clone());
                Ok(NodeKind::Collection(state).to_node(node_id))
            }
            Self::ControlFlow(flows) => {
                let flows = ControlFlows::new(flows.clone(), eval.scope);
                Ok(NodeKind::ControlFlow(flows).to_node(node_id))
            }
        }
    }
}

pub trait FromContext: Sized {
    type Ctx;
    type Value: Truthy + Clone;
    type Err;

    fn from_context(
        ctx: &Self::Ctx,
        bucket: &BucketRef<'_, Self::Value>,
    ) -> Result<Self, Self::Err>;
}
