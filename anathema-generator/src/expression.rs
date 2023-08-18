use std::sync::Arc;

use anathema_values::{
    AsSlice, Container, List, Listen, PathId, ScopeId, ScopeValue, StoreRef, Truthy, ValueRef,
};

use crate::nodes::controlflow::{ControlFlow, ControlFlows};
use crate::nodes::loops::LoopState;
use crate::nodes::{Node, NodeKind, Nodes};
use crate::{DataCtx, ExpressionValue, ExpressionValues, NodeId};

pub struct EvaluationContext<'a, Val> {
    store: &'a StoreRef<'a, Val>,
    scope: Option<ScopeId>,
}

impl<'a, Val> EvaluationContext<'a, Val> {
    pub fn new(store: &'a StoreRef<'a, Val>, scope: impl Into<Option<ScopeId>>) -> Self {
        Self {
            scope: scope.into(),
            store,
        }
    }
}

#[derive(Debug)]
pub enum ControlFlowExpr<T> {
    If(ExpressionValue<T>),
    Else(Option<ExpressionValue<T>>),
}

#[derive(Debug)]
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
        node_id: impl Into<NodeId>,
    ) -> Result<Node<Output>, Output::Err> {
        let node_id = node_id.into();
        match self {
            Self::Node {
                context,
                children,
                attributes,
            } => {
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
                let collection =
                    collection.to_scope_value::<Output::Notifier>(eval.store, eval.scope, &node_id);
                let scope = eval.store.new_scope(eval.scope);
                let state = LoopState::new(scope, *binding, collection, body.clone());
                Ok(NodeKind::Collection(state).to_node(node_id))
            }
            Self::ControlFlow(flows) => {
                let mut node_flows = vec![];
                for (cond, expressions) in flows {
                    let cond = match cond {
                        ControlFlowExpr::If(val) => {
                            let val = val.to_scope_value::<Output::Notifier>(
                                eval.store, eval.scope, &node_id,
                            );
                            ControlFlow::If(val)
                        }
                        ControlFlowExpr::Else(Some(val)) => {
                            let val = val.to_scope_value::<Output::Notifier>(
                                eval.store, eval.scope, &node_id,
                            );
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
    type Ctx: std::fmt::Debug;
    type Value: Truthy + Clone + std::fmt::Debug;
    type Err;
    type Notifier: Listen<Key = NodeId, Value = Self::Value>;

    fn from_context(ctx: DataCtx<'_, Self>) -> Result<Self, Self::Err>;
}

#[cfg(test)]
mod test {
    use anathema_values::Store;

    use super::*;
    use crate::testing::{controlflow, expression, for_expression, Widget};

    #[test]
    fn eval_node_expression() {
        let store = Store::empty();
        let store_ref = store.read();
        let expr = expression("node_name", (), ());
        let ctx = EvaluationContext::new(&store_ref, None);
        let node = expr.to_node(&ctx, 0).unwrap();

        assert_eq!(NodeId::new(0), *node.id());
        assert_eq!(Widget { ident: "node_name" }, node.single().unwrap().0);
    }

    #[test]
    fn eval_for_loop() {
        let store = Store::empty();
        let store_ref = store.read();
        let expr = for_expression(0, [1, 2], expression("node_here", (), ()));
        let ctx = EvaluationContext::new(&store_ref, None);
        let node = expr.to_node(&ctx, 0).unwrap();
        let mut nodes = Nodes::new(vec![node]);
        assert!(nodes.next(&store_ref).is_some());
        assert!(nodes.next(&store_ref).is_some());
        assert!(nodes.next(&store_ref).is_none());
    }

    #[test]
    fn eval_controlflow() {
        fn eval<
            A: Into<ControlFlowExpr<u32>>,
            B: Into<ControlFlowExpr<u32>>,
            C: Into<ControlFlowExpr<u32>>,
        >(
            conds: (A, B, C),
        ) -> Widget {
            let store = Store::<u32>::empty();
            let store_ref = store.read();
            let ctx = EvaluationContext::new(&store_ref, None);
            let expr = controlflow([
                (conds.0.into(), vec![expression("it's true", (), ())]),
                (conds.1.into(), vec![expression("else this", (), ())]),
                (conds.2.into(), vec![expression("this", (), ())]),
            ]);
            let node = expr.to_node(&ctx, 0).unwrap();
            let mut nodes = Nodes::new(vec![node]);
            *nodes.next(&store_ref).unwrap().unwrap().0
        }

        let node = eval((1u32, Some(1u32), None));
        assert_eq!(node.ident, "it's true");
        let node = eval((0u32, Some(1u32), None));
        assert_eq!(node.ident, "else this");
        let node = eval((0u32, Some(0u32), None));
        assert_eq!(node.ident, "this");
    }
}
