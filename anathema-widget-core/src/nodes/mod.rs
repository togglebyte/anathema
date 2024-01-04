use std::fmt;
use std::iter::once;
use std::ops::ControlFlow;

use anathema_values::{
    Change, Context, Deferred, Immediate, NextNodeId, NodeId, ScopeStorage, Value, ValueRef,
};

pub(crate) use self::controlflow::IfElse;
pub(crate) use self::loops::LoopNode;
use self::query::Query;
use crate::error::Result;
use crate::expressions::{Collection, Expression, ViewState};
use crate::views::{AnyView, Views};
use crate::{Event, WidgetContainer};

mod controlflow;
mod loops;
mod query;
pub mod visitor;

pub fn make_it_so(expressions: &[crate::expressions::Expression]) -> Nodes<'_> {
    Nodes::new(expressions, 0.into())
}

// TODO: good grief rename this function!
fn c_and_b<'expr, F>(
    nodes: &mut Nodes<'expr>,
    context: &Context<'_, 'expr>,
    f: &mut F,
) -> Result<ControlFlow<(), ()>>
where
    F: FnMut(&mut WidgetContainer<'expr>, &mut Nodes<'expr>, &Context<'_, 'expr>) -> Result<()>,
{
    while let Ok(res) = nodes.next(context, f) {
        match res {
            ControlFlow::Continue(()) => continue,
            ControlFlow::Break(()) => break,
        }
    }

    Ok(ControlFlow::Continue(()))
}

#[derive(Debug)]
pub struct Node<'e> {
    pub node_id: NodeId,
    pub kind: NodeKind<'e>,
    pub(crate) scope: ScopeStorage<'e>,
}

impl<'e> Node<'e> {
    pub fn next<F>(&mut self, context: &Context<'_, 'e>, f: &mut F) -> Result<ControlFlow<(), ()>>
    where
        F: FnMut(&mut WidgetContainer<'e>, &mut Nodes<'e>, &Context<'_, 'e>) -> Result<()>,
    {
        match &mut self.kind {
            NodeKind::Single(Single {
                widget, children, ..
            }) => {
                f(widget, children, context)?;
                Ok(ControlFlow::Continue(()))
            }
            NodeKind::Loop(loop_state) => loop_state.next(&mut self.scope, context, f),
            NodeKind::ControlFlow(if_else) => {
                let Some(body) = if_else.body_mut() else {
                    return Ok(ControlFlow::Break(()));
                };
                c_and_b(body, context, f)
            }
            NodeKind::View(View {
                nodes, state, view, ..
            }) => match state {
                ViewState::Dynamic(state) => {
                    let scope = context.new_scope(&self.scope);
                    let context = context.with_state(*state);
                    let context = context.with_scope(&scope);
                    let context = context.with_state(view.get_any_state());
                    c_and_b(nodes, &context, f)
                }
                ViewState::External { expr, .. } => {
                    let mut resolver = Immediate::new(context.lookup(), &self.node_id);

                    match expr.eval(&mut resolver) {
                        ValueRef::Map(state) => {
                            let scope = context.new_scope(&self.scope);
                            let context = context.with_state(state);
                            let context = context.with_scope(&scope);
                            let context = context.with_state(view.get_any_state());
                            c_and_b(nodes, &context, f)
                        }
                        _ => {
                            let scope = context.new_scope(&self.scope);
                            let context = context.with_scope(&scope);
                            c_and_b(nodes, &context, f)
                        }
                    }
                }
                ViewState::Map(map) => {
                    let mut resolver = Deferred::new(context.lookup());
                    for (k, expr) in map.0 {
                        match expr.eval(&mut resolver) {
                            ValueRef::Deferred => self.scope.deferred(k, expr),
                            val => self.scope.value(k, val),
                        }
                    }

                    let scope = context.new_scope(&self.scope);
                    let context = context.with_scope(&scope);
                    let context = context.with_state(view.get_any_state());
                    c_and_b(nodes, &context, f)
                }
                ViewState::Internal => {
                    let scope = context.new_scope(&self.scope);
                    let context = context.with_state(view.get_any_state());
                    let context = context.with_scope(&scope);
                    c_and_b(nodes, &context, f)
                }
            },
        }
    }

    fn reset_cache(&mut self) {
        match &mut self.kind {
            NodeKind::Single(Single { children, .. }) => children.reset_cache(),
            NodeKind::Loop(loop_state) => loop_state.reset_cache(),
            NodeKind::ControlFlow(if_else) => if_else.reset_cache(),
            NodeKind::View(View { nodes, .. }) => nodes.reset_cache(),
        }
    }

    // Update this node.
    // This means that the update was specifically for this node,
    // and not one of its children
    fn update(&mut self, change: &Change, context: &Context<'_, '_>) {
        let scope = context.new_scope(&self.scope);
        let context = context.with_scope(&scope);

        match &mut self.kind {
            NodeKind::Single(Single { widget, .. }) => widget.update(&context, &self.node_id),
            NodeKind::Loop(loop_node) => {
                // if the collection is bound to a state
                // we need to resub to the state

                if let Collection::State { expr, .. } = loop_node.collection {
                    let mut immediate = Immediate::new(context.lookup(), &self.node_id);
                    let _ = expr.eval(&mut immediate);
                }

                match change {
                    Change::InsertIndex(index) => loop_node.insert(*index),
                    Change::RemoveIndex(index) => loop_node.remove(*index),
                    Change::Push => loop_node.push(),
                    _ => (),
                }
            }
            NodeKind::View(View {
                tabindex, state: _, ..
            }) => {
                tabindex.resolve(&context, &self.node_id);
                Views::update(&self.node_id, tabindex.value());
            }
            // NOTE: the control flow has no immediate information
            // that needs updating, so an update should never end with the
            // control flow node
            NodeKind::ControlFlow(_) => {}
        }
    }
}

#[derive(Debug)]
pub struct Single<'e> {
    pub(crate) widget: WidgetContainer<'e>,
    pub(crate) children: Nodes<'e>,
    pub(crate) ident: &'e str,
}

pub struct View<'e> {
    pub(crate) view: Box<dyn AnyView>,
    pub(crate) nodes: Nodes<'e>,
    pub(crate) state: ViewState<'e>,
    pub tabindex: Value<u32>,
}

impl fmt::Debug for View<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("View")
            .field("nodes", &self.nodes)
            .field("state", &self.state)
            .finish()
    }
}

impl View<'_> {
    pub fn on_event(&mut self, event: Event) {
        self.view.on_any_event(event, &mut self.nodes);
    }

    pub fn tick(&mut self) {
        self.view.tick_any();
    }

    pub fn focus(&mut self) {
        self.view.focus_any();
    }

    pub fn blur(&mut self) {
        self.view.blur_any();
    }
}

#[derive(Debug)]
pub enum NodeKind<'e> {
    Single(Single<'e>),
    Loop(LoopNode<'e>),
    ControlFlow(IfElse<'e>),
    View(View<'e>),
}

#[derive(Debug)]
pub struct Nodes<'expr> {
    expressions: &'expr [Expression],
    inner: Vec<Node<'expr>>,
    expr_index: usize,
    root_id: NodeId,
    next_node_id: NextNodeId,
    cache_index: usize,
}

impl<'expr> Nodes<'expr> {
    pub fn with_view<F>(&mut self, node_id: &NodeId, mut f: F)
    where
        F: FnMut(&mut View<'_>),
    {
        if let Some(Node {
            kind: NodeKind::View(view),
            ..
        }) = self.query().get(node_id)
        {
            f(view);
        }
    }

    fn new_node(&mut self, context: &Context<'_, 'expr>) -> Option<Result<()>> {
        let expr = self.expressions.get(self.expr_index)?;
        self.expr_index += 1;
        match expr.eval(context, self.next_node_id.next(&self.root_id)) {
            Ok(node) => self.inner.push(node),
            Err(e) => return Some(Err(e)),
        };
        Some(Ok(()))
    }

    pub(crate) fn next<F>(
        &mut self,
        context: &Context<'_, 'expr>,
        f: &mut F,
    ) -> Result<ControlFlow<(), ()>>
    where
        F: FnMut(&mut WidgetContainer<'expr>, &mut Nodes<'expr>, &Context<'_, 'expr>) -> Result<()>,
    {
        match self.inner.get_mut(self.cache_index) {
            Some(n) => {
                self.cache_index += 1;
                n.next(context, f)
            }
            None => {
                let res = self.new_node(context);
                match res {
                    None => Ok(ControlFlow::Break(())),
                    Some(Err(e)) => Err(e),
                    Some(Ok(())) => self.next(context, f),
                }
            }
        }
    }

    pub fn for_each<F>(&mut self, context: &Context<'_, 'expr>, mut f: F) -> Result<()>
    where
        F: FnMut(&mut WidgetContainer<'expr>, &mut Nodes<'expr>, &Context<'_, 'expr>) -> Result<()>,
    {
        #[allow(clippy::while_let_loop)]
        loop {
            match self.next(context, &mut f)? {
                ControlFlow::Continue(()) => continue,
                ControlFlow::Break(()) => break,
            }
        }
        Ok(())
    }

    /// Update and apply the change to the specific node.
    /// This is currently done by the runtime
    #[doc(hidden)]
    pub fn update(&mut self, node_id: &[usize], change: &Change, context: &Context<'_, '_>) {
        update(&mut self.inner, node_id, change, context);
    }

    pub(crate) fn new(expressions: &'expr [Expression], root_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            expr_index: 0,
            next_node_id: NextNodeId::new(root_id.last()),
            root_id,
            cache_index: 0,
        }
    }

    /// Count the number of widgets in the node tree
    pub fn count(&self) -> usize {
        count_widgets(self.inner.iter())
    }

    /// Reset the widget cache.
    /// This should be done per frame
    #[doc(hidden)]
    pub fn reset_cache(&mut self) {
        self.cache_index = 0;
        for node in &mut self.inner {
            node.reset_cache();
        }
    }

    /// Query the node tree.
    /// See [`Query`] for more information
    pub fn query(&mut self) -> Query<'_, 'expr, ()> {
        Query {
            nodes: self,
            filter: (),
        }
    }

    /// A mutable iterator over [`WidgetContainer`]s and their children
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer<'expr>, &mut Nodes<'expr>)> + '_ {
        self.inner.iter_mut().flat_map(
            |node| -> Box<dyn Iterator<Item = (&mut WidgetContainer<'expr>, &mut Nodes<'expr>)>> {
                match &mut node.kind {
                    NodeKind::Single(Single {
                        widget, children, ..
                    }) => Box::new(once((widget, children))),
                    NodeKind::Loop(loop_state) => Box::new(loop_state.iter_mut()),
                    NodeKind::ControlFlow(control_flow) => Box::new(control_flow.iter_mut()),
                    NodeKind::View(View { nodes, .. }) => Box::new(nodes.iter_mut()),
                }
            },
        )
    }

    /// First mutable [`WidgetContainer`] and its children
    pub fn first_mut(&mut self) -> Option<(&mut WidgetContainer<'expr>, &mut Nodes<'expr>)> {
        self.iter_mut().next()
    }
}

fn count_widgets<'a>(nodes: impl Iterator<Item = &'a Node<'a>>) -> usize {
    nodes
        .map(|node| match &node.kind {
            NodeKind::Single(Single { children, .. }) => 1 + children.count(),
            NodeKind::Loop(loop_state) => loop_state.count(),
            NodeKind::ControlFlow(if_else) => if_else.count(),
            NodeKind::View(View { nodes, .. }) => nodes.count(),
        })
        .sum()
}

// Apply change / update to relevant nodes
fn update(nodes: &mut [Node<'_>], node_id: &[usize], change: &Change, context: &Context<'_, '_>) {
    for node in nodes {
        if !node.node_id.contains(node_id) {
            continue;
        }
        // Found the node to update
        if node.node_id.eq(node_id) {
            return node.update(change, context);
        }

        let scope = context.new_scope(&node.scope);
        let context = context.with_scope(&scope);

        match &mut node.kind {
            NodeKind::Single(Single { children, .. }) => {
                return children.update(node_id, change, &context)
            }
            NodeKind::Loop(loop_node) => {
                return loop_node.update(node_id, change, &context);
            }
            NodeKind::ControlFlow(if_else) => return if_else.update(node_id, change, &context),
            NodeKind::View(view) => {
                // TODO: make this into its own function.
                //       also note: it's strange to return a unit here for a bogus state
                //       it's also strange to return the state of the view as it's also
                //       set on the match arm of Internal

                // Don't return a state here
                let state = match view.state {
                    ViewState::Dynamic(state) => state,
                    ViewState::External { expr, .. } => {
                        let mut resolver = Immediate::new(context.lookup(), &node.node_id);
                        match expr.eval(&mut resolver) {
                            ValueRef::Map(state) => state,
                            _ => &(),
                        }
                    }
                    ViewState::Map(_map) => &(),
                    ViewState::Internal => view.view.get_any_state(),
                };

                let context = context.with_state(state);
                // TODO: this is silly. see above TODO
                // let context = context.with_state(view.view.get_any_state());
                // let scope = context.new_scope(&node.scope);
                // let context = context.with_scope(&scope);

                return view.nodes.update(node_id, change, &context);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_render::Size;
    use anathema_values::testing::{ident, list};
    use anathema_values::ValueExpr;

    use crate::testing::expressions::{expression, for_expression, if_expression};
    use crate::testing::nodes::*;

    #[test]
    fn generate_a_single_widget() {
        let test = expression("test", None, [], []).test();
        let mut node = test.eval().unwrap();
        let (widget, _nodes) = node.single();
        assert_eq!(widget.kind(), "text");
    }

    #[test]
    fn for_loop() {
        let string = "hello".into();
        let body = expression("test", Some(string), [], []);
        let exprs = vec![for_expression("item", list([1, 2, 3]), [body])];
        let mut runtime = test_runtime(&exprs);
        let size = runtime.layout().unwrap();
        assert_eq!(size, Size::new(5, 3));
        assert_eq!(runtime.nodes.count(), 3);
    }

    #[test]
    fn for_loop_from_state() {
        let string = ValueExpr::Ident("item".into());
        let body = expression("test", Some(string), [], []);
        let exprs = vec![for_expression("item", ident("generic_list"), [body])];
        let mut runtime = test_runtime(&exprs);
        let size = runtime.layout().unwrap();
        assert_eq!(size, Size::new(1, 3));
        assert_eq!(runtime.nodes.count(), 3);
    }

    fn test_if_else(is_true: bool, else_cond: Option<bool>, expected: &str) {
        let is_true = is_true.into();
        let is_else = else_cond.map(|val| val.into());

        let else_if_expr = vec![expression("test", Some("else branch".into()), [], [])];
        let if_expr = vec![expression("test", Some("true".into()), [], [])];
        let else_expr = vec![expression(
            "test",
            Some("else branch without condition".into()),
            [],
            [],
        )];

        let exprs = vec![if_expression(
            (is_true, if_expr),
            vec![(is_else, else_if_expr), (None, else_expr)],
        )];
        let mut runtime = test_runtime(&exprs);
        let _ = runtime.layout().unwrap();
        let (node, _) = runtime.nodes.first_mut().unwrap();
        let widget = node.to_ref::<TestWidget>();

        assert_eq!(widget.0.value_ref().unwrap(), expected);
    }

    #[test]
    fn if_else() {
        test_if_else(true, None, "true");
        test_if_else(false, Some(true), "else branch");
        test_if_else(false, None, "else branch");
        test_if_else(false, Some(false), "else branch without condition");
    }
}
