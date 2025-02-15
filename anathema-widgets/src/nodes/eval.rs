use anathema_geometry::{Pos, Region};
use anathema_state::{Subscriber, Value};
use anathema_store::slab::SlabIndex;
use anathema_store::smallmap::SmallIndex;
use anathema_templates::blueprints::{Blueprint, Component, ControlFlow, For, Single};
use anathema_value_resolver::{resolve, resolve_collection, Attributes, ResolverCtx, Scope, ValueKey};

use super::element::Element;
use super::{component, controlflow, WidgetContainer};
use crate::container::{Cache, Container};
use crate::error::{Error, Result};
use crate::layout::EvalCtx;
use crate::widget::WidgetTreeView;
use crate::WidgetKind;

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
        let mut attributes = Attributes::empty(widget_id);

        if let Some(expr) = single.value.as_ref() {
            let ctx = ResolverCtx::new(ctx.globals, scope, ctx.states, ctx.attribute_storage);

            let value = attributes.insert_with(ValueKey::Value, |value_index| {
                resolve(expr, &ctx, (widget_id, value_index))
            });
            attributes.value = Some(value);
        }

        for (key, expr) in single.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let ctx = ResolverCtx::new(ctx.globals, &scope, ctx.states, ctx.attribute_storage);
                resolve(expr, &ctx, (widget_id, value_index))
            });
        }

        let widget = ctx.factory.make(&single.ident, &attributes)?;

        // Is the widget a floating widget?
        if widget.any_floats() {
            ctx.floating_widgets.insert(widget_id);
        }

        ctx.attribute_storage.insert(widget_id, attributes);

        // Container
        let container = Container {
            inner: widget,
            id: widget_id,
            pos: Pos::ZERO,
            inner_bounds: Region::ZERO,
            cache: Cache::ZERO,
        };

        // Widget
        let widget = WidgetKind::Element(Element::new(&single.ident, container));
        let widget = WidgetContainer::new(widget, &single.children);

        transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;

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
        let value_id = Subscriber::from((transaction.node_id(), SmallIndex::ZERO));

        let ctx = ResolverCtx::new(ctx.globals, &scope, ctx.states, ctx.attribute_storage);
        let collection = resolve_collection(&for_loop.data, &ctx, value_id);

        let for_loop = super::loops::For {
            binding: &for_loop.binding,
            body: &for_loop.body,
            collection,
        };

        let body = for_loop.body;
        let widget = WidgetKind::For(for_loop);
        let widget = WidgetContainer::new(widget, body);
        transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;
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
                    let ctx = ResolverCtx::new(ctx.globals, scope, ctx.states, ctx.attribute_storage);

                    controlflow::Else {
                        cond: e
                            .cond
                            .as_ref()
                            .map(|cond| resolve(cond, &ctx, (widget_id, SmallIndex::from_usize(i)))),
                        body: &e.body,
                        show: false,
                    }
                })
                .collect(),
        });
        let widget = WidgetContainer::new(widget, &[]);
        transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;
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

        let mut attributes = Attributes::empty(widget_id);

        for (key, expr) in input.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let ctx = ResolverCtx::new(ctx.globals, &scope, ctx.states, ctx.attribute_storage);
                resolve(expr, &ctx, (widget_id, value_index))
            });
        }

        let component_id = usize::from(input.id).into();
        ctx.attribute_storage.insert(widget_id, attributes);
        let (kind, component, state) = ctx.get_component(component_id).ok_or(Error::ComponentConsumed)?;
        let state_id = ctx.states.insert(Value::new(state));
        let accept_ticks = component.any_ticks();

        let comp_widget = component::Component::new(
            &input.name,
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
        let widget = WidgetContainer::new(widget, &input.body);
        let widget_id = transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;

        let path = tree.path(widget_id);
        ctx.components
            .push(path, component_id, widget_id, state_id, accept_ticks);

        Ok(())
    }
}

pub(super) struct SlotEval;

impl Evaluator for SlotEval {
    type Input<'bp> = &'bp [Blueprint];

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        _: &mut EvalCtx<'_, 'bp>,
        _: &Scope<'_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget = WidgetContainer::new(WidgetKind::Slot, input);
        transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;
        Ok(())
    }
}
