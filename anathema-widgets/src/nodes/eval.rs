use std::iter::Cycle;
use std::slice::Iter;

use anathema_geometry::{Pos, Region, Size};
use anathema_state::{AnyState, States, Value};
use anathema_store::smallmap::SmallIndex;
use anathema_templates::blueprints::{Blueprint, Component, ControlFlow, Else, For, Single};
use anathema_templates::{Globals, WidgetComponentId};

use super::element::Element;
use super::loops::{Iteration, LOOP_INDEX};
use super::{component, controlflow, WidgetContainer};
use crate::components::{AnyComponent, ComponentKind, ComponentRegistry};
use crate::container::{Cache, Container};
use crate::error::{Error, Result};
use crate::expressions::{eval, eval_collection};
use crate::layout::Viewport;
use crate::values::{Collection, ValueId, ValueIndex};
use crate::widget::{Attributes, Components, FloatingWidgets, ValueKey, WidgetTreeView};
use crate::{
    eval_blueprint, AttributeStorage, DirtyWidgets, Factory, GlyphMap, Scope, WidgetId, WidgetKind, WidgetTree,
};

/// Evaluation context
pub struct EvalContext<'a, 'b, 'bp> {
    pub(super) globals: &'bp Globals,
    pub(super) factory: &'a Factory,
    pub(crate) scope: &'b mut Scope<'bp>,
    pub(super) states: &'b mut States,
    pub(super) component_registry: &'b mut ComponentRegistry,
    pub(super) floating_widgets: &'b mut FloatingWidgets,
    pub(super) components: &'b mut Components,
    pub dirty_widgets: &'b mut DirtyWidgets,
    pub(super) parent: Option<WidgetId>,
    pub attribute_storage: &'b mut AttributeStorage<'bp>,
    pub viewport: &'a Viewport,
    pub glyph_map: &'b mut GlyphMap,
    pub force_layout: bool,
}

impl<'a, 'b, 'bp> EvalContext<'a, 'b, 'bp> {
    pub fn new(
        globals: &'bp Globals,
        factory: &'a Factory,
        scope: &'b mut Scope<'bp>,
        states: &'b mut States,
        component_registry: &'b mut ComponentRegistry,
        attribute_storage: &'b mut AttributeStorage<'bp>,
        floating_widgets: &'b mut FloatingWidgets,
        components: &'b mut Components,
        dirty_widgets: &'b mut DirtyWidgets,
        viewport: &'a Viewport,
        glyph_map: &'b mut GlyphMap,
        force_layout: bool,
    ) -> Self {
        Self {
            globals,
            factory,
            scope,
            states,
            component_registry,
            attribute_storage,
            floating_widgets,
            components,
            dirty_widgets,
            parent: None,
            viewport,
            glyph_map,
            force_layout,
        }
    }

    fn get_component(
        &mut self,
        component_id: WidgetComponentId,
    ) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn AnyState>)> {
        self.component_registry.get(component_id)
    }

    pub fn needs_layout(&self, node_id: WidgetId) -> bool {
        self.dirty_widgets.contains(node_id) || self.force_layout
    }
}

/// Evaluate a node kind
pub(super) trait Evaluator {
    type Input<'bp>;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        context: &mut EvalContext<'_, '_, 'bp>,
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
        ctx: &mut EvalContext<'_, '_, 'bp>,
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
            let value = attributes.insert_with(ValueKey::Value, |value_index| {
                eval(
                    expr,
                    ctx.globals,
                    ctx.scope,
                    ctx.states,
                    ctx.attribute_storage,
                    (widget_id, value_index),
                )
            });
            attributes.value = Some(value);
        }

        for (key, expr) in single.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                eval(
                    expr,
                    ctx.globals,
                    ctx.scope,
                    ctx.states,
                    ctx.attribute_storage,
                    (widget_id, value_index),
                )
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
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let value_id = ValueId::from((transaction.node_id(), ValueIndex::ZERO));

        let collection = eval_collection(
            &for_loop.data,
            ctx.globals,
            ctx.scope,
            ctx.states,
            ctx.attribute_storage,
            value_id,
        );

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
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        let widget = WidgetKind::ControlFlow(controlflow::ControlFlow {
            // if_node: controlflow::If {
            //     cond: eval(
            //         &control_flow.if_node.cond,
            //         ctx.globals,
            //         ctx.scope,
            //         ctx.states,
            //         ctx.attribute_storage,
            //         (widget_id, SmallIndex::ZERO),
            //     ),
            //     body: &control_flow.if_node.body,
            //     show: false,
            // },
            elses: control_flow
                .elses
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let value_index = SmallIndex::from(i + 1);

                    controlflow::Else {
                        cond: e.cond.as_ref().map(|cond| {
                            eval(
                                cond,
                                ctx.globals,
                                ctx.scope,
                                ctx.states,
                                ctx.attribute_storage,
                                (widget_id, SmallIndex::ZERO),
                            )
                        }),
                        body: &e.body,
                        show: false,
                    }
                })
                .collect(),
        });
        let widget = WidgetContainer::new(widget, &[]);
        let for_loop_id = transaction.commit_child(widget).ok_or(Error::TreeTransactionFailed)?;
        let parent = tree.path(for_loop_id);

        // tree.with_value_mut(for_loop_id, move |parent, _widget, tree| {
        //     IfEval.eval(&control_flow.if_node, ctx, parent, tree)?;
        //     control_flow
        //         .elses
        //         .iter()
        //         .try_for_each(|e| ElseEval.eval(e, ctx, parent, tree))?;
        //     Ok(())
        // })?;

        // let mut was_set = false;

        // tree.children_of(&parent, |node, values| {
        //     let Some((_, widget)) = values.get_mut(node.value()) else { return };
        //     match widget {
        //         WidgetKind::If(widget) => {
        //             if widget.is_true() {
        //                 widget.show = true;
        //                 was_set = true;
        //             } else {
        //                 widget.show = false;
        //             }
        //         }
        //         WidgetKind::Else(widget) => {
        //             if was_set {
        //                 widget.show = false;
        //             } else if widget.is_true() {
        //                 widget.show = true;
        //                 was_set = true;
        //             }
        //         }
        //         _ => unreachable!(),
        //     }
        // });

        Ok(())
    }
}

// struct IfEval;

// impl Evaluator for IfEval {
//     type Input<'bp> = &'bp If;

//     fn eval<'bp>(
//         &mut self,
//         input: Self::Input<'bp>,
//         ctx: &mut EvalContext<'_, '_, 'bp>,
//         parent: &[u16],
//         tree: &mut WidgetTreeView<'_, 'bp>,
//     ) -> Result<()> {
//         panic!("once control flow is done, come back here")
//         // let transaction = tree.insert(parent);
//         // let node_id = transaction.node_id();

//         // let value_id = (node_id, ValueIndex::ZERO);
//         // let cond = eval(
//         //     &input.cond,
//         //     ctx.globals,
//         //     ctx.scope,
//         //     ctx.states,
//         //     ctx.attribute_storage,
//         //     value_id,
//         // );

//         // let if_widget = controlflow::If { cond, show: false };

//         // let if_widget_id = transaction
//         //     .commit_child(WidgetKind::If(if_widget))
//         //     .ok_or(Error::TreeTransactionFailed)?;

//         // let parent = tree.path(if_widget_id);
//         // for bp in &input.body {
//         //     eval_blueprint(bp, ctx, &parent, tree)?;
//         // }

//         // Ok(())
//     }
// }

struct ElseEval;

impl Evaluator for ElseEval {
    type Input<'bp> = &'bp Else;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        panic!("do it once if is done")
        // let transaction = tree.insert(parent);
        // let widget_id = transaction.node_id();
        // let value_id = (widget_id, ValueIndex::ZERO);

        // let cond = input.cond.as_ref().map(|cond| {
        //     eval(
        //         cond,
        //         ctx.globals,
        //         ctx.scope,
        //         ctx.states,
        //         ctx.attribute_storage,
        //         value_id,
        //     )
        // });

        // let else_widget = controlflow::Else {
        //     cond,
        //     body: &input.body,
        //     show: false,
        // };

        // let _ = transaction
        //     .commit_child(WidgetKind::Else(else_widget))
        //     .ok_or(Error::TreeTransactionFailed)?;

        // let parent = tree.path(widget_id);
        // for bp in &input.body {
        //     eval_blueprint(bp, ctx, &parent, tree)?;
        // }

        // Ok(())
    }
}

pub(super) struct ComponentEval;

impl Evaluator for ComponentEval {
    type Input<'bp> = &'bp Component;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &[u16],
        tree: &mut WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        let mut attributes = Attributes::empty(widget_id);

        for (key, expr) in input.attributes.iter() {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                eval(
                    expr,
                    ctx.globals,
                    ctx.scope,
                    ctx.states,
                    ctx.attribute_storage,
                    (widget_id, value_index),
                )
            });
        }

        let component_id = usize::from(input.id).into();
        ctx.attribute_storage.insert(widget_id, attributes);
        let (kind, component, state) = ctx.get_component(component_id).ok_or(Error::ComponentConsumed)?;
        let state_id = ctx.states.insert(state);
        let comp_widget = component::Component::new(
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

        // tree.with_value_mut(widget_id, move |parent, widget, tree| {
        //     let WidgetKind::Component(component) = widget else { unreachable!() };
        //     ctx.scope.push();

        //     // Insert internal state
        //     let state_id = component.state_id();
        //     ctx.scope.insert_state(state_id);

        //     // Expose attributes to the template

        //     // Scope attributes
        //     ctx.scope.scope_component_attributes(widget_id);

        //     for bp in &input.body {
        //         ctx.parent = Some(widget_id);
        //         eval_blueprint(bp, ctx, parent, tree)?;
        //     }

        //     ctx.scope.pop();

        //     Ok(())
        // })?;

        Ok(())
    }
}
