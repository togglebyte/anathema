use std::cell::RefCell;

use anathema_state::States;

use self::scope::Scope;
use super::error::Result;
use crate::attributes::{AllAttributes, Attributes};
use crate::runtime::components::Components;
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::eval::expression::{eval_by_id, RuntimeExpressions};
use crate::runtime::functions::{Function, FunctionTable};
use crate::runtime::widgets::RegisteredWidgets;
use crate::templates::expressions::{self, Expressions};
use crate::templates::{Blueprint, Component, ExpressionId, For, Single, Variables};
use crate::ui::Document;

mod assoc;
pub(crate) mod expression;
pub(crate) mod scope;
pub(crate) mod values;

#[derive(Debug)]
pub struct EvalCtx<'a, 'bp> {
    pub(crate) elements: &'a mut Elements<'bp>,
    pub(crate) attributes: &'a mut AllAttributes<'bp>,
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
        attributes: &'frame mut AllAttributes<'bp>,
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

    fn get_attributes(&self, el: ElementId) -> &Attributes<'bp> {
        todo!()
    }

    fn reserve_element_id(&mut self) -> ElementId {
        self.elements.elements.next_id()
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
        Blueprint::With(with) => todo!(),
        Blueprint::ControlFlow(control_flow) => todo!(),
        Blueprint::Component(stmt) => ComponentEval.eval(stmt, ctx, factory, parent),
        Blueprint::Slot(blueprints) => todo!(),
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
            let rte = eval_by_id(*expr, element_id, ctx);
            eprintln!("{key}: {rte:?}");
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
        // Resolve collection
        // resolve(forloop.data);
        let collection = [1];

        let el = Element::For {
            binding: &input.binding,
        };

        let parent = ctx.insert_element(el, parent);

        for val in collection {
            // scope.scope(forloop.binding, val);
            for child in &input.body {
                eval(child, ctx, factory, Some(parent));
            }
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
        for (key, expr) in input.attributes.iter() {}

        let attributes = Attributes::empty();

        // scope.push_state(state_id);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::attributes::AllAttributes;
    use crate::layout::Layout;
    use crate::testing::{RunBuilder, TestWidget};

    #[test]
    fn eval_single() {
        let tpl = "
            let x = [1, 2, 3]
            node [a: x[0]] x[0]
                node [lol: x[0]] x[0]
        ";

        let mut test = RunBuilder::from_src(tpl);
        let mut inst = test.finish();
        inst.eval(|ctx| panic!("{:?}", ctx.elements));
    }

    #[test]
    fn forloop() {
        let tpl = "
            for x in y
                node x
        ";

        let mut test = RunBuilder::from_src(tpl);
        let mut inst = test.finish();
        inst.eval(|ctx| panic!("{:#?}", ctx.elements));
    }
}
