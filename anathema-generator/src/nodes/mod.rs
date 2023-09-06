use std::borrow::Cow;
use std::ops::DerefMut;
use std::rc::Rc;

use anathema_values::{Collection, Path, Scope, ScopeValue, State, Context, NodeId};

use self::controlflow::{Else, If};
use crate::expressions::Expression;
use crate::IntoWidget;

mod controlflow;

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct LoopNode<Widget: IntoWidget> {
    pub(crate) body: Nodes<Widget>,
    pub(crate) binding: Path,
    pub(crate) collection: Collection,
    pub(crate) value_index: usize,
}

#[derive(Debug)]
pub struct Node<WidgetMeta: IntoWidget> {
    pub node_id: NodeId,
    pub(crate) kind: NodeKind<WidgetMeta>,
}

#[cfg(test)]
impl<Widget: IntoWidget> Node<Widget> {
    pub(crate) fn single(&mut self) -> (&mut Widget, &mut Nodes<Widget>) {
        match &mut self.kind {
            NodeKind::Single(inner, nodes) => (inner, nodes),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeKind<WidgetMeta: IntoWidget> {
    Single(WidgetMeta::Widget, Nodes<WidgetMeta>),
    Loop(LoopNode<WidgetMeta>),
    ControlFlow {
        if_node: If<WidgetMeta>,
        elses: Vec<Else<WidgetMeta>>,
        body: Nodes<WidgetMeta>,
    },
}

#[derive(Debug)]
// TODO: possibly optimise this by making nodes optional on the node
pub struct Nodes<Widget: IntoWidget> {
    expressions: Rc<[Expression<Widget>]>,
    inner: Vec<Node<Widget>>,
    active_loop: Option<Box<Node<Widget>>>,
    expr_index: usize,
    next_id: NodeId,
    node_index: usize,
}

impl<WidgetMeta: IntoWidget> Nodes<WidgetMeta> {
    pub(crate) fn new(expressions: Rc<[Expression<WidgetMeta>]>, next_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            active_loop: None,
            expr_index: 0,
            next_id,
            node_index: 0,
        }
    }

    fn reset(&mut self) {
        self.expr_index = 0;
        self.node_index = 0;
    }

    fn eval_active_loop<S: State>(
        &mut self,
        state: &mut S,
        scope: &mut Scope<'_>,
    ) -> Option<Result<(), WidgetMeta::Err>> {
        if let Some(active_loop) = self.active_loop.as_mut() {
            let Node {
                kind: NodeKind::Loop(loop_node),
                node_id: parent_id,
            } = active_loop.deref_mut()
            else {
                unreachable!()
            };

            let ScopeValue::Static(item) = scope.lookup(&"item".into()).unwrap() else {
                panic!()
            };
            let item: &str = &*item;

            match loop_node.body.next(state, scope) {
                result @ Some(_) => return result,
                None => {
                    loop_node.value_index += 1;
                    if loop_node.value_index == loop_node.collection.len() {
                        self.inner.push(*self.active_loop.take().expect(""));
                        self.expr_index += 1;
                    } else {
                        scope.scope_collection(
                            loop_node.binding.clone(),
                            &loop_node.collection,
                            loop_node.value_index,
                        );
                        loop_node.body.reset();

                        return self.next(state, scope);
                    }
                }
            }
        }

        None
    }

    pub fn next<S: State>(
        &mut self,
        state: &mut S,
        scope: &mut Scope<'_>,
    ) -> Option<Result<(), WidgetMeta::Err>> {
        if let ret @ Some(_) = self.eval_active_loop(state, scope) {
            return ret;
        }

        let expr = self.expressions.get(self.expr_index)?;
        let node = match expr.eval(state, scope, self.next_id.clone()) {
            Ok(node) => node,
            Err(e) => return Some(Err(e)),
        };

        match node.kind {
            NodeKind::Single(element, nodes) => {
                self.expr_index += 1;
                let context = Context::new(state, scope);
                // let size = element.layout(nodes, layout, context);
                Some(Ok(()))
            }
            NodeKind::Loop { .. } => {
                self.active_loop = Some(node.into());
                self.next(state, scope)
            }
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }
    
    // pub fn layout(&mut self, layout: LayoutCtx, context: Context<'_, '_>) -> Result<WidgetMeta::Output> {
    //     self.node_index = 0;
    //     self.next()
    // }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut WidgetMeta::Widget, &mut Nodes<WidgetMeta>)> + '_ {
        self.inner
            .iter_mut()
            .map(|node| -> Box<dyn Iterator<Item = (&mut WidgetMeta::Widget, &mut Nodes<WidgetMeta>)>> {
                match &mut node.kind {
                    NodeKind::Single(widget, nodes) => Box::new(std::iter::once((widget, nodes))),
                    NodeKind::Loop(LoopNode { body, .. }) => Box::new(body.iter_mut()),
                    NodeKind::ControlFlow { body, .. } => Box::new(body.iter_mut()),
                }
            })
            .flatten()
    }

    pub fn first_mut(&mut self) -> Option<(&mut WidgetMeta::Widget, &mut Nodes<WidgetMeta>)> {
        self.iter_mut().next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    #[test]
    fn generate_a_single_widget() {
        let mut state = ();
        let mut scope = Scope::new(None);

        let expr = expression("text", (), []);
        let mut node = expr.eval(&mut state, &mut scope, 0.into()).unwrap();
        let (widget, nodes) = node.single();

        assert_eq!(&*widget.ident, "text");
    }

    #[test]
    fn for_loop() {
        let mut state = ();
        let mut scope = Scope::new(None);

        let body = expression("text", (), []);
        let for_loop = for_expression("item", [1, 2, 3], [body]);
        let mut nodes = Nodes::new(vec![for_loop].into(), NodeId::new(0));

        let node_1 = nodes.next(&mut state, &mut scope);
        let node_2 = nodes.next(&mut state, &mut scope);
        let node_3 = nodes.next(&mut state, &mut scope);
        let node_none = nodes.next(&mut state, &mut scope);

        assert!(node_1.is_some());
        assert!(node_2.is_some());
        assert!(node_3.is_some());
        assert!(node_none.is_none());
    }
}
