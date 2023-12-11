use anathema_render::Size;
use anathema_values::{
    Attributes, Context, Deferred, DynValue, NodeId, Path, State, ValueExpr, ValueRef,
};

pub use self::controlflow::{ElseExpr, IfExpr};
use crate::error::Result;
use crate::factory::FactoryContext;
use crate::nodes::{IfElse, LoopNode, Node, NodeKind, Nodes, Single, View};
use crate::views::{RegisteredViews, TabIndex, Views};
use crate::{Factory, Pos, WidgetContainer};

mod controlflow;

// -----------------------------------------------------------------------------
//   - A single Node -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct SingleNodeExpr {
    pub ident: String,
    pub text: Option<ValueExpr>,
    pub attributes: Attributes,
    pub children: Vec<Expression>,
}

impl SingleNodeExpr {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        // TODO: add > < >= <=, this message is not really about single nodes, but about evaluating
        // values, however this message was attached to another message so here we are... (the
        // other message was an issue that is now resolved under the name of FactoryContext)

        let scope = context.new_scope();

        let text = self
            .text
            .as_ref()
            .map(|text| String::init_value(context, Some(&node_id), text))
            .unwrap_or_default();

        let context = FactoryContext::new(
            context,
            node_id.clone(),
            &self.ident,
            &self.attributes,
            text,
        );

        let widget = WidgetContainer {
            display: context.get("display"),
            background: context.get("background"),
            padding: context.get("padding"),
            pos: Pos::ZERO,
            size: Size::ZERO,

            node_id: node_id.clone(),
            inner: Factory::exec(context)?,
            expr: None,
            attributes: &self.attributes,
        };

        let node = Node {
            kind: NodeKind::Single(Single {
                widget,
                children: Nodes::new(&self.children, node_id.child(0)),
                ident: &self.ident,
            }),
            node_id,
            scope,
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) enum Collection<'e> {
    Static(&'e [ValueExpr]),
    State { len: usize, path: Path },
    Empty,
}

impl<'e> Collection<'e> {
    pub(super) fn push(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
        }
    }

    pub(super) fn insert(&mut self, index: usize) {
        if let Collection::State { len, .. } = self {
            if index <= *len {
                *len += 1;
            }
        }
    }

    pub(super) fn remove(&mut self) {
        if let Collection::State { len, .. } = self {
            if *len > 0 {
                *len -= 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub body: Vec<Expression>,
    pub binding: Path,
    pub collection: ValueExpr,
}

impl LoopExpr {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        // Need to know if this is a collection or a path
        let collection = match &self.collection {
            ValueExpr::List(list) => Collection::Static(list),
            col => {
                let mut resolver = Deferred::new(context);
                let val = resolver.resolve(col);
                match val {
                    ValueRef::Expressions(list) => Collection::Static(list),
                    ValueRef::Deferred(path) => {
                        let len = match context.state.get(&path, Some(&node_id)) {
                            ValueRef::List(list) => list.len(),
                            _ => 0,
                        };

                        Collection::State { path, len }
                    }
                    _ => Collection::Empty,
                }
            }
        };

        let loop_node = LoopNode::new(
            &self.body,
            self.binding.clone(),
            collection,
            node_id.child(0),
        );

        let node = Node {
            kind: NodeKind::Loop(loop_node),
            node_id,
            scope: context.new_scope(),
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Controlflow -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ControlFlow {
    pub if_expr: IfExpr,
    pub elses: Vec<ElseExpr>,
}

impl ControlFlow {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        let node = Node {
            kind: NodeKind::ControlFlow(IfElse::new(
                &self.if_expr,
                &self.elses,
                context,
                node_id.child(0),
            )),
            node_id,
            scope: context.new_scope(),
        };
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) enum ViewState<'e> {
    Static(&'e dyn State),
    External { path: Path },
    Internal,
}

#[derive(Debug, Clone)]
pub struct ViewExpr {
    pub id: String,
    pub state: Option<ValueExpr>,
    pub body: Vec<Expression>,
}

impl ViewExpr {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        TabIndex::insert(node_id.clone());
        Views::insert(node_id.clone());

        let state = match &self.state {
            Some(expr) => {
                let mut resolver = Deferred::new(context);
                let val = resolver.resolve(expr);
                match val {
                    ValueRef::Map(state) => ViewState::Static(state),
                    ValueRef::Deferred(path) => ViewState::External { path },
                    _ => ViewState::Internal,
                }
            }
            None => ViewState::Internal,
        };

        let node = Node {
            kind: NodeKind::View(View {
                view: RegisteredViews::get(&self.id)?,
                nodes: Nodes::new(&self.body, node_id.child(0)),
                state,
            }),
            node_id,
            scope: context.new_scope(),
        };
        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Expression -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Expression {
    Node(SingleNodeExpr),
    View(ViewExpr),
    Loop(LoopExpr),
    ControlFlow(ControlFlow),
}

impl Expression {
    pub(crate) fn eval<'expr>(
        &'expr self,
        context: &Context<'_, 'expr>,
        node_id: NodeId,
    ) -> Result<Node<'expr>> {
        match self {
            Self::Node(node) => node.eval(context, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(context, node_id),
            Self::ControlFlow(controlflow) => controlflow.eval(context, node_id),
            Self::View(view_expr) => view_expr.eval(context, node_id),
        }
    }
}

#[cfg(all(test, feature = "testing"))]
mod test {
    use anathema_values::testing::{list, TestState};

    use super::*;
    use crate::contexts::LayoutCtx;
    use crate::layout::Constraints;
    use crate::testing::expressions::{expression, for_expression, if_expression, view_expression};
    use crate::testing::nodes::*;
    use crate::Padding;

    impl Expression {
        pub fn test(self) -> TestExpression<TestState> {
            register_test_widget();

            let constraint = Constraints::new(80, 20);

            TestExpression {
                state: TestState::new(),
                expr: Box::new(self),
                layout: LayoutCtx::new(constraint, Padding::ZERO),
            }
        }
    }

    #[derive(Debug)]
    struct AView;
    impl crate::views::View for AView {
        type State = ();
    }

    #[test]
    fn eval_node() {
        let test = expression("test", None, [], []).test();
        let mut node = test.eval().unwrap();
        let (widget, _) = node.single();
        assert_eq!("text", widget.kind());
    }

    #[test]
    fn eval_for() {
        let expr =
            for_expression("item", *list([1, 2, 3]), [expression("test", None, [], [])]).test();
        let node = expr.eval().unwrap();

        assert!(matches!(
            node,
            Node {
                kind: NodeKind::Loop { .. },
                ..
            }
        ));
    }

    #[test]
    fn eval_if() {
        let expr = if_expression(
            (true.into(), vec![expression("test", None, [], [])]),
            vec![],
        )
        .test();

        let node = expr.eval().unwrap();

        assert!(matches!(
            node,
            Node {
                kind: NodeKind::ControlFlow(..),
                ..
            }
        ));
    }

    #[test]
    #[should_panic(expected = "ViewNotFound")]
    fn eval_missing_view() {
        let expr = view_expression("theview", None, vec![]).test();
        let _ = expr.eval().unwrap();
    }

    #[test]
    fn eval_prototype_view() {
        RegisteredViews::add_prototype("aview", || AView);

        let expr = view_expression("aview", None, vec![]).test();
        let node = expr.eval().unwrap();

        assert!(matches!(
            node,
            Node {
                kind: NodeKind::View(..),
                ..
            }
        ));
    }

    #[test]
    #[should_panic(expected = "ViewConsumed")]
    fn consume_view_twice() {
        RegisteredViews::add_view("aview", AView);
        let expr = view_expression("aview", None, vec![]).test();
        let _ = expr.eval().unwrap();
        let _ = expr.eval().unwrap();
    }
}