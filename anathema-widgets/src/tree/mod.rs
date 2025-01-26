use std::marker::PhantomData;
use std::ops::ControlFlow;

use anathema_state::Value as StateValue;
use anathema_store::tree::TreeView;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::{AttributeStorage, Collection, Scope, Value};

use crate::layout::display::DISPLAY;
use crate::layout::{Display, LayoutCtx, LayoutFilter};
use crate::nodes::controlflow;
use crate::nodes::loops::Iteration;
use crate::widget::WidgetTreeView;
use crate::{eval_blueprint, Element, WidgetContainer, WidgetId, WidgetKind};

// TODO: Add the option to "skip" values with an offset for `inner_each` (this is for overflow widgets)

/// Determine what kind of widgets that should be laid out:
/// Fixed or floating.
#[derive(Debug, Copy, Clone)]
pub enum WidgetPositionFilter {
    Floating,
    Fixed,
}

#[derive(Debug, Copy, Clone)]
pub enum Generator<'widget, 'bp> {
    Single(&'bp [Blueprint]),
    Loop {
        len: usize,
        binding: &'bp str,
        body: &'bp [Blueprint],
    },
    Iteration(&'bp str, &'bp [Blueprint]),
    ControlFlow(&'widget controlflow::ControlFlow<'bp>),
}

impl<'widget, 'bp> From<&'widget WidgetContainer<'bp>> for Generator<'widget, 'bp> {
    fn from(widget: &'widget WidgetContainer<'bp>) -> Self {
        match &widget.kind {
            WidgetKind::Element(_) => panic!("use Self::Single directly"),
            WidgetKind::For(for_loop) => panic!("use Self::Loop directory"),
            WidgetKind::ControlFlowContainer(_) => Self::Single(widget.children),
            WidgetKind::Component(_) => Self::Single(widget.children),
            WidgetKind::Iteration(iter) => Self::Iteration(iter.binding, widget.children),
            WidgetKind::ControlFlow(controlflow) => Self::ControlFlow(&controlflow),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Layout -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct LayoutForEach<'a, 'bp> {
    tree: WidgetTreeView<'a, 'bp>,
    scope: &'a Scope<'a, 'bp>,
    generator: Option<Generator<'a, 'bp>>,
    parent_component: Option<WidgetId>,
    filter: LayoutFilter,
    offset: usize,
}

impl<'a, 'bp> LayoutForEach<'a, 'bp> {
    pub fn new(
        tree: WidgetTreeView<'a, 'bp>,
        scope: &'a Scope<'a, 'bp>,
        filter: LayoutFilter,
        parent_component: Option<WidgetId>,
    ) -> Self {
        Self {
            tree,
            scope,
            generator: None,
            parent_component,
            filter,
            offset: 0,
        }
    }

    pub fn skip(&mut self, count: usize) -> &mut Self {
        self.offset = count;
        self
    }

    fn with_generator(
        tree: WidgetTreeView<'a, 'bp>,
        scope: &'a Scope<'a, 'bp>,
        generator: Generator<'a, 'bp>,
        filter: LayoutFilter,
        parent_component: Option<WidgetId>,
    ) -> Self {
        Self {
            tree,
            scope,
            generator: Some(generator),
            filter,
            parent_component,
            offset: 0,
        }
    }

    pub fn each<F>(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        self.inner_each(ctx, &mut f)
    }

    fn inner_each<F>(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        let mut processed_count = 0;
        for index in self.offset..self.tree.layout_len() {
            self.process(index, ctx, f)?;
            processed_count += 1;
        }

        // If there is no parent then there can be no children generated
        let Some(parent) = self.generator else { return ControlFlow::Continue(()) };

        // NOTE: Generate will never happen unless the preceeding iteration returns `Continue(())`.
        //       Therefore there is no need to worry about excessive creation of `Iter`s for loops.

        loop {
            let index = self.tree.layout_len();
            if !generate(parent, &mut self.tree, ctx, self.scope, self.parent_component) {
                break;
            }
            self.process(index, ctx, f)?;
        }

        ControlFlow::Continue(())
    }

    fn process<F>(&mut self, index: usize, ctx: &mut LayoutCtx<'_, 'bp>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let node = self
            .tree
            .layout
            .get(index)
            .expect("widgets are always generated before processed");

        let widget_count = self.tree.values.count_all_entries();

        let widget_id = node.value();
        self.tree.with_value_mut(widget_id, |path, widget, mut children| {
            match self.filter.filter(widget, ctx.attribute_storage) {
                FilterOutput::Exclude => return ControlFlow::Continue(()),
                _ => {}
            }

            let cf = match &mut widget.kind {
                WidgetKind::Element(el) => {
                    let children = LayoutForEach::with_generator(
                        children,
                        self.scope,
                        Generator::Single(&widget.children),
                        self.filter,
                        self.parent_component,
                    );
                    f(ctx, el, children)
                }
                WidgetKind::ControlFlow(controlflow) => {
                    if controlflow.has_changed(&children) {
                        let path = children.offset;
                        children.relative_remove(&[0]);
                    }
                    let generator = Generator::from(&*widget);
                    let mut children = LayoutForEach::with_generator(
                        children,
                        self.scope,
                        generator,
                        self.filter,
                        self.parent_component,
                    );
                    children.inner_each(ctx, f)
                }
                WidgetKind::For(for_loop) => {
                    let mut scope = Scope::with_collection(for_loop.binding, &for_loop.collection, self.scope);
                    let generator = Generator::Loop {
                        binding: for_loop.binding,
                        body: widget.children,
                        len: for_loop.collection.len(),
                    };
                    let mut children =
                        LayoutForEach::with_generator(children, &scope, generator, self.filter, self.parent_component);
                    children.inner_each(ctx, f)
                }
                WidgetKind::Iteration(iteration) => {
                    let loop_index = *iteration.loop_index.to_ref() as usize;
                    let scope = Scope::with_index(iteration.binding, loop_index, self.scope);
                    let mut children = LayoutForEach::with_generator(
                        children,
                        &scope,
                        Generator::from(&*widget),
                        self.filter,
                        self.parent_component,
                    );
                    children.inner_each(ctx, f)
                }
                WidgetKind::Component(component) => {
                    let parent_component = component.widget_id;
                    let state_id = component.state_id();
                    let scope = Scope::with_component(state_id, component.widget_id, self.scope);
                    let mut children = LayoutForEach::with_generator(
                        children,
                        &scope,
                        Generator::from(&*widget),
                        self.filter,
                        Some(parent_component),
                    );
                    children.inner_each(ctx, f)
                }
                WidgetKind::ControlFlowContainer(_) => {
                    let mut children = LayoutForEach::with_generator(
                        children,
                        self.scope,
                        Generator::from(&*widget),
                        self.filter,
                        self.parent_component,
                    );
                    children.inner_each(ctx, f)
                }
            };

            cf
        })
    }
}

// Generate the next available widget into the tree
// TODO: break this down into more manageable code.
//       this is a hot mess
fn generate<'bp>(
    parent: Generator<'_, 'bp>,
    tree: &mut WidgetTreeView<'_, 'bp>,
    ctx: &mut LayoutCtx<'_, 'bp>,
    scope: &Scope<'_, 'bp>,
    parent_component: Option<WidgetId>,
) -> bool {
    #[cfg(feature = "profile")]
    puffin::profile_function!();

    match parent {
        Generator::Single(blueprints) | Generator::Iteration(_, blueprints) => {
            if blueprints.is_empty() {
                return false;
            }

            let index = tree.layout_len();
            if index >= blueprints.len() {
                return false;
            }

            let mut ctx = ctx.eval_ctx(parent_component);
            // TODO: unwrap.
            // this should propagate somewhere useful
            eval_blueprint(&blueprints[index], &mut ctx, scope, tree.offset, tree).unwrap();
            true
        }
        Generator::Loop { len, .. } if len == tree.layout_len() => false,
        Generator::Loop { binding, body, .. } => {
            let loop_index = tree.layout_len();

            let transaction = tree.insert(tree.offset);
            let widget = WidgetKind::Iteration(Iteration {
                loop_index: StateValue::new(loop_index as i64),
                binding,
            });
            let widget = WidgetContainer::new(widget, body);
            // TODO: unwrap, ewww...
            transaction.commit_child(widget).unwrap();

            true
        }
        Generator::ControlFlow(controlflow) => {
            let child_count = tree.layout_len();
            assert_eq!(child_count.saturating_sub(1), 0, "too many branches have been created");

            // TODO: this could probably be replaced with the functionality in
            // ControlFlow::has_changed

            let should_create = {
                if child_count == 0 {
                    true
                } else {
                    let node_id = tree.layout[0].value();
                    let (path, widget) = tree
                        .values
                        .get(node_id)
                        .expect("because the node exists, the value exist");

                    let is_true = match &widget.kind {
                        WidgetKind::ControlFlowContainer(id) => controlflow.elses[*id as usize].is_true(),
                        _ => unreachable!("the child of `ControlFlow` can only be `If` or `Else`"),
                    };

                    // The condition no longer holds so the branch has to be trimmed
                    if is_true {
                        return false;
                    }

                    is_true
                }
            };

            if !should_create {
                return false;
            }

            let thing = controlflow
                .elses
                .iter()
                .enumerate()
                .filter_map(|(id, node)| {
                    let cond = node.cond.as_ref().and_then(|cond| cond.as_bool()).unwrap_or(true);
                    match cond {
                        true => Some((id, node.body)),
                        false => None,
                    }
                })
                .next();

            match thing {
                Some((id, body)) => {
                    let kind = WidgetKind::ControlFlowContainer(id as u16);
                    let widget = WidgetContainer::new(kind, body);
                    let transaction = tree.insert(tree.offset);
                    transaction.commit_child(widget);
                }
                None => return false,
            }

            true
        }
    }
}

pub enum FilterOutput<T, F> {
    Include(T, F),
    Exclude,
    Continue,
}

pub trait Filter<'bp>: Copy {
    type Output;

    fn filter<'a>(
        &self,
        widget: &'a mut WidgetContainer<'bp>,
        attribute_storage: &AttributeStorage<'_>,
    ) -> FilterOutput<&'a mut Self::Output, Self>;
}

// -----------------------------------------------------------------------------
//   - Position / Paint -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ForEach<'a, 'bp, Fltr> {
    tree: WidgetTreeView<'a, 'bp>,
    attribute_storage: &'a AttributeStorage<'bp>,
    filter: Fltr,
}

impl<'a, 'bp, Fltr: Filter<'bp>> ForEach<'a, 'bp, Fltr> {
    pub fn new(tree: WidgetTreeView<'a, 'bp>, attribute_storage: &'a AttributeStorage<'bp>, filter: Fltr) -> Self {
        Self {
            tree,
            attribute_storage,
            filter,
        }
    }

    pub fn each<F>(&mut self, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&mut Fltr::Output, ForEach<'_, 'bp, Fltr>) -> ControlFlow<()>,
    {
        self.inner_each(None, &mut f)
    }

    fn inner_each<F>(&mut self, parent: Option<&WidgetContainer<'bp>>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut Fltr::Output, ForEach<'_, 'bp, Fltr>) -> ControlFlow<()>,
    {
        for index in 0..self.tree.layout_len() {
            self.process(index, f);
        }

        ControlFlow::Continue(())
    }

    fn process<F>(&mut self, index: usize, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut Fltr::Output, ForEach<'_, 'bp, Fltr>) -> ControlFlow<()>,
    {
        let Some(node) = self.tree.layout.get(index) else { panic!() };
        self.tree.with_value_mut(node.value(), |path, widget, children| {
            match self.filter.filter(widget, self.attribute_storage) {
                FilterOutput::Include(el, filter) => f(el, ForEach::new(children, self.attribute_storage, filter)),
                FilterOutput::Exclude => ControlFlow::Break(()),
                FilterOutput::Continue => {
                    ForEach::new(children, self.attribute_storage, self.filter).inner_each(Some(widget), f)
                }
            }
        })
    }
}
