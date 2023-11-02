use anathema_render::Size;
use anathema_values::{
    Attributes, Context, Deferred, DynValue, LocalScope, NodeId, Path, State, Value, ValueExpr,
    ValueRef, ValueResolver,
};

pub use self::controlflow::{ElseExpr, IfExpr};
use super::nodes::{IfElse, LoopNode, Single};
use crate::error::Result;
use crate::factory::FactoryContext;
use crate::generator::nodes::{Node, NodeKind, Nodes};
use crate::{Display, Factory, Padding, Pos, WidgetContainer};

mod controlflow;

// -----------------------------------------------------------------------------
//   - A single Node -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct SingleNode {
    pub ident: String,
    pub text: Option<ValueExpr>,
    pub attributes: Attributes,
    pub children: Vec<Expression>,
}

impl SingleNode {
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
            inner: Factory::exec(context)?,
            background: None,       //context.background(),
            display: Display::Show, //context.display(),
            padding: Padding::ZERO, // context.padding(),
            pos: Pos::ZERO,
            size: Size::ZERO,
            node_id: node_id.clone(),
            expr: None,
        };

        let node = Node {
            kind: NodeKind::Single(Single {
                widget,
                children: Nodes::new(&self.children, node_id.child(0)),
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
pub struct Loop {
    pub body: Vec<Expression>,
    pub binding: Path,
    pub collection: ValueExpr,
}

#[derive(Debug)]
pub enum Collection<'e> {
    ValueExpressions(&'e [ValueExpr]),
    State { len: usize, path: Path },
    Path(Path),
    Empty,
}

impl<'e> Collection<'e> {
    pub(super) fn add(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
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

impl Loop {
    fn eval<'e>(&'e self, context: &Context<'_, 'e>, node_id: NodeId) -> Result<Node<'e>> {
        let collection = match &self.collection {
            ValueExpr::List(expr) => Collection::ValueExpressions(expr),
            ValueExpr::Ident(_) | ValueExpr::Dot(..) | ValueExpr::Index(..) => {
                match Deferred::new(context).resolve_path(&self.collection) {
                    Some(path) => match context.state.get_collection(&path, Some(&node_id)) {
                        Some(len) => Collection::State { len, path },
                        None => Collection::Path(path),
                    },
                    None => Collection::Empty,
                }
            }
            _ => Collection::Empty,
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
#[derive(Debug)]
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

// -----------------------------------------------------------------------------
//   - Expression -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Expression {
    Node(SingleNode),
    Loop(Loop),
    ControlFlow(ControlFlow),
}

impl Expression {
    pub(crate) fn eval<'a, 'expr>(
        &'expr self,
        context: &Context<'a, 'expr>,
        node_id: NodeId,
    ) -> Result<Node<'expr>> {
        match self {
            Self::Node(node) => node.eval(context, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(context, node_id),
            Self::ControlFlow(controlflow) => controlflow.eval(context, node_id),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::{list, TestState};

    use super::*;
    use crate::contexts::LayoutCtx;
    use crate::generator::testing::*;
    use crate::layout::Constraints;

    impl Expression {
        pub fn test<'a>(self) -> TestExpression<TestState> {
            register_test_widget();

            let constraint = Constraints::new(80, 20);

            TestExpression {
                state: TestState::new(),
                expr: Box::new(self),
                layout: LayoutCtx::new(constraint, Padding::ZERO),
            }
        }
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
}
