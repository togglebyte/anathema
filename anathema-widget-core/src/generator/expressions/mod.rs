use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, Path, Scope, State, ValueExpr, ValueRef};

pub use self::controlflow::{Else, If};
use super::nodes::{IfElse, LoopNode};
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
    fn eval(&self, context: &Context<'_, '_>, node_id: NodeId) -> Result<Node> {
        // TODO: add > < >= <=, this message is not really about single nodes, but about evaluating
        // values, however this message was attached to another message so here we are... (the
        // other message was an issue that is now resolved under the name of FactoryContext)

        let context = FactoryContext::new(
            context,
            node_id.clone(),
            &self.ident,
            &self.attributes,
            self.text.as_ref(),
        );

        let widget = WidgetContainer {
            inner: Factory::exec(context)?,
            background: None,       //context.background(),
            display: Display::Show, //context.display(),
            padding: Padding::ZERO, // context.padding(),
            pos: Pos::ZERO,
            size: Size::ZERO,
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

#[derive(Debug)]
pub enum Lol<'e> {
    Things(&'e [ValueExpr]),
    Path(Path),
    Nothing,
}

impl Loop {
    fn eval(&self, context: &Context<'_, '_>, node_id: NodeId) -> Result<Node<'_>> {

        let collection = match &self.collection {
            ValueExpr::List(expr) => Lol::Things(expr),
            ValueExpr::Ident(_) | ValueExpr::Dot(..) | ValueExpr::Index(..) => {
                match self.collection.eval_path(context, Some(&node_id)) {
                    Some(path) => {
                        match context.scope.lookup(&path) {
                            // Some(ValueRef::Expressions(value)) => {
                            //     Lol::Things(value)
                            // }
                            _ => Lol::Path(path)
                        }
                    }
                    None => Lol::Nothing,
                }
            }
            _ => Lol::Nothing,
        };

        let node = Node {
            kind: NodeKind::Loop(LoopNode::new(
                Nodes::new(&self.body, node_id.child(0)),
                self.binding.clone(),
                collection,
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
    fn eval(&self, node_id: NodeId) -> Result<Node<'_>> {
        let node = Node {
            kind: NodeKind::ControlFlow(IfElse::new(&self.if_expr, &self.elses)),
            node_id,
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
    pub(crate) fn eval<'a: 'val, 'val>(
        &self,
        context: &Context<'a, 'val>,
        node_id: NodeId,
    ) -> Result<Node<'_>> {
        match self {
            Self::Node(node) => node.eval(context, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(context, node_id),
            Self::ControlFlow(controlflow) => controlflow.eval(node_id),
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
        let mut scope = Scope::new(None);
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
