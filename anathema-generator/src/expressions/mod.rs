mod controlflow;

use std::rc::Rc;

use anathema_values::{Collection, Context, Path, Scope, ScopeValue, State};
use controlflow::Cond;

use self::controlflow::FlowState;
use crate::nodes::{Node, NodeKind, Nodes};
use crate::{Attributes, IntoWidget, NodeId};

#[derive(Debug)]
pub enum Expression<Widget: IntoWidget> {
    Node {
        meta: Rc<Widget::Meta>,
        attributes: Attributes,
        children: Rc<[Expression<Widget>]>,
    },
    Loop {
        body: Rc<[Expression<Widget>]>,
        binding: Path,
        collection: ScopeValue,
    },
    ControlFlow(FlowState),
}

impl<Widget: IntoWidget> Expression<Widget> {
    pub(crate) fn eval(
        &self,
        state: &mut Widget::State,
        scope: &mut Scope<'_>,
        node_id: NodeId,
    ) -> Result<Node<Widget>, Widget::Err> {
        let node = match self {
            Self::Node {
                meta,
                attributes,
                children,
            } => {
                let context = Context::new(state, scope);
                let item = Widget::create_widget(meta, context, attributes)?;
                Node {
                    kind: NodeKind::Single(item, Nodes::new(children.clone(), node_id.child(0))),
                    node_id,
                }
            }
            Self::Loop {
                body,
                binding,
                collection,
            } => {
                let collection: Collection = match collection {
                    ScopeValue::List(values) => Collection::Rc(values.clone()),
                    ScopeValue::Static(string) => Collection::Empty,
                    ScopeValue::Dyn(path) => scope
                        .lookup_list(path)
                        .map(Collection::Rc)
                        .unwrap_or_else(|| state.get_collection(path).unwrap_or(Collection::Empty)),
                };

                Node {
                    kind: NodeKind::Loop {
                        body: Nodes::new(body.clone(), node_id.child(0)),
                        binding: binding.clone(),
                        collection,
                        value_index: 0,
                    },
                    node_id,
                }
            }
            _ => panic!(),
        };

        Ok(node)
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
