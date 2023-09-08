use std::rc::Rc;

use anathema_render::Size;
use anathema_values::{Collection, Context, NodeId, Path, Scope, ScopeValue, State};

use self::controlflow::{Else, If};
use crate::error::Result;
use crate::generator::nodes::{Node, NodeKind, Nodes};
use crate::generator::Attributes;
use crate::{Display, Factory, Padding, Pos, WidgetContainer};

use super::nodes::LoopNode;

mod controlflow;

// -----------------------------------------------------------------------------
//   - A single Node -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct SingleNode {
    pub ident: String,
    pub text: Option<ScopeValue>,
    pub attributes: Attributes,
    pub children: Rc<[Expression]>,
}

impl SingleNode {
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
        let context = Context::new(state, scope);

        let widget = WidgetContainer {
            background: context.attribute("background", Some(&node_id), &self.attributes),
            display: context
                .attribute("display", Some(&node_id), &self.attributes)
                .unwrap_or(Display::Show),
            padding: context
                .attribute("padding", Some(&node_id), &self.attributes)
                .unwrap_or(Padding::ZERO),
            pos: Pos::ZERO,
            size: Size::ZERO,
            inner: Factory::exec(context, &self, &node_id)?,
            node_id: node_id.clone(),
        };

        let node = Node {
            kind: NodeKind::Single(widget, Nodes::new(self.children.clone(), node_id.child(0))),
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
    pub body: Rc<[Expression]>,
    pub binding: Path,
    pub collection: ScopeValue,
}

impl Loop {
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
        let collection: Collection =
            match &self.collection {
                ScopeValue::List(values) => Collection::Rc(values.clone()),
                ScopeValue::Static(string) => Collection::Empty,
                ScopeValue::Dyn(path) => scope
                    .lookup_list(path)
                    .map(Collection::Rc)
                    .unwrap_or_else(|| {
                        state
                            .get_collection(path, Some(&node_id))
                            .unwrap_or(Collection::Empty)
                    }),
            };

        let node = Node {
            kind: NodeKind::Loop(LoopNode::new(
                Nodes::new(self.body.clone(), node_id.child(0)),
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
    if_expr: If,
    elses: Vec<Else>,
}

impl ControlFlow {
    fn eval(&self, state: &mut dyn State, scope: &mut Scope<'_>, node_id: NodeId) -> Result<Node> {
        if self.if_expr.is_true(scope, state) {}

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
    use crate::testing::*;

    #[test]
    fn eval_node() {
        let mut scope = Scope::new(None);
        let expr = expression("text", (), []);
        let mut node = expr.eval(&mut (), &mut scope, 0.into()).unwrap();
        let (widget, _) = node.single();
        assert_eq!("text", &*widget.ident);
    }

    #[test]
    fn eval_for() {
        let mut scope = Scope::new(None);
        let expr = for_expression("item", [1, 2, 3], [expression("text", (), [])]);
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
