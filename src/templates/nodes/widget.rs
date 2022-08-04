use crate::widgets::{fields, Attributes, NodeId, Value};

use crate::templates::parser::Text;
use crate::templates::error::{Error, Result};
use super::template::TemplateNode;

mod keywords {
    pub(super) const FOR: &str = "for";
    pub(super) const IF: &str = "if";
    pub(super) const ELSE: &str = "else";
    pub(super) const COND: &str = "cond";
    pub(super) const INCLUDE: &str = "include";
}

static ID_EXCEMPT: &[&str] = &["span", "for", "if", "else", "elif", "include"];

#[derive(Debug, Clone)]
pub(super) enum Statement {
    Node { children: Vec<WidgetNode> },
    If { children: Vec<WidgetNode>, cond: Value, elses: Vec<(Option<Value>, Vec<WidgetNode>)> },
    For { binding: Value, data: Value, template: Vec<WidgetNode> },
    Include { path: Text },
}

// Note:
// The widget node is created once and once only, by consuming its
// corresponding `TemplateNode`.
//
// However a single `WidgetNode` can produce different `Widget`s depending
// on the `DataCtx`.
#[derive(Debug, Clone)]
pub struct WidgetNode {
    pub(super) ident: String,
    pub(super) text: Option<Text>,
    pub(super) attributes: Attributes,
    pub(super) stmt: Statement,
    pub(super) node_id: NodeId,
}

impl WidgetNode {
    pub(crate) fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }
}

pub(crate) fn to_widget_nodes(node_tree: Vec<TemplateNode<'_>>, needs_id: bool) -> Result<Vec<WidgetNode>> {
    let mut nodes = Vec::with_capacity(node_tree.len());

    let mut tree = node_tree.into_iter().peekable();

    while let Some(mut node) = tree.next() {
        let stmt = match node.ident {
            keywords::IF => {
                let children = to_widget_nodes(node.children, true)?;
                let cond = node.attributes.get_value(keywords::COND).ok_or(Error::MissingCondition)?;
                let mut elses = vec![];

                while let Some(sib) = tree.next_if(|n| n.ident == keywords::ELSE) {
                    elses.push((sib.attributes.get_value(keywords::COND), to_widget_nodes(sib.children, true)?));
                }

                Statement::If { children, cond, elses }
            }
            keywords::FOR => {
                let binding = match node.attributes.get_value("binding") {
                    Some(binding @ Value::String(_)) => binding,
                    _ => return Err(Error::BindingInvalidString),
                };

                // Make sure `data` is a collection
                let data = match node.attributes.get_value("data") {
                    Some(data @ Value::List(_)) => data,
                    Some(data @ Value::DataBinding(_)) => data,
                    _ => return Err(Error::NonCollectionValue),
                };

                let template = to_widget_nodes(node.children, true)?;
                Statement::For { binding, data, template }
            }
            keywords::INCLUDE => {
                let path = match node.text.take() {
                    Some(data @ Text::String(_)) => data,
                    Some(data @ Text::Fragments(_)) => data,
                    None => return Err(Error::MissingIncludePath),
                };
                Statement::Include { path }
            }
            _ => Statement::Node { children: to_widget_nodes(node.children, needs_id)? },
        };

        let node_id = node.attributes.take_value(fields::ID);
        let node_id = match node_id {
            Some(val) => NodeId::Value(val), //NodeId::String(val.to_string()),
            // Some(Value::DataBinding(binding)) => ctx.by_path(binding),
            None if !ID_EXCEMPT.contains(&node.ident) && needs_id => return Err(Error::MissingId),
            None => NodeId::auto(),
        };

        let node =
            WidgetNode { ident: node.ident.to_string(), text: node.text, node_id, attributes: node.attributes, stmt };

        nodes.push(node);
    }

    Ok(nodes)
}
