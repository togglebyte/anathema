use std::rc::Rc;

use anathema_render::Size;
use anathema_values::{Collection, Context, NodeId, Path, Scope, ScopeValue, State, ValueExpr};

pub use self::controlflow::{Else, If};
use super::nodes::LoopNode;
use crate::error::Result;
use crate::generator::nodes::{Node, NodeKind, Nodes};
use crate::generator::Attributes;
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
    pub children: Rc<[Expression]>,
}

impl SingleNode {
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
        panic!()
        // let context = Context::new(state, scope);

        // let widget = WidgetContainer {
        //     background: context.attribute("background", Some(&node_id), &self.attributes),
        //     display: context
        //         .attribute("display", Some(&node_id), &self.attributes)
        //         .unwrap_or(Display::Show),
        //     padding: context
        //         .attribute("padding", Some(&node_id), &self.attributes)
        //         .unwrap_or(Padding::ZERO),
        //     pos: Pos::ZERO,
        //     size: Size::ZERO,
        //     inner: Factory::exec(context, &self, &node_id)?,
        //     node_id: node_id.clone(),
        // };

        // let node = Node {
        //     kind: NodeKind::Single(widget, Nodes::new(self.children.clone(), node_id.child(0))),
        //     node_id,
        // };

        // Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Loop {
    pub body: Rc<[Expression]>,
    pub binding: Path,
    pub collection: ValueExpr,
}

impl Loop {
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
        panic!()
        // let collection: Collection =
        //     match &self.collection {
        //         ScopeValue::List(values) => Collection::Rc(values.clone()),
        //         ScopeValue::Dyn(path) => scope
        //             .lookup_list(path)
        //             .map(Collection::Rc)
        //             .unwrap_or_else(|| {
        //                 state
        //                     .get_collection(path, Some(&node_id))
        //                     .unwrap_or(Collection::Empty)
        //             }),
        //         ScopeValue::Static(_) | ScopeValue::Invalid => Collection::Empty,
        //     };

        // let node = Node {
        //     kind: NodeKind::Loop(LoopNode::new(
        //         Nodes::new(self.body.clone(), node_id.child(0)),
        //         self.binding.clone(),
        //         collection,
        //     )),
        //     node_id,
        // };

        // Ok(node)
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
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
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
    pub(crate) fn eval(
        &self,
        state: &mut dyn State,
        scope: &mut Scope,
        node_id: NodeId,
    ) -> Result<Node> {
        match self {
            Self::Node(node) => node.eval(state, scope, node_id),
            Self::Loop(loop_expr) => loop_expr.eval(state, scope, node_id),
            Self::ControlFlow(controlflow) => controlflow.eval(state, scope, node_id),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::testing::*;

    #[test]
    fn eval_node() {
        register_test_widget();
        let mut scope = Scope::new(None);
        let expr = expression("test", None, [], []);
        let mut node = expr.eval(&mut (), &mut scope, 0.into()).unwrap();
        let (widget, _) = node.single();
        assert_eq!("test", widget.kind());
    }

    #[test]
    fn eval_for() {
        let mut scope = Scope::new(None);
        let expr = for_expression("item", [1, 2, 3], [expression("test", None, [], [])]);
        let node = expr.eval(&mut (), &mut scope, 0.into()).unwrap();
        assert!(matches!(
            node,
            Node {
                kind: NodeKind::Loop { .. },
                ..
            }
        ));
    }
}
