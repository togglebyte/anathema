use anathema_render::Size;
use anathema_values::{
    Attributes, Context, Deferred, DynValue, ExpressionMap, Expressions, Immediate, NextNodeId,
    NodeId, Path, ScopeStorage, State, Value, ValueExpr, ValueRef,
};

pub use self::controlflow::{ElseExpr, IfExpr};
use crate::error::Result;
use crate::factory::FactoryContext;
use crate::nodes::{IfElse, LoopNode, Node, NodeKind, Nodes, Single, View};
use crate::views::{RegisteredViews, Views};
use crate::{Factory, Pos, WidgetContainer};

mod controlflow;

// Create the root view, this is so events can be handled and state can
// be associated with the root view, without having to register additional
// views.
pub fn root_view(body: Vec<Expression>, id: usize) -> Expression {
    Expression::View(ViewExpr {
        id,
        state: None,
        body,
        attributes: Attributes::new(),
    })
}

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
        let scope = context.clone_scope();

        let text = self
            .text
            .as_ref()
            .map(|text| String::init_value(context, &node_id, text))
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
    State { len: usize, expr: &'e ValueExpr },
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
                let mut resolver = Deferred::new(context.lookup());
                let val = col.eval(&mut resolver);
                match val {
                    ValueRef::Expressions(Expressions(list)) => Collection::Static(list),
                    ValueRef::Deferred => {
                        let mut resolver = Immediate::new(context.lookup(), &node_id);
                        let val = col.eval(&mut resolver);
                        let len = match val {
                            ValueRef::List(list) => list.len(),
                            _ => 0,
                        };

                        Collection::State { expr: col, len }
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
            scope: ScopeStorage::new(),
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
        let inner_node_id = node_id.child(0);
        let next_node = NextNodeId::new(node_id.last());

        let node = Node {
            kind: NodeKind::ControlFlow(IfElse::new(
                &self.if_expr,
                &self.elses,
                context,
                inner_node_id,
                next_node,
            )),
            node_id,
            scope: ScopeStorage::new(),
        };
        Ok(node)
    }
}

#[derive(Debug)]
pub(crate) enum ViewState<'e> {
    Dynamic(&'e dyn State),
    External { expr: &'e ValueExpr },
    Map(ExpressionMap<'e>),
    Internal,
}

#[derive(Debug, Clone)]
pub struct ViewExpr {
    pub id: usize,
    pub state: Option<ValueExpr>,
    pub body: Vec<Expression>,
    pub attributes: Attributes,
}

impl ViewExpr {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        let tabindex = self
            .attributes
            .get("tabindex") // TODO: should be a constant. Look into reserving (more) keywords
            .map(|expr| u32::init_value(context, &node_id, expr))
            .unwrap_or(Value::Empty);

        Views::insert(node_id.clone(), tabindex.value());

        let state = match self.state {
            Some(ref expr) => {
                let mut resolver = Deferred::new(context.lookup());
                let val = expr.eval(&mut resolver);
                match val {
                    ValueRef::Map(state) => ViewState::Dynamic(state),
                    ValueRef::Deferred => ViewState::External { expr },
                    ValueRef::ExpressionMap(map) => ViewState::Map(map),
                    _ => ViewState::Internal,
                }
            }
            None => ViewState::Internal,
        };

        let node = Node {
            kind: NodeKind::View(View {
                view: RegisteredViews::get(self.id)?,
                nodes: Nodes::new(&self.body, node_id.child(0)),
                state,
                tabindex,
            }),
            node_id,
            scope: ScopeStorage::new(),
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
    impl crate::views::View for AView {}

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
            for_expression("item", list([1, 2, 3]), [expression("test", None, [], [])]).test();
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
        let expr = view_expression(12345, None, vec![]).test();
        let _ = expr.eval().unwrap();
    }

    #[test]
    fn eval_prototype_view() {
        RegisteredViews::add_prototype(0, || AView);

        let expr = view_expression(0, None, vec![]).test();
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
        RegisteredViews::add_view(0, AView);
        let expr = view_expression(0, None, vec![]).test();
        let _ = expr.eval().unwrap();
        let _ = expr.eval().unwrap();
    }
}
