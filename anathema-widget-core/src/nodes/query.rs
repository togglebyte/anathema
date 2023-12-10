use anathema_values::{NodeId, ValueExpr};

use super::{LoopNode, Node, Single, View};
use crate::nodes::NodeKind;
use crate::Nodes;

pub struct Query<'nodes, 'expr, F> {
    pub(super) nodes: &'nodes mut Nodes<'expr>,
    pub(super) filter: F,
}

impl<'nodes, 'expr: 'nodes, F: Filter> Query<'nodes, 'expr, F> {
    pub fn by_attrib(self, key: &str, value: impl Into<ValueExpr>) -> Query<'nodes, 'expr, impl Filter> {
        let filter = ByAttribute(key.into(), value.into());

        Query {
            nodes: self.nodes,
            filter: self.filter.chain(filter),
        }
    }

    pub fn by_tag(self, tag: &str) -> Query<'nodes, 'expr, impl Filter> {
        let filter = ByTag(tag.into());

        Query {
            nodes: self.nodes,
            filter: self.filter.chain(filter),
        }
    }

    pub fn filter<Fun>(self, f: Fun) -> Query<'nodes, 'expr, impl Filter>
    where
        Fun: Fn(&Node) -> bool,
    {
        let filter = FilterFn(f);
        Query {
            nodes: self.nodes,
            filter: self.filter.chain(filter),
        }
    }

    fn remove_nodes(filter: &F, nodes: &mut Nodes<'expr>) {
        let mut indices = vec![];

        for (index, node) in nodes.inner.iter_mut().enumerate() {
            if filter.filter(&node) {
                indices.push(index);
            }

            match &mut node.kind {
                NodeKind::Single(Single { children, .. }) => Self::remove_nodes(filter, children),
                NodeKind::View(View { nodes, .. }) => Self::remove_nodes(filter, nodes),
                NodeKind::Loop(LoopNode { iterations, .. }) => {
                    for iteration in iterations {
                        Self::remove_nodes(filter, &mut iteration.body)
                    }
                }
                NodeKind::ControlFlow(if_else) => {
                    if let Some(body) = if_else.body_mut() {
                        Self::remove_nodes(filter, body);
                    }
                }
            }
        }

        indices.reverse();
        indices
            .into_iter()
            .for_each(|index| drop(nodes.inner.remove(index)));
    }

    fn for_each_nodes<Fun>(filter: &F, nodes: &mut Nodes<'expr>, fun: &mut Fun)
    where
        Fun: FnMut(&mut Node),
    {
        for node in &mut nodes.inner {
            if filter.filter(&node) {
                fun(node);
            }

            match &mut node.kind {
                NodeKind::Single(Single { children, .. }) => {
                    Self::for_each_nodes(filter, children, fun)
                }
                NodeKind::View(View { nodes, .. }) => Self::for_each_nodes(filter, nodes, fun),
                NodeKind::Loop(LoopNode { iterations, .. }) => {
                    for iteration in iterations {
                        Self::for_each_nodes(filter, &mut iteration.body, fun);
                    }
                }
                NodeKind::ControlFlow(if_else) => {
                    if let Some(body) = if_else.body_mut() {
                        Self::for_each_nodes(filter, body, fun);
                    }
                }
            }
        }
    }

    pub fn remove(self) {
        Self::remove_nodes(&self.filter, self.nodes);
    }

    pub fn for_each<Fun>(self, mut fun: Fun)
    where
        Fun: FnMut(&mut Node),
    {
        Self::for_each_nodes(&self.filter, self.nodes, &mut fun);
    }

    fn get_node<'a>(node_id: &NodeId, nodes: &'a mut Nodes<'expr>) -> Option<&'a mut Node<'expr>> {
        for node in &mut nodes.inner {
            // Found the node
            if node.node_id.eq(node_id) {
                return Some(node);
            }

            if !node.node_id.contains(&node_id.0) {
                continue
            }

            return match &mut node.kind {
                NodeKind::Single(Single { children, .. }) => Self::get_node(node_id, children),
                NodeKind::View(View { nodes, .. }) => Self::get_node(node_id, nodes),
                NodeKind::ControlFlow(if_else) => {
                    let nodes = if_else.body_mut()?;
                    Self::get_node(node_id, nodes)
                }
                NodeKind::Loop(LoopNode { iterations, .. }) => {
                    for iteration in iterations {
                        if let node @ Some(_) = Self::get_node(node_id, &mut iteration.body) {
                            return node;
                        }
                    }
                    None
                }
            }
        }

        None
    }

    pub fn get(&mut self, node_id: &NodeId) -> Option<&mut Node<'expr>> {
        Self::get_node(node_id, &mut self.nodes)
    }
}

pub trait Filter {
    fn filter(&self, node: &Node) -> bool {
        true
    }

    fn chain<F: Filter>(self, other: F) -> FilterChain<Self, F>
    where
        Self: Sized,
    {
        FilterChain {
            lhs: self,
            rhs: other,
        }
    }
}

struct FilterChain<A, B> {
    lhs: A,
    rhs: B,
}

impl<A, B> Filter for FilterChain<A, B>
where
    A: Filter,
    B: Filter,
{
    fn filter(&self, node: &Node) -> bool {
        if self.lhs.filter(node) {
            self.rhs.filter(node)
        } else {
            false
        }
    }
}

impl Filter for () {}

struct ByAttribute(String, ValueExpr);

// TODO: attributes are not resolved at this point.
//       Alternatively we can resolve all attributes upon creation, 
//       and thus having a cached value for lookups
impl Filter for ByAttribute {
    fn filter(&self, node: &Node) -> bool {
        match &node.kind {
            NodeKind::Single(Single { widget, .. }) => widget
                .attributes
                .get(&self.0)
                .map(|a| a.eq(&self.1))
                .unwrap_or(false),
            _ => false,
        }
    }
}

struct ByTag(String);

impl Filter for ByTag {
    fn filter(&self, node: &Node) -> bool {
        match node.kind {
            NodeKind::Single(Single { ident, .. }) => {
                ident == self.0
            }
            _ => false,
        }
    }
}

struct FilterFn<F>(F);

impl<F> Filter for FilterFn<F>
where
    F: Fn(&Node) -> bool,
{
    fn filter(&self, node: &Node) -> bool {
        (self.0)(node)
    }
}
