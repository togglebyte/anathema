use std::ops::ControlFlow;

use anathema_state::Value;
use anathema_store::tree::TreeView;
use anathema_templates::blueprints::Blueprint;

use super::{WidgetId, WidgetTreeView};
use crate::nodes::loops::Iteration;
use crate::{eval_blueprint, Element, EvalContext, WidgetContainer, WidgetKind};

// TODO: Add the option to "skip" values with an offset for `inner_each` (this is for overflow widgets)

#[derive(Debug, Copy, Clone)]
pub enum Generator<'bp> {
    Single(&'bp [Blueprint]),
    Loop(usize, &'bp str, &'bp [Blueprint]),
    Parent(()),
}

impl<'bp> From<&WidgetContainer<'bp>> for Generator<'bp> {
    fn from(widget: &WidgetContainer<'bp>) -> Self {
        match &widget.kind {
            WidgetKind::Element(_) | WidgetKind::Component(_) | WidgetKind::Iteration(_) => Self::Single(widget.children),
            WidgetKind::For(for_loop) => Self::Loop(for_loop.collection.count(), for_loop.binding, widget.children),
            _ => Self::Parent(()),
        }
    }
}

#[derive(Debug)]
pub struct LayoutForEach<'a, 'bp> {
    tree: WidgetTreeView<'a, 'bp>,
    parent: Option<Generator<'bp>>,
}

impl<'a, 'bp> LayoutForEach<'a, 'bp> {
    pub fn new(tree: WidgetTreeView<'a, 'bp>, parent: Option<Generator<'bp>>) -> Self {
        Self { tree, parent }
    }

    pub fn each<F>(&mut self, ctx: &mut EvalContext<'_, '_, 'bp>, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&mut EvalContext<'_, '_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        self.inner_each(ctx, &mut f)
    }

    fn inner_each<F>(&mut self, ctx: &mut EvalContext<'_, '_, 'bp>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut EvalContext<'_, '_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        for index in 0..self.tree.layout_len() {
            self.process(index, ctx, f);
        }

        // If there is no parent then there can be no children generated
        let Some(parent) = self.parent else { return ControlFlow::Continue(()) };

        // NOTE: Generate will never happen unless the preceeding iteration returns `Continue(())`.
        //       Therefore there is no need to worry about excessive creation of `Iter`s for loops.

        loop {
            let index = self.tree.layout_len();
            if !generate(parent, &mut self.tree, ctx) {
                break;
            }
            self.process(index, ctx, f);
        }

        ControlFlow::Continue(())
    }

    fn process<F>(&mut self, index: usize, ctx: &mut EvalContext<'_, '_, 'bp>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut EvalContext<'_, '_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        let Some(node) = self.tree.layout.get(index) else { panic!() };

        self.tree.with_value_mut(node.value(), |path, widget, children| {
            widget.push_scope(ctx);
            let generator = Generator::from(&*widget);
            let cf = match &mut widget.kind {
                WidgetKind::Element(el) => {
                    let children = LayoutForEach::new(children, Some(generator));
                    f(ctx, el, children)
                }
                _ => {
                    let mut children = LayoutForEach::new(children, Some(generator));
                    children.inner_each(ctx, f)
                } // WidgetKind::For(_) => todo!(),
                  // WidgetKind::Iteration(_) => todo!(),
                  // WidgetKind::ControlFlow(_) => todo!(),
                  // WidgetKind::If(_) => todo!(),
                  // WidgetKind::Else(_) => todo!(),
            };
            widget.pop_scope(ctx);
            cf
        })
    }
}

// Generate the next available widget into the tree
fn generate<'bp>(
    parent: Generator<'bp>,
    tree: &mut WidgetTreeView<'_, 'bp>,
    ctx: &mut EvalContext<'_, '_, 'bp>,
) -> bool {
    match parent {
        Generator::Single(blueprints) => {
            if blueprints.is_empty() {
                return false;
            }

            let index = tree.layout_len();
            if index >= blueprints.len() {
                return false;
            }

            eval_blueprint(&blueprints[index], ctx, tree.offset, tree);
            true
        }
        Generator::Loop(count, _, _) if count == tree.layout_len() => false,
        Generator::Loop(count, binding, body) => {
            let loop_index = tree.layout_len();
            let widget = WidgetKind::Iteration(Iteration {
                loop_index: Value::new(loop_index as i64),
                binding,
            });
            let widget = WidgetContainer::new(widget, body);
            let iter_id = tree.insert(tree.offset).commit_child(widget).unwrap();
            // .ok_or(Error::TreeTransactionFailed)?;

            false
        }
        // Generator::Iteration(())  => {
        // }
        Generator::Parent(()) => {
            // Generate a new iter but only if the last iter has generated all the children...
            // How is this even going to get done?

            todo!()
        } // WidgetKind::Iteration(_) => todo!(),
          // WidgetKind::ControlFlow(_) => todo!(),
          // WidgetKind::If(_) => todo!(),
          // WidgetKind::Else(_) => todo!(),
    }
}

#[derive(Debug)]
pub struct ForEach<'a, 'bp> {
    tree: WidgetTreeView<'a, 'bp>,
}

impl<'a, 'bp> ForEach<'a, 'bp> {
    pub fn new(tree: WidgetTreeView<'a, 'bp>) -> Self {
        Self { tree }
    }

    pub fn each<F>(&mut self, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&mut Element<'bp>, ForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        self.inner_each(None, &mut f)
    }

    fn inner_each<F>(&mut self, parent: Option<&WidgetContainer<'bp>>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut Element<'bp>, ForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        for index in 0..self.tree.layout_len() {
            self.process(index, f);
        }

        ControlFlow::Continue(())
    }

    fn process<F>(&mut self, index: usize, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut Element<'bp>, ForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        let Some(node) = self.tree.layout.get(index) else { panic!() };
        self.tree.with_value_mut(node.value(), |path, widget, children| {
            let mut children = ForEach::new(children);
            match &mut widget.kind {
                WidgetKind::Element(el) => f(el, children),
                _ => children.inner_each(Some(widget), f),
                // WidgetKind::Component(_) => children.inner_each(Some(widget), f),
                // WidgetKind::For(_) => todo!(),
                // WidgetKind::Iteration(_) => todo!(),
                // WidgetKind::ControlFlow(_) => todo!(),
                // WidgetKind::If(_) => todo!(),
                // WidgetKind::Else(_) => todo!(),
            }
        })
    }
}
