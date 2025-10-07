use anathema_geometry::Region;
use anathema_store::slab::SlabIndex;
use anathema_store::smallmap::SmallIndex;
use anathema_templates::blueprints::{Blueprint, Component, ControlFlow, For, Single, With};
use anathema_value_resolver::{Attributes, ResolverCtx, Scope, ValueKey, resolve, resolve_collection};

use super::element::Element;
use super::{WidgetContainer, component, controlflow};
use crate::WidgetKind;
use crate::container::{Cache, Container};
use crate::error::{ErrorKind, Result};
use crate::layout::EvalCtx;
use crate::widget::WidgetTreeView;

/// Evaluate a node kind
pub(super) trait Evaluator {
    type Input<'bp>;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        context: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()>;
}

pub(super) struct SingleEval;

impl Evaluator for SingleEval {
    type Input<'bp> = &'bp Single;

    fn eval<'bp>(
        &mut self,
        single: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        // -----------------------------------------------------------------------------
        //   - New api -
        // -----------------------------------------------------------------------------
        let mut attributes = Attributes::empty();

        if let Some(expr_id) = single.value.as_ref() {
            let mut ctx = ResolverCtx::new(
                ctx.globals,
                scope,
                ctx.states,
                ctx.attribute_storage,
                ctx.function_table,
                ctx.expressions,
            );

            _ = attributes.insert_with(ValueKey::Value, |value_index| {
                resolve(*expr_id, &mut ctx, (widget_id, value_index))
            });
        }

        for (key, expr_id) in single.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let mut ctx = ResolverCtx::new(
                    ctx.globals,
                    scope,
                    ctx.states,
                    ctx.attribute_storage,
                    ctx.function_table,
                    ctx.expressions,
                );
                resolve(*expr_id, &mut ctx, (widget_id, value_index))
            });
        }

        let widget = match ctx.factory.make(&single.ident, &attributes) {
            Ok(widget) => widget,
            Err(e) => return Err(ctx.error(e)),
        };

        // Is the widget a floating widget?
        if widget.any_floats() {
            ctx.floating_widgets.insert(widget_id);
        }

        ctx.attribute_storage.insert(widget_id, attributes);

        // Container
        let container = Container {
            inner: widget,
            id: widget_id,
            inner_bounds: Region::ZERO,
            cache: Cache::ZERO,
        };

        // Widget
        let widget = WidgetKind::Element(Element::new(&single.ident, container));
        let widget = WidgetContainer::new(widget, &single.children, ctx.parent_widget);

        transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;

        Ok(())
    }
}

pub(super) struct ForLoopEval;

impl Evaluator for ForLoopEval {
    type Input<'bp> = &'bp For;

    fn eval<'bp>(
        &mut self,
        for_loop: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);

        let mut resolver_ctx = ResolverCtx::new(
            ctx.globals,
            scope,
            ctx.states,
            ctx.attribute_storage,
            ctx.function_table,
            ctx.expressions,
        );
        let collection = resolve_collection(
            for_loop.data,
            &mut resolver_ctx,
            transaction.node_id(),
            SmallIndex::ZERO,
        );

        let for_loop = super::loops::For {
            binding: &for_loop.binding,
            body: &for_loop.body,
            collection,
        };

        let body = for_loop.body;
        let widget = WidgetKind::For(for_loop);
        let widget = WidgetContainer::new(widget, body, ctx.parent_widget);
        transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;
        Ok(())
    }
}

pub(super) struct WithEval;

impl Evaluator for WithEval {
    type Input<'bp> = &'bp With;

    fn eval<'bp>(
        &mut self,
        with: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);

        let mut resolver_ctx = ResolverCtx::new(
            ctx.globals,
            scope,
            ctx.states,
            ctx.attribute_storage,
            ctx.function_table,
            ctx.expressions,
        );
        let data = resolve(with.data, &mut resolver_ctx, (transaction.node_id(), SmallIndex::ZERO));

        let with = super::with::With {
            binding: &with.binding,
            body: &with.body,
            data,
        };

        let body = with.body;
        let widget = WidgetKind::With(with);
        let widget = WidgetContainer::new(widget, body, ctx.parent_widget);
        transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;
        Ok(())
    }
}

pub(super) struct ControlFlowEval;

impl Evaluator for ControlFlowEval {
    type Input<'bp> = &'bp ControlFlow;

    fn eval<'bp>(
        &mut self,
        control_flow: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        let widget = WidgetKind::ControlFlow(controlflow::ControlFlow {
            elses: control_flow
                .elses
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let mut ctx = ResolverCtx::new(
                        ctx.globals,
                        scope,
                        ctx.states,
                        ctx.attribute_storage,
                        ctx.function_table,
                        ctx.expressions,
                    );

                    controlflow::Else {
                        cond: e
                            .cond
                            .map(|cond| resolve(cond, &mut ctx, (widget_id, SmallIndex::from_usize(i)))),
                        body: &e.body,
                        show: false,
                    }
                })
                .collect(),
        });
        let widget = WidgetContainer::new(widget, &[], ctx.parent_widget);
        transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;
        Ok(())
    }
}

pub(super) struct ComponentEval;

impl Evaluator for ComponentEval {
    type Input<'bp> = &'bp Component;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        scope: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        let mut attributes = Attributes::empty();

        for (key, expr_id) in input.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let mut ctx = ResolverCtx::new(
                    ctx.globals,
                    scope,
                    ctx.states,
                    ctx.attribute_storage,
                    ctx.function_table,
                    ctx.expressions,
                );
                resolve(*expr_id, &mut ctx, (widget_id, value_index))
            });
        }

        let component_id = usize::from(input.id).into();
        ctx.attribute_storage.insert(widget_id, attributes);

        // let (kind, component, state) = ctx.get_component(component_id);
        let comp = ctx.get_component(component_id);
        let (kind, component, state) = match comp {
            Some(comp) => comp,
            None => return Err(ctx.error(ErrorKind::ComponentConsumed(input.name.clone()))),
        };

        let state_id = ctx.states.insert(state);
        let accept_ticks = component.any_ticks();

        let comp_widget = component::Component::new(
            &input.name,
            input.name_id,
            &input.body,
            component,
            state_id,
            component_id,
            widget_id,
            kind,
            &input.assoc_functions,
            ctx.parent_component,
        );

        let widget = WidgetKind::Component(comp_widget);
        let widget = WidgetContainer::new(widget, &input.body, ctx.parent_widget);
        let widget_id = transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;
        ctx.new_components.push((widget_id, state_id));

        ctx.components.push(component_id, widget_id, state_id, accept_ticks);

        Ok(())
    }
}

pub(super) struct SlotEval;

impl Evaluator for SlotEval {
    type Input<'bp> = &'bp [Blueprint];

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'_, 'bp>,
        _: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget = WidgetContainer::new(WidgetKind::Slot, input, ctx.parent_widget);
        transaction
            .commit_child(widget)
            .ok_or_else(|| ctx.error(ErrorKind::TreeTransactionFailed))?;
        Ok(())
    }
}
