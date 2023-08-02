use std::sync::Arc;

use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, ValueV2};

use crate::nodes::controlflow::ControlFlows;
use crate::nodes::loops::LoopState;
use crate::{Node, Nodes};

pub enum Value<T> {
    /// Value that can change at runtime
    Dynamic(PathId),
    /// A static value that never changes
    Static(T)
}

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

pub enum Cond<Val> {
    If(Value<Val>),
    Else(Option<Value<Val>>),
}

impl<Val> Cond<Val> {
    pub(crate) fn eval(&self, bucket: &BucketRef<'_, Val>, scope: Option<ScopeId>) -> bool
    where
        Val: Truthy,
    {
        match self {
            Self::If(Value::Static(val)) | Self::Else(Some(Value::Static(val))) => val.is_true(),
            Self::If(Value::Dynamic(path)) | Self::Else(Some(Value::Dynamic(path))) => bucket
                .by_path(*path, scope)
                .and_then(|val| bucket.get(val))
                .map(|val| val.is_true())
                .unwrap_or(false),
            Self::Else(None) => true,
        }
    }
}

pub struct ControlFlow<Ctx, Val> {
    pub cond: Cond<Val>,
    pub body: Arc<[Expression<Ctx, Val>]>,
}

pub enum Expression<Ctx, Val> {
    Node {
        context: Ctx,
        children: Arc<[Expression<Ctx, Val>]>,
    },
    Loop {
        collection: Value<Val>,
        binding: PathId,
        body: Arc<[Expression<Ctx, Val>]>,
    },
    ControlFlow(Arc<[ControlFlow<Ctx, Val>]>),
}

impl<Ctx, Val> Expression<Ctx, Val> {
    pub fn to_node<Output>(&self, eval: &EvaluationContext<'_, Val>) -> Option<Node<Output>>
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
                let collection = match collection {
                    Value::Dynamic(path) => eval.bucket.by_path(*path, eval.scope)?,
                    Value::Static(val) => panic!("impl to slice or whatever")
                };

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
