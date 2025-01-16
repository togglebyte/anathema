use std::marker::PhantomData;
use std::ops::ControlFlow;

use anathema_state::Value as StateValue;
use anathema_store::tree::TreeView;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::{Collection, Scope, Value};

use crate::layout::LayoutCtx;
use crate::nodes::controlflow;
use crate::nodes::loops::Iteration;
use crate::widget::WidgetTreeView;
use crate::{eval_blueprint, Element, WidgetContainer, WidgetKind};

// TODO: Add the option to "skip" values with an offset for `inner_each` (this is for overflow widgets)

#[derive(Debug, Copy, Clone)]
pub enum Generator<'widget, 'bp> {
    Single(&'bp [Blueprint]),
    Loop {
        collection: &'widget Collection<'bp>,
        binding: &'bp str,
        body: &'bp [Blueprint],
    },
    Iteration(&'bp str, &'bp [Blueprint]),
    ControlFlow(&'widget controlflow::ControlFlow<'bp>),
    Parent(()),
}

impl<'widget, 'bp> From<&'widget WidgetContainer<'bp>> for Generator<'widget, 'bp> {
    fn from(widget: &'widget WidgetContainer<'bp>) -> Self {
        match &widget.kind {
            WidgetKind::Element(_) | WidgetKind::Component(_) | WidgetKind::ControlFlowContainer(_) => {
                Self::Single(widget.children)
            }
            WidgetKind::Iteration(iter) => Self::Iteration(iter.binding, widget.children),
            WidgetKind::For(for_loop) => Self::Loop {
                collection: &for_loop.collection,
                binding: for_loop.binding,
                body: widget.children,
            },
            WidgetKind::ControlFlow(controlflow) => Self::ControlFlow(&controlflow),
            _ => Self::Parent(()),
        }
    }
}

#[derive(Debug)]
pub struct LayoutForEach<'a, 'bp> {
    tree: WidgetTreeView<'a, 'bp>,
    generator: Option<Generator<'a, 'bp>>,
}

impl<'a, 'bp> LayoutForEach<'a, 'bp> {
    pub fn new(tree: WidgetTreeView<'a, 'bp>) -> Self {
        Self { tree, generator: None }
    }

    fn with_generator(tree: WidgetTreeView<'a, 'bp>, generator: Generator<'a, 'bp>) -> Self {
        Self {
            tree,
            generator: Some(generator),
        }
    }

    pub fn each<F>(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        let scope = Scope::empty();
        self.inner_each(ctx, &scope, &mut f)
    }

    fn inner_each<F>(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, scope: &Scope<'_, 'bp>, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        for index in 0..self.tree.layout_len() {
            self.process(index, ctx, scope, f)?;
        }

        // If there is no parent then there can be no children generated
        let Some(parent) = self.generator else { return ControlFlow::Continue(()) };

        // NOTE: Generate will never happen unless the preceeding iteration returns `Continue(())`.
        //       Therefore there is no need to worry about excessive creation of `Iter`s for loops.

        loop {
            let index = self.tree.layout_len();
            if !generate(parent, &mut self.tree, ctx, scope) {
                break;
            }
            self.process(index, ctx, scope, f)?;
        }

        ControlFlow::Continue(())
    }

    fn process<F>(
        &mut self,
        index: usize,
        ctx: &mut LayoutCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        f: &mut F,
    ) -> ControlFlow<()>
    where
        F: FnMut(&mut LayoutCtx<'_, 'bp>, &mut Element<'bp>, LayoutForEach<'_, 'bp>) -> ControlFlow<()>,
    {
        let node = self
            .tree
            .layout
            .get(index)
            .expect("widgets are always generated before processed");

        let widget_count = self.tree.values.count_all_entries();

        let widget_id = node.value();
        self.tree.with_value_mut(widget_id, |path, widget, mut children| {
            widget.resolve_pending_values(ctx, widget_id);

            let cf = match &mut widget.kind {
                WidgetKind::Element(el) => {
                    let children = LayoutForEach::with_generator(children, Generator::Single(&widget.children));
                    f(ctx, el, children)
                }
                WidgetKind::ControlFlow(controlflow) => {
                    if controlflow.has_changed(&children) {
                        let path = children.offset;
                        children.relative_remove(&[0]);
                    }
                    let generator = Generator::from(&*widget);
                    let mut children = LayoutForEach::with_generator(children, generator);
                    children.inner_each(ctx, &scope, f)
                }
                WidgetKind::For(for_loop) => {
                    let collection = &for_loop.collection;
                    let collection = panic!("the loop need a value expr instead");
                    let mut scope = Scope::with_collection(for_loop.binding, collection, scope);

                    let mut children = LayoutForEach::with_generator(children, Generator::from(&*widget));
                    children.inner_each(ctx, &scope, f)
                }
                WidgetKind::Iteration(iteration) => {
                    let loop_index = *iteration.loop_index.to_ref() as usize;
                    let mut scope = Scope::with_index(iteration.binding, loop_index, scope);
                    let mut children = LayoutForEach::with_generator(children, Generator::from(&*widget));
                    children.inner_each(ctx, &scope, f)
                }
                WidgetKind::Component(component) => {
                    let state_id = component.state_id();
                    let scope = Scope::with_component(state_id, component.widget_id, scope);
                    let mut children = LayoutForEach::with_generator(children, Generator::from(&*widget));
                    children.inner_each(ctx, &scope, f)
                }
                WidgetKind::ControlFlowContainer(_) => {
                    let mut children = LayoutForEach::with_generator(children, Generator::from(&*widget));
                    children.inner_each(ctx, &scope, f)
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
) -> bool {
    match parent {
        Generator::Single(blueprints) | Generator::Iteration(_, blueprints) => {
            if blueprints.is_empty() {
                return false;
            }

            let index = tree.layout_len();
            if index >= blueprints.len() {
                return false;
            }

            let mut ctx = ctx.eval_ctx();
            eval_blueprint(&blueprints[index], &mut ctx, scope, tree.offset, tree);
            true
        }
        Generator::Loop { collection, .. } if collection.len() == tree.layout_len() => false,
        Generator::Loop {
            collection,
            binding,
            body,
        } => {
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

            // What if we don't store the condition, but rather a container and somehow identify
            // the container against an id instead? if = 0, elses = N
            //
            // During an update the tree can be cleared and the container can be regenerated.
            //
            // Should probably move the generation function to it's own function for the control
            // flow since it's a bit messy

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

            // if controlflow.if_node.cond.load_bool() {
            //     let kind = WidgetKind::ControlFlowContainer(0);
            //     let widget = WidgetContainer::new(kind, controlflow.if_node.body);
            //     let transaction = tree.insert(tree.offset);
            //     transaction.commit_child(widget);
            // } else {
            let thing = controlflow
                .elses
                .iter()
                .enumerate()
                .filter_map(|(id, node)| {
                    let cond = node
                        .cond
                        .as_ref()
                        .and_then(|cond| cond.as_bool())
                        .unwrap_or(true);
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
            // }

            true
        }
        Generator::Parent(()) => {
            todo!("what is this even for?")
        } // WidgetKind::ControlFlow(_) => todo!(),
          // WidgetKind::If(_) => todo!(),
          // WidgetKind::Else(_) => todo!(),
    }
}

pub trait Filter<'bp> {
    type Output;

    fn filter<'a>(widget: &'a mut WidgetContainer<'bp>) -> Option<&'a mut Self::Output>;
}

#[derive(Debug)]
pub struct ForEach<'a, 'bp, Fltr> {
    tree: WidgetTreeView<'a, 'bp>,
    _filter: PhantomData<Fltr>,
}

impl<'a, 'bp, Fltr: Filter<'bp>> ForEach<'a, 'bp, Fltr> {
    pub fn new(tree: WidgetTreeView<'a, 'bp>) -> Self {
        Self {
            tree,
            _filter: PhantomData,
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
            let mut children = ForEach::new(children);

            if let Some(el) = Fltr::filter(widget) {
                f(el, children)
            } else {
                children.inner_each(Some(widget), f)
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
