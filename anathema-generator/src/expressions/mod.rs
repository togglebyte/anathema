use std::rc::Rc;

use anathema_values::{Collection, Context, Path, Scope, ScopeValue, State, NodeId};

use self::controlflow::{Else, If};
use crate::nodes::{LoopNode, Node, NodeKind, Nodes};
use crate::{Attributes, IntoWidget};

mod controlflow;

// -----------------------------------------------------------------------------
//   - A single Node -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct SingleNode<Widget: IntoWidget> {
    pub meta: Rc<Widget>,
    pub attributes: Attributes,
    pub children: Rc<[Expression<Widget>]>,
}

impl<WidgetMeta: IntoWidget> SingleNode<WidgetMeta> {
    fn eval<S: State>(
        &self,
        state: &mut S,
        scope: &mut Scope<'_>,
        node_id: NodeId,
    ) -> Result<Node<WidgetMeta>, WidgetMeta::Err> {
        let context = Context::new(state, scope);
        let item = self.meta.create_widget(context, &self.attributes)?;
        let node = Node {
            kind: NodeKind::Single(item, Nodes::new(self.children.clone(), node_id.child(0))),
            node_id,
        };
        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Loop<Widget: IntoWidget> {
    pub body: Rc<[Expression<Widget>]>,
    pub binding: Path,
    pub collection: ScopeValue,
}

impl<Widget: IntoWidget> Loop<Widget> {
    fn eval<S: State>(
        &self,
        state: &mut S,
        scope: &mut Scope<'_>,
        node_id: NodeId,
    ) -> Result<Node<Widget>, Widget::Err> {
        let collection: Collection = match &self.collection {
            ScopeValue::List(values) => Collection::Rc(values.clone()),
            ScopeValue::Static(string) => Collection::Empty,
            ScopeValue::Dyn(path) => scope
                .lookup_list(path)
                .map(Collection::Rc)
                .unwrap_or_else(|| state.get_collection(path, Some(&node_id)).unwrap_or(Collection::Empty)),
        };

        scope.scope_collection(self.binding.clone(), &collection, 0);

        let node = Node {
            kind: NodeKind::Loop(LoopNode {
                body: Nodes::new(self.body.clone(), node_id.child(0)),
                binding: self.binding.clone(),
                collection,
                value_index: 0,
            }),
            node_id,
        };

        Ok(node)
    }
}

// -----------------------------------------------------------------------------
//   - Controlflow -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ControlFlow<Widget: IntoWidget> {
    if_expr: If<Widget>,
    elses: Vec<Else<Widget>>,
}

impl<Widget: IntoWidget> ControlFlow<Widget> {
    fn eval<S: State>(
        &self,
        state: &mut S,
        scope: &mut Scope<'_>,
        node_id: NodeId,
    ) -> Result<Node<Widget>, Widget::Err> {
        if self.if_expr.is_true(scope, state) {}

        panic!()
    }
}

// -----------------------------------------------------------------------------
//   - Expression -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Expression<Widget: IntoWidget> {
    Node(SingleNode<Widget>),
    Loop(Loop<Widget>),
    ControlFlow(ControlFlow<Widget>),
}

impl<Widget: IntoWidget> Expression<Widget> {
    pub(crate) fn eval<S: State>(
        &self,
        state: &mut S,
        scope: &mut Scope<'_>,
        node_id: NodeId,
    ) -> Result<Node<Widget>, Widget::Err> {
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
