mod controlflow;

use std::rc::Rc;

use anathema_values::{Path, State};
use controlflow::Cond;

use self::controlflow::FlowState;
use crate::nodes::{Node, NodeKind, Nodes};
use crate::{Attributes, IntoWidget, NodeId, Value};

#[derive(Debug)]
pub enum Expression<Widget: IntoWidget> {
    Node {
        context: Rc<Widget::Meta>,
        attributes: Attributes,
        children: Rc<[Expression<Widget>]>,
    },
    Loop {
        body: Rc<[Expression<Widget>]>,
        binding: Path,
        collection: Value,
    },
    ControlFlow(FlowState),
}

impl<Widget: IntoWidget> Expression<Widget> {
    pub(crate) fn eval(
        &self,
        state: &mut Widget::State,
        node_id: NodeId,
    ) -> Result<Node<Widget>, Widget::Err> {
        let node = match self {
            Self::Node {
                context,
                attributes,
                children,
            } => {
                let item = Widget::create_widget(context, state, attributes)?;
                Node {
                    kind: NodeKind::Single(item, Nodes::new(children.clone(), node_id.child(0))),
                    node_id,
                }
            }
            Self::Loop { body, binding, collection } => Node {
                kind: NodeKind::Loop {
                    body: Nodes::new(body.clone(), node_id.child(0)),
                    binding: binding.clone(),
                    collection: eval the collection,
                    value_index: 0,
                },
                node_id,
            },
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
        let expr = expression("text", (), []);
        let mut node = expr.eval(&mut (), 0.into()).unwrap();
        let (widget, _) = node.single();
        assert_eq!("text", &*widget.ident);
    }

    #[test]
    fn eval_for() {
        let expr = for_expression("item", [1, 2, 3], [expression("text", (), [])]);
        let node = expr.eval(&mut (), 0.into()).unwrap();
        assert!(matches!(
            node,
            Node {
                kind: NodeKind::Loop { .. },
                ..
            }
        ));
    }
}
