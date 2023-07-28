use std::sync::Arc;

use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, ValueV2};

use crate::nodes::controlflow::ControlFlows;
use crate::nodes::loops::LoopState;
use crate::{Node, Nodes};

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

#[derive(Debug)]
pub enum Cond {
    If(PathId),
    Else(Option<PathId>),
}

impl Cond {
    pub(crate) fn eval<Val>(&self, bucket: &BucketRef<'_, Val>, scope: Option<ScopeId>) -> bool
    where
        Val: Truthy,
    {
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

#[derive(Debug)]
pub struct ControlFlow<Ctx> {
    pub(crate) cond: Cond,
    pub(crate) body: Arc<[Expression<Ctx>]>,
}

#[derive(Debug)]
pub enum Expression<Ctx> {
    Node {
        context: Ctx,
        children: Arc<[Expression<Ctx>]>,
    },
    Loop {
        collection: PathId,
        binding: PathId,
        body: Arc<[Expression<Ctx>]>,
    },
    ControlFlow(Arc<[ControlFlow<Ctx>]>),
}

impl<Ctx> Expression<Ctx> {
    pub fn to_node<Output, Val>(&self, eval: &EvaluationContext<'_, Val>) -> Option<Node<Output>>
    where
        Val: Truthy,
        Output: FromContext<Ctx = Ctx, Value = Val>,
    {
        match self {
            Self::Node { context, children } => {
                let output = Output::from_context(&context, &eval.bucket)?;
                let nodes = children
                    .iter()
                    .filter_map(|expr| expr.to_node(eval))
                    .collect();
                Some(Node::Single(output, Nodes::new(nodes)))
            }
            Self::Loop {
                collection,
                binding,
                body,
            } => {
                let collection = eval.bucket.by_path(*collection, eval.scope)?;
                let scope = eval.bucket.new_scope(eval.scope);
                let state = LoopState::new(scope, *binding, collection, body.clone());
                Some(Node::Collection(state))
            }
            Self::ControlFlow(flows) => {
                let flows = ControlFlows::new(flows.clone(), eval.scope);
                Some(Node::ControlFlow(flows))
            }
        }
    }
}

pub trait FromContext: Sized {
    type Ctx;
    type Value: Truthy;

    fn from_context(ctx: &Self::Ctx, bucket: &BucketRef<'_, Self::Value>) -> Option<Self>;
}
