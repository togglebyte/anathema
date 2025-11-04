use std::cell::RefCell;

use anathema_compiler::blueprints::{Blueprint, Component, ControlFlow, For, Single, With};
use anathema_compiler::expressions::{ExpressionId, Expressions};
use anathema_compiler::Variables;
use anathema_store::remotecell::RemoteHandle;

use self::scope::Scope;
use super::error::Result;
use crate::attributes::{AttributeRegistry, Attributes, ValueKey};
use crate::components::Components;
use crate::elements::{Element, ElementId, Elements};
use crate::eval::expression::{eval_by_id, eval_collection, RuntimeExpression, RuntimeExpressions};
use crate::eval::values::TemplateValue;
use crate::functions::{Function, FunctionTable};
use crate::value::{Value, ValueIndex};
use crate::widgets::RegisteredWidgets;

mod assoc;
pub(crate) mod expression;
pub(crate) mod scope;
pub(crate) mod values;

#[derive(Debug)]
pub struct EvalCtx<'a, 'bp> {
    pub(crate) elements: &'a mut Elements<'bp>,
    pub(crate) attributes: &'a mut AttributeRegistry<'bp>,
    pub(crate) components: &'a mut Components,
    pub(crate) variables: &'a Variables,
    pub(crate) expressions: &'bp Expressions,
    pub(crate) runtime_expressions: &'a mut RuntimeExpressions<'bp>,
    pub(crate) functions: &'bp FunctionTable,
    pub(crate) scope: &'a mut Scope<'bp>,
    pub(crate) dirty_elements: &'a mut Vec<ElementId>,
}

impl<'frame, 'bp> EvalCtx<'frame, 'bp> {
    pub(crate) fn new(
        elements: &'frame mut Elements<'bp>,
        attributes: &'frame mut AttributeRegistry<'bp>,
        components: &'frame mut Components,
        variables: &'frame Variables,
        expressions: &'bp Expressions,
        functions: &'bp FunctionTable,
        scope: &'frame mut Scope<'bp>,
        runtime_expressions: &'frame mut RuntimeExpressions<'bp>,
        dirty_elements: &'frame mut Vec<ElementId>,
    ) -> Self {
        Self {
            elements,
            attributes,
            components,
            variables,
            expressions,
            functions,
            scope,
            runtime_expressions,
            dirty_elements,
        }
    }

    fn insert_element(&mut self, element: Element<'bp>, parent: Option<ElementId>) -> ElementId {
        self.elements.insert(element, parent)
    }

    fn insert_attributes(&mut self, id: ElementId, attributes: Attributes<'bp>) {
        self.attributes.insert(id, attributes);
    }

    fn lookup_function(&self, fun: &str) -> Option<&'bp Function> {
        self.functions.lookup(fun)
    }

    fn get_attributes(&self, element: ElementId) -> Option<&Attributes<'bp>> {
        self.attributes.get(element)
    }

    fn reserve_element_id(&mut self) -> ElementId {
        self.elements.elements.next_id()
    }

    fn nearest_scope_id(&self, element: Option<ElementId>) -> Option<scope::ScopeId> {
        let element = element?;
        self.scope.nearest_scope_id(element, &self.elements)
    }
}

trait Evaluator {
    type Input<'bp>;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()>;
}

pub fn eval<'a, 'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalCtx<'a, 'bp>,
    factory: &RegisteredWidgets,
    parent: Option<ElementId>,
) -> Result<()> {
    match blueprint {
        Blueprint::Single(stmt) => SingleEval.eval(stmt, ctx, factory, parent),
        Blueprint::For(stmt) => ForEval.eval(stmt, ctx, factory, parent),
        Blueprint::With(stmt) => WithEval.eval(stmt, ctx, factory, parent),
        Blueprint::ControlFlow(stmt) => ControlFlowEval.eval(stmt, ctx, factory, parent),
        Blueprint::Component(stmt) => ComponentEval.eval(stmt, ctx, factory, parent),
        Blueprint::Slot(stmt) => todo!(),
    }
}

struct SingleEval;

impl Evaluator for SingleEval {
    type Input<'bp> = &'bp Single;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()> {
        let element_id = ctx.reserve_element_id();

        let mut attributes = Attributes::empty();

        for (key, expr) in input.attributes.iter() {
            let (value, _) = eval_by_id(*expr, element_id, parent, ctx);
            attributes.set_attribute(ValueKey::Attribute(key), value);
        }

        if let Some(expr) = &input.value {
            let (val, _) = eval_by_id(*expr, element_id, parent.map(Into::into), ctx);
            attributes.set_attribute(ValueKey::Value, val);
        }

        let el = match factory.make(&input.ident, &attributes) {
            Ok(el) => Element::Widget(RefCell::new(el)),
            Err(e) => panic!(), //return Err(ctx.error(e)),
        };

        let parent = ctx.insert_element(el, parent);
        assert_eq!(parent, element_id);

        ctx.insert_attributes(parent, attributes);

        for child in &input.children {
            eval(child, ctx, factory, Some(parent));
        }

        Ok(())
    }
}

struct ForEval;

impl Evaluator for ForEval {
    type Input<'bp> = &'bp For;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()> {
        let element_id = ctx.reserve_element_id();
        let collection = eval_collection(input.data, element_id, parent.map(Into::into), ctx);

        let el = Element::For {
            binding: &input.binding,
            collection: collection.clone(),
        };

        let parent = ctx.insert_element(el, parent);
        assert_eq!(parent, element_id);

        for loop_counter in 0..collection.len() {
            let loop_counter = Value::new(loop_counter as u32);
            let loop_counter_ref = loop_counter.reference();
            let iteration = Element::Iteration { loop_counter };
            let parent = ctx.insert_element(iteration, Some(parent));

            ctx.scope
                .scope_iteration(parent, &input.binding, collection.expr, loop_counter_ref);

            for child in &input.body {
                eval(child, ctx, factory, Some(parent));
            }
        }

        Ok(())
    }
}

struct ControlFlowEval;

impl Evaluator for ControlFlowEval {
    type Input<'bp> = &'bp ControlFlow;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()> {
        let element_id = ctx.reserve_element_id();

        let el = Element::ControlFlow;
        let parent = ctx.insert_element(el, parent);

        for els in &input.elses {
            let cond = els
                .cond
                .map(|cond| eval_by_id(cond, element_id, Some(parent).map(Into::into), ctx))
                .map(|(val, _)| val);
            let el = Element::Condition(cond);
            let parent = ctx.insert_element(el, Some(parent));

            for child in &els.body {
                eval(child, ctx, factory, Some(parent));
            }
        }

        Ok(())
    }
}

struct WithEval;

impl Evaluator for WithEval {
    type Input<'bp> = &'bp With;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()> {
        let element_id = ctx.reserve_element_id();

        let (_, value_index) = eval_by_id(input.data, element_id, parent.map(Into::into), ctx);

        let parent = ctx.insert_element(Element::With, parent);
        ctx.scope.scope_with(parent, &input.binding, value_index);

        for child in &input.body {
            eval(child, ctx, factory, Some(parent));
        }

        Ok(())
    }
}

struct ComponentEval;

impl Evaluator for ComponentEval {
    type Input<'bp> = &'bp Component;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        parent: Option<ElementId>,
    ) -> Result<()> {
        let element_id = ctx.reserve_element_id();
        let comp_id = ctx.components.by_blueprint_id(input.id);

        let mut attributes = Attributes::empty();

        for (key, expr) in input.attributes.iter() {
            let rte = eval_by_id(*expr, element_id, parent.map(Into::into), ctx);
        }

        let el = Element::Component(comp_id);

        let parent = ctx.insert_element(el, parent);
        assert_eq!(parent, element_id);

        ctx.insert_attributes(parent, attributes);

        ctx.scope.push_component(element_id, comp_id);

        for child in &input.body {
            eval(child, ctx, factory, Some(parent));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anathema::State;

    use super::*;
    use crate::attributes::AttributeRegistry;
    use crate::testing::{RunBuilder, TestWidget};
    use crate::value::{List, Value};
    use crate::widgets::Layout;

    #[derive(Debug, State)]
    struct TestState {
        value: Value<u16>,
        list: Value<List<u16>>,
    }

    struct Comp;

    impl crate::components::Component for Comp {
        type Message = ();
        type State = ();
    }

    #[test]
    fn eval_single() {
        let component_tpl = "
            node state.value
            node state.value
            node state.value
            node state.value
            node state.value
            node state.value
        ";

        let state = TestState {
            value: 123.into(),
            list: List::from_iter(0..10).into(),
        };

        let mut test = RunBuilder::from_src("@comp");
        let comp_id = test.add_component("comp", component_tpl, Comp, state);

        let mut inst = test.finish();
        inst.eval(|ctx| {
            // * Get hold of the state
            {
                let state = ctx.components.get_state_mut(comp_id).unwrap();
                let state = &mut *state.to_mut();
                let state = state.as_mut() as &mut dyn std::any::Any;
                let test_state = state.downcast_mut::<TestState>().unwrap();
                test_state.value.set(321);
            }

            // * Drain changes
            let mut changes = crate::value::Changes::empty();
            crate::value::drain_changes(&mut changes);

            eprintln!("{changes:?}");

            while let Some((expr_id, _)) = changes.pop() {
                expression::re_evalute_expr(expr_id.into(), ctx);
            }

            // * Change the value
            // * Look at dirty widgets
            // * Update the values using the remote handle

            // panic!("{:#?}", ctx.attributes)
        });
    }

    #[test]
    fn forloop() {
        let tpl = "
            let list = [2, 2, 30, 40]
            for x in list
                node x
        ";

        let mut test = RunBuilder::from_src("@comp");
        let state = TestState {
            value: 123.into(),
            list: List::from_iter(5..15).into(),
        };
        let comp_id = test.add_component("comp", tpl, Comp, state);
        let mut inst = test.finish();
        inst.eval(|ctx| panic!("{:#?}", ctx.elements));
    }

    #[test]
    fn if_else() {
        let tpl = "
            if true
                node value
            else if !false 
                node 'test'
            else
                node 'meh'
        ";

        let mut test = RunBuilder::from_src("@comp");
        let state = TestState {
            value: 123.into(),
            list: List::from_iter(5..15).into(),
        };
        let comp_id = test.add_component("comp", tpl, Comp, state);
        let mut inst = test.finish();
        inst.eval(|ctx| panic!("{:#?}", ctx.elements));
    }

    #[test]
    fn with_value() {
        let tpl = "
            with x as 2
                node x
        ";

        let mut test = RunBuilder::from_src("@comp");
        let state = TestState {
            value: 123.into(),
            list: List::from_iter(5..15).into(),
        };
        let comp_id = test.add_component("comp", tpl, Comp, state);
        let mut inst = test.finish();
        inst.eval(|ctx| panic!("{:#?}", ctx.elements));
    }
}
