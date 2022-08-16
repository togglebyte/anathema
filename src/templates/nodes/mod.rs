use crate::widgets::{Attributes, Fragment, NodeId, Value};

use super::ctx::{NodeCtx, SubContext};
use super::error::{Error, Result};
use widget::{Statement, WidgetNode};

pub mod diff;
pub mod template;
pub mod widget;

static DEFAULT_VALUE: &Value = &Value::String(String::new());
const MAX_INCLUDE_DEPTH: usize = 42;

// -----------------------------------------------------------------------------
//     - Node kind -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Kind {
    Node { ident: String },
    Span(String),
}

// -----------------------------------------------------------------------------
//     - Node -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Node {
    pub kind: Kind,
    pub children: Vec<Node>,
    pub attributes: Attributes,
    pub(crate) id: NodeId,
}

impl Node {
    pub fn id(&self) -> NodeId {
        self.id.clone()
    }

    pub(crate) fn ident(&self) -> &str {
        match self.kind {
            Kind::Node { ref ident } => ident,
            Kind::Span(_) => "text",
        }
    }

    pub(crate) fn by_id(&self, id: &NodeId) -> Option<&Self> {
        if self.id.eq(id) {
            return Some(self);
        }

        for child in &self.children {
            if let Some(node) = child.by_id(id) {
                return Some(node);
            }
        }

        None
    }

    pub fn stringify(&self) -> String {
        to_string(self, 0)
    }
}

// Convert the node tree to a string
fn to_string(node: &Node, level: usize) -> String {
    let padding = " ".repeat(level * 4);
    let mut string = format!("{padding}{} {}\n", node.ident(), node.id);

    for child in &node.children {
        let s = to_string(child, level + 1);
        string.push_str(&s);
    }

    string
}

fn for_loop(
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
    binding: &Value,
    data: &Value,
    template: &[WidgetNode],
) -> Result<Vec<Node>> {
    // Lookup data if needed
    let data = if let Value::DataBinding(path) = data { data_ctx.by_path(path).unwrap_or(DEFAULT_VALUE) } else { data };

    // Lookup binding if needed
    let binding =
        if let Value::DataBinding(path) = binding { data_ctx.by_path(path).unwrap_or(DEFAULT_VALUE) } else { binding };

    let many = match data {
        Value::List(values) => {
            let mut nodes = vec![];

            let binding = binding.to_string();
            for value in values {
                let sub_ctx = data_ctx.sub(&binding, value.clone());
                for t in template {
                    nodes.extend(to_nodes(t, &sub_ctx, node_ctx)?);
                }
            }

            nodes
        }
        value => {
            let sub_ctx = data_ctx.sub(&binding.to_string(), value.clone());
            let mut nodes = vec![];
            for child in template {
                nodes.extend(to_nodes(child, &sub_ctx, node_ctx)?);
            }
            nodes
        }
    };

    Ok(many)
}

fn if_statement(
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
    cond: &Value,
    children: &[WidgetNode],
    elses: &[(Option<Value>, Vec<WidgetNode>)],
) -> Result<Vec<Node>> {
    let cond = match cond {
        Value::DataBinding(path) => match data_ctx.by_path(path) {
            Some(Value::Bool(val)) => *val,
            Some(_) | None => false,
        },
        Value::Bool(val) => *val,
        _ => panic!("bad template!: {cond:?}"),
    };

    if cond {
        let mut nodes = vec![];
        for child in children {
            nodes.extend(to_nodes(child, data_ctx, node_ctx)?);
        }
        // return children.iter().map(|child| to_nodes(child, data_ctx, node_ctx)).flatten().collect();
        return Ok(nodes);
    }

    for (cond, children) in elses {
        let cond = match cond {
            Some(Value::DataBinding(path)) => data_ctx.by_path(path).and_then(Value::to_bool).unwrap_or(false),
            Some(Value::Bool(val)) => *val,
            Some(_) => panic!("bad template!"),
            None => true,
        };

        if cond {
            let mut nodes = vec![];
            for child in children {
                nodes.extend(to_nodes(child, data_ctx, node_ctx)?);
            }
            // return children.iter().map(|child| to_nodes(child, data_ctx, node_ctx)).flatten().collect();
            return Ok(nodes);
        }
    }

    Ok(vec![])
}

fn widget_node_to_nodes(
    widget_node: &WidgetNode,
    children: &[WidgetNode],
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
) -> Result<Vec<Node>> {
    // Node kind
    let kind = match widget_node.ident.as_str() {
        "span" => {
            let text = match widget_node.text.as_ref() {
                Some(text) => text.path(data_ctx),
                None => String::new(),
            };
            Kind::Span(text)
        }
        ident => Kind::Node { ident: ident.to_string() },
    };

    let mut nodes = vec![];
    // let children = children.iter().flat_map(|n| to_nodes(n, data_ctx, node_ctx)).collect();
    for child in children {
        nodes.extend(to_nodes(child, data_ctx, node_ctx)?);
    }
    // let children = children.iter().flat_map(|n| to_nodes(n, data_ctx, node_ctx)).collect();
    let attributes = lookup_attributes(&widget_node.attributes, data_ctx);

    let id = match widget_node.node_id() {
        NodeId::Value(Value::DataBinding(path)) => match data_ctx.by_path(&path) {
            Some(data) => NodeId::Value(data.clone()),
            None => return Err(Error::IdNotFound(path)),
        },
        NodeId::Value(Value::Fragments(ref fragments)) => NodeId::Value(fragments_to_values(fragments, data_ctx)),
        id => id,
    };

    Ok(vec![Node { id, kind, attributes, children: nodes }])
}

pub(super) fn to_nodes(
    widget_node: &WidgetNode,
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
) -> Result<Vec<Node>> {
    match &widget_node.stmt {
        Statement::For { binding, data, template } => for_loop(data_ctx, node_ctx, binding, data, template),
        Statement::If { cond, children, elses } => if_statement(data_ctx, node_ctx, cond, children, elses),
        Statement::Include { path } => {
            if node_ctx.include_depth > MAX_INCLUDE_DEPTH {
                return Ok(vec![]);
            }

            let path_buffer = path.path(data_ctx);
            let widget_nodes = node_ctx.includes(path_buffer).unwrap();
            node_ctx.include_depth += 1;
            super::to_nodes(&widget_nodes, data_ctx, node_ctx)
        }
        Statement::Node { children } => widget_node_to_nodes(widget_node, children, data_ctx, node_ctx),
    }
}

// -----------------------------------------------------------------------------
//     - Lookup attributes -
//     Lookup attributes by path,
//     or get the value from a transition
// -----------------------------------------------------------------------------
fn lookup_attributes<'a>(attributes: &'a Attributes, ctx: &SubContext<'_>) -> Attributes {
    let mut attr = attributes.clone();
    for (k, v) in attributes {
        // Path
        if let Value::DataBinding(path) = v {
            if let Some(data) = ctx.by_path(path) {
                let new_value = data.clone();
                attr.set(k, new_value);
            }
        }

        // Transition
        if let Value::Transition(value, duration, easing) = v {
            if let Value::DataBinding(path) = value.as_ref() {
                if let Some(data) = ctx.by_path(path) {
                    let new_value = Value::Transition(Box::new(data.clone()), *duration, *easing);
                    attr.set(k, new_value);
                }
            }
        }
    }
    attr
}

fn fragments_to_values(fragments: &[Fragment], ctx: &SubContext<'_>) -> Value {
    let values = fragments
        .iter()
        .filter_map(|frag| match frag {
            Fragment::String(s) => Some(Value::String(s.clone())),
            Fragment::Data(path) => ctx.by_path(path).cloned(),
        })
        .collect();
    Value::List(values)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::templates::ctx::DataCtx;
    use crate::widgets::Path;

    fn ctx(val: &Value) -> DataCtx {
        DataCtx::with_value("path", val.clone())
    }

    fn attributes() -> Attributes {
        let mut attribs = Attributes::empty();
        attribs.set("thing", Value::DataBinding(Path::new("path")));
        attribs
    }

    #[test]
    fn fragments() {
        let frags = [Fragment::Data(Path::new("path"))];
        let expected_value = Value::String("value".into());
        let ctx = ctx(&expected_value);
        let ctx = SubContext::new(&ctx);
        let value = fragments_to_values(&frags, &ctx);
        assert_eq!(value, Value::List(vec![expected_value]));
    }

    #[test]
    fn attribute_lookup() {
        // Look up a value in a `DataCtx` using a path from the `Attributes`.
        let val = Value::from(1u64);
        let ctx = ctx(&val);
        let ctx = SubContext::new(&ctx);
        // These attributes has a value of `Path` with the key "thing"
        let attribs = attributes();
        // .. there the `Path` is replaced with the actual value in the contex
        // which is `1` in this case.
        let attribs = lookup_attributes(&attribs, &ctx);

        assert_eq!(attribs.get_value("thing").unwrap(), Value::from(1u64));
    }
}
