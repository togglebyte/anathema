use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, Path, Scope, State, ValueExpr};

pub use self::controlflow::{Else, If};
use super::nodes::LoopNode;
use crate::error::Result;
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
    fn eval<'a: 'val, 'val>(
        &self,
        context: &Context<'a, 'val>,
        node_id: NodeId,
    ) -> Result<Node> {
        let widget = WidgetContainer {
            background: context
                .attribute("background", Some(&node_id), &self.attributes)
                .map(|val| *val),

            // TODO: don't hard code these
            display: Display::Show, /* context .attribute("display", Some(&node_id), &self.attributes) .unwrap_or(Display::Show), */
            padding: Padding::ZERO, /* context .attribute("padding", Some(&node_id), &self.attributes) .unwrap_or(Padding::ZERO), */

            pos: Pos::ZERO,
            size: Size::ZERO,
            inner: Factory::exec(context, &self, &node_id)?,
            node_id: node_id.clone(),
        };

        let node = Node {
            kind: NodeKind::Single(widget, Nodes::new(&self.children, node_id.child(0))),
            node_id,
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

impl Loop {
    fn eval<'a: 'val, 'val>(&self, node_id: NodeId) -> Result<Node<'_>> {
        let node = Node {
            kind: NodeKind::Loop(LoopNode::new(
                Nodes::new(&self.body, node_id.child(0)),
                self.binding.clone(),
                &self.collection,
            )),
            node_id,
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Controlflow -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ControlFlow {
    pub if_expr: If,
    pub elses: Vec<Else>,
}

impl ControlFlow {
    fn eval(&self, state: &dyn State, scope: &Scope<'_>, node_id: NodeId) -> Result<Node> {
        if self.if_expr.is_true(scope, state, Some(&node_id)) {}
        panic!()
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
    pub(crate) fn eval<'a: 'val, 'val>(
        &self,
        context: &Context<'a, 'val>,
        node_id: NodeId,
    ) -> Result<Node> {
        match self {
            Self::Node(node) => node.eval(context, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(node_id),
            Self::ControlFlow(controlflow) => panic!(),//controlflow.eval(state, scope, node_id),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::TestState;

    use super::*;
    use crate::contexts::LayoutCtx;
    use crate::generator::testing::*;
    use crate::layout::Constraints;

    impl Expression {
        pub fn test<'a>(self) -> TestExpression<'a, TestState> {
            register_test_widget();
            let scope = Scope::new(None);

            let constraint = Constraints::new(80, 20);

            TestExpression {
                state: TestState::new(),
                scope,
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
        assert_eq!("test", widget.kind());
    }

    #[test]
    fn eval_for() {
        panic!()
        // let mut scope = Scope::new(None);
        // let expr = for_expression("item", [1, 2, 3], [expression("test", None, [], [])]);
        // let node = expr.eval(&mut (), &mut scope, 0.into()).unwrap();
        // assert!(matches!(
        //     node,
        //     Node {
        //         kind: NodeKind::Loop { .. },
        //         ..
        //     }
        // ));
    }
}
