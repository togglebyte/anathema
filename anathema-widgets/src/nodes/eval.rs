use std::iter::Cycle;
use std::slice::Iter;

use anathema_geometry::{Pos, Region, Size};
use anathema_state::{AnyValue, States, Value};
use anathema_store::slab::SlabIndex;
use anathema_store::smallmap::SmallIndex;
use anathema_templates::blueprints::{Blueprint, Component, ControlFlow, Else, For, Single};
use anathema_templates::{ComponentBlueprintId, Globals};

use super::element::Element;
use super::loops::{Iteration, LOOP_INDEX};
use super::{component, controlflow, WidgetContainer};
use crate::components::{AnyComponent, ComponentKind, ComponentRegistry};
use crate::container::{Cache, Container};
use crate::error::{Error, Result};
use crate::expressions::{eval, eval_collection, ExprEvalCtx};
use crate::layout::{EvalCtx, Viewport};
use crate::values::{Collection, ValueId, ValueIndex};
use crate::widget::{Attributes, Components, FloatingWidgets, ValueKey, WidgetTreeView};
use crate::{
    eval_blueprint, AttributeStorage, ChangeList, DirtyWidgets, Factory, GlyphMap, Scope, WidgetId, WidgetKind,
    WidgetTree,
};

/// Evaluation context
// pub struct EvalContext<'rt, 'bp> {
//     pub viewport: Viewport,
//     pub(crate) scope: Scope<'bp>,
//     pub(crate) states: States,
//     pub(crate) globals: &'bp Globals,
//     pub(crate) parent: Option<WidgetId>,
//     pub sidecar: &'rt mut Sidecar<'rt, 'bp>,
//     pub(crate) force_layout: bool,
// }

// impl<'rt, 'bp> EvalContext<'rt, 'bp> {
//     pub fn new(
//         globals: &'bp Globals,
//         factory: &'rt Factory,
//         scope: Scope<'bp>,
//         states: States,
//         viewport: Viewport,
//         sidecar: &'rt mut Sidecar<'rt, 'bp>,
//         force_layout: bool,
//     ) -> Self {
//         Self {
//             globals,
//             scope,
//             states,
//             parent: None,
//             viewport,
//             sidecar,
//             force_layout,
//         }
//     }

//     pub fn needs_layout(&self, node_id: WidgetId) -> bool {
//         self.sidecar.dirty_widgets.contains(node_id) || self.force_layout
//     }

//     pub(crate) fn expr_eval_ctx(&self) -> ExprEvalCtx<'_, 'bp> {
//         ExprEvalCtx {
//             scope: &self.scope,
//             states: &self.states,
//             attributes: &*self.sidecar.attribute_storage,
//             globals: self.globals,
//         }
//     }
// }

/// Evaluate a node kind
pub(super) trait Evaluator {
    type Input<'bp>;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        context: &mut EvalCtx<'_, 'bp>,
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
            // let expr_eval_ctx = ctx.expr_eval_ctx();

            let strings = &mut *ctx.strings;
            let mut ctx = ExprEvalCtx {
                scope: ctx.scope,
                states: ctx.states,
                attributes: ctx.attribute_storage,
                globals: ctx.globals,
            };

            let value = attributes.insert_with(ValueKey::Value, |value_index| {
                eval(expr, &ctx, strings, (widget_id, value_index))
            });
            attributes.value = Some(value);
        }

        for (key, expr) in single.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let strings = &mut *ctx.strings;
                let ctx = ExprEvalCtx {
                    scope: ctx.scope,
                    states: ctx.states,
                    attributes: ctx.attribute_storage,
                    globals: ctx.globals,
                };

                eval(expr, &ctx, strings, (widget_id, value_index))
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
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let value_id = ValueId::from((transaction.node_id(), ValueIndex::ZERO));

        let strings = &mut *ctx.strings;
        let ctx = ExprEvalCtx {
            scope: ctx.scope,
            states: ctx.states,
            attributes: ctx.attribute_storage,
            globals: ctx.globals,
        };
        let collection = eval_collection(&for_loop.data, &ctx, strings, value_id);

        let for_loop = super::loops::For {
            binding: &for_loop.binding,
            body: &for_loop.body,
            collection,
        };

        let body = for_loop.body;
        let widget = WidgetKind::For(for_loop);
        let widget = WidgetContainer::new(widget, body);

        let for_loop_id = transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;

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
                    let value_index = SmallIndex::from_usize(i);
                    let strings = &mut *ctx.strings;
                    let ctx = ExprEvalCtx {
                        scope: ctx.scope,
                        states: ctx.states,
                        attributes: ctx.attribute_storage,
                        globals: ctx.globals,
                    };

                    controlflow::Else {
                        cond: e
                            .cond
                            .as_ref()
                            .map(|cond| eval(cond, &ctx, strings, (widget_id, SmallIndex::from_usize(i)))),
                        body: &e.body,
                        show: false,
                    }
                })
                .collect(),
        });
        let widget = WidgetContainer::new(widget, &[]);
        let for_loop_id = transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;
        let parent = tree.path(for_loop_id);
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
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        let mut attributes = Attributes::empty(widget_id);

        for (key, expr) in input.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                let strings = &mut *ctx.strings;
                let ctx = ExprEvalCtx {
                    scope: ctx.scope,
                    states: ctx.states,
                    attributes: ctx.attribute_storage,
                    globals: ctx.globals,
                };

                eval(expr, &ctx, strings, (widget_id, value_index))
            });
        }

        let component_id = usize::from(input.id).into();
        ctx.attribute_storage.insert(widget_id, attributes);
        let (kind, component, state) = ctx.get_component(component_id).ok_or(Error::ComponentConsumed)?;
        let state_id = ctx.states.insert(state);
        let comp_widget = component::Component::new(
            &input.name,
            &input.body,
            component,
            state_id,
            component_id,
            widget_id,
            kind,
            &input.assoc_functions,
            ctx.parent,
        );

        let widget = WidgetKind::Component(comp_widget);
        let widget = WidgetContainer::new(widget, &input.body);
        let widget_id = transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;

        let path = tree.path(widget_id);
        ctx.components.push(path, component_id, widget_id, state_id);

        Ok(())
    }
}
