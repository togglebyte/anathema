use std::collections::HashMap;

use anathema_geometry::{Pos, Size};
use anathema_state::{State, States, Value};
use anathema_store::smallmap::SmallIndex;
use anathema_store::tree::NodePath;
use anathema_templates::blueprints::{Component, ControlFlow, Else, For, If, Single};

use super::element::Element;
use super::loops::{Iteration, LOOP_INDEX};
use super::{component, controlflow};
use crate::components::{AnyComponent, ComponentId, ComponentRegistry};
use crate::container::Container;
use crate::expressions::{eval, eval_collection};
use crate::values::{ValueId, ValueIndex};
use crate::widget::{Attributes, FloatingWidgets, ValueKey};
use crate::{eval_blueprint, AttributeStorage, Factory, Scope, WidgetKind, WidgetTree};

/// Evaluation context
pub struct EvalContext<'a, 'b, 'bp> {
    pub(super) factory: &'a Factory,
    pub(super) scope: &'b mut Scope<'bp>,
    pub(super) states: &'b mut States,
    pub(super) components: &'b mut ComponentRegistry,
    pub(super) attribute_storage: &'b mut AttributeStorage<'bp>,
    pub(super) floating_widgets: &'b mut FloatingWidgets,
}

impl<'a, 'b, 'bp> EvalContext<'a, 'b, 'bp> {
    pub fn new(
        factory: &'a Factory,
        scope: &'b mut Scope<'bp>,
        states: &'b mut States,
        components: &'b mut ComponentRegistry,
        attribute_storage: &'b mut AttributeStorage<'bp>,
        floating_widgets: &'b mut FloatingWidgets,
    ) -> Self {
        Self {
            factory,
            scope,
            states,
            components,
            attribute_storage,
            floating_widgets,
        }
    }

    fn get_component(&mut self, component_id: ComponentId) -> (Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>) {
        self.components.get(component_id)
    }
}

/// Evaluate a node kind
pub(super) trait Evaluator {
    type Input<'bp>;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        context: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    );
}

pub(super) struct SingleEval;

impl Evaluator for SingleEval {
    type Input<'bp> = &'bp Single;

    fn eval<'bp>(
        &mut self,
        single: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();

        // -----------------------------------------------------------------------------
        //   - New api -
        // -----------------------------------------------------------------------------
        let mut attributes = Attributes::empty(widget_id);

        if let Some(expr) = single.value.as_ref() {
            attributes.insert_with(ValueKey::Value, |value_index| {
                eval(expr, ctx.scope, ctx.states, (widget_id, value_index))
            });
        }

        for (key, expr) in &single.attributes {
            attributes.insert_with(ValueKey::Attribute(key), |value_index| {
                eval(expr, ctx.scope, ctx.states, (widget_id, value_index))
            });
        }

        let widget = ctx.factory.make(&single.ident, &attributes);

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
            size: Size::ZERO,
        };

        // Widget
        let widget = WidgetKind::Element(Element::new(&single.ident, container));

        transaction.commit_child(widget);

        // Children
        let parent = &tree.path(widget_id).clone();
        for child in &single.children {
            super::eval_blueprint(child, ctx, parent, tree);
        }
    }
}

pub(super) struct ForLoopEval;

impl ForLoopEval {
    pub(super) fn eval_body<'bp>(
        &self,
        for_loop: &super::loops::For<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        for index in 0..for_loop.collection.len() {
            ctx.scope.push();
            for_loop.scope_value(ctx.scope, index);

            let iter_id = tree
                .insert(parent)
                .commit_child(WidgetKind::Iteration(Iteration {
                    loop_index: Value::new(index as i64),
                    binding: for_loop.binding,
                }))
                .unwrap(); // TODO unwrap

            // Scope the iteration value
            tree.with_value(iter_id, |parent, widget, tree| {
                let WidgetKind::Iteration(iter) = widget else { unreachable!() };
                ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

                for bp in for_loop.body {
                    eval_blueprint(bp, ctx, parent, tree);
                }
            });

            ctx.scope.pop();
        }
    }
}

impl Evaluator for ForLoopEval {
    type Input<'bp> = &'bp For;

    fn eval<'bp>(
        &mut self,
        for_loop: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);
        let value_id = ValueId::from((transaction.node_id(), ValueIndex::ZERO));

        let widget = WidgetKind::For(super::loops::For {
            binding: &for_loop.binding,
            collection: eval_collection(&for_loop.data, ctx.scope, ctx.states, value_id),
            body: &for_loop.body,
        });

        let for_loop_id = transaction.commit_child(widget).unwrap(); // TODO: unwrap...

        tree.with_value(for_loop_id, move |parent, widget, tree| {
            let WidgetKind::For(for_loop) = widget else { unreachable!() };
            self.eval_body(for_loop, ctx, parent, tree);
        });
    }
}

pub(super) struct ControlFlowEval;

impl Evaluator for ControlFlowEval {
    type Input<'bp> = &'bp ControlFlow;

    fn eval<'bp>(
        &mut self,
        control_flow: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);
        let widget = WidgetKind::ControlFlow(controlflow::ControlFlow {});
        let for_loop_id = transaction.commit_child(widget).unwrap(); // TODO: unwrap...
        let parent = tree.path(for_loop_id).clone();

        tree.with_value(for_loop_id, move |parent, _widget, tree| {
            IfEval.eval(&control_flow.if_node, ctx, parent, tree);
            control_flow
                .elses
                .iter()
                .for_each(|e| ElseEval.eval(e, ctx, parent, tree));
        });

        let mut was_set = false;

        tree.children_of(&parent, |node, values| {
            let Some((_, widget)) = values.get_mut(node.value()) else { return };
            match widget {
                WidgetKind::If(widget) => {
                    if widget.is_true() {
                        widget.show = true;
                        was_set = true;
                    } else {
                        widget.show = false;
                    }
                }
                WidgetKind::Else(widget) => {
                    if was_set {
                        widget.show = false;
                    } else if widget.is_true() {
                        widget.show = true;
                        was_set = true;
                    }
                }
                _ => unreachable!(),
            }
        });
    }
}

struct IfEval;

impl Evaluator for IfEval {
    type Input<'bp> = &'bp If;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);
        let node_id = transaction.node_id();

        let value_id = (node_id, ValueIndex::ZERO);
        let cond = eval(&input.cond, ctx.scope, ctx.states, value_id);

        let if_widget = controlflow::If { cond, show: false };

        let if_widget_id = transaction.commit_child(WidgetKind::If(if_widget)).unwrap(); // TODO: unwrap..

        let parent = &tree.path(if_widget_id).clone();
        for bp in &input.body {
            eval_blueprint(bp, ctx, parent, tree);
        }
    }
}

struct ElseEval;

impl Evaluator for ElseEval {
    type Input<'bp> = &'bp Else;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);
        let widget_id = transaction.node_id();
        let value_id = (widget_id, ValueIndex::ZERO);

        let cond = input
            .cond
            .as_ref()
            .map(|cond| eval(cond, ctx.scope, ctx.states, value_id));

        let else_widget = controlflow::Else {
            cond,
            body: &input.body,
            show: false,
        };

        let _ = transaction.commit_child(WidgetKind::Else(else_widget)).unwrap();
        let parent = &tree.path(widget_id).clone();
        for bp in &input.body {
            eval_blueprint(bp, ctx, parent, tree);
        }
    }
}

pub(super) struct ComponentEval;

impl Evaluator for ComponentEval {
    type Input<'bp> = &'bp Component;

    fn eval<'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        parent: &NodePath,
        tree: &mut WidgetTree<'bp>,
    ) {
        let transaction = tree.insert(parent);

        let external_state = match &input.state {
            Some(map) => {
                let mut state_map = HashMap::new();
                for (i, (k, v)) in map.iter().enumerate() {
                    let idx: SmallIndex = (i as u8).into();
                    let val = eval(v, ctx.scope, ctx.states, (transaction.node_id(), idx));
                    state_map.insert((&**k, idx), val);
                }
                Some(state_map)
            }
            None => None,
        };

        let component_id = usize::from(input.id).into();
        let (component, state) = ctx.get_component(component_id);
        let Some(component) = component else { return };
        let state_id = state.map(|state| ctx.states.insert(state));
        let comp_widget = component::Component::new(&input.body, component, state_id, external_state, component_id);

        let widget_id = transaction.commit_child(WidgetKind::Component(comp_widget)).unwrap();

        tree.with_value(widget_id, move |parent, widget, tree| {
            let WidgetKind::Component(component) = widget else { panic!() };
            ctx.scope.push();

            // Insert internal state
            if let Some(state_id) = component.state_id() {
                ctx.scope.insert_state(state_id);
            }

            // Insert external state (if there is one)
            if let Some(state) = &component.external_state {
                for ((k, _), v) in state.iter() {
                    let v = v.downgrade();
                    ctx.scope.scope_downgrade(k, v);
                }
            }

            for bp in &input.body {
                eval_blueprint(bp, ctx, parent, tree);
            }

            ctx.scope.pop();
        });
    }
}
