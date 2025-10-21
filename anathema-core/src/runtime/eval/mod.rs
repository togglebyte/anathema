use std::cell::RefCell;

use anathema_state::States;

use self::scope::Scope;
use super::error::Result;
use crate::attributes::{AllAttributes, Attributes};
use crate::runtime::components::Components;
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::functions::{Function, FunctionTable};
use crate::runtime::widgets::RegisteredWidgets;
use crate::templates::expressions::Expressions;
use crate::templates::{Blueprint, Component, ExpressionId, For, Single, Variables};
use crate::ui::Document;

mod expression;
pub(crate) mod scope;
mod testing;
pub(crate) mod values;

pub struct EvalCtx<'a, 'bp> {
    pub(crate) elements: &'a mut Elements<'bp>,
    pub(crate) attributes: &'a mut AllAttributes<'bp>,
    pub(crate) components: &'a mut Components,
    pub(crate) variables: &'a Variables,
    pub(crate) expressions: &'bp Expressions,
    pub(crate) functions: &'bp FunctionTable,
    pub(crate) scope: &'a mut Scope<'bp>,
}

impl<'frame, 'bp> EvalCtx<'frame, 'bp> {
    fn insert_element(&mut self, element: Element<'bp>, parent: Option<ElementId>) -> ElementId {
        self.elements.insert(element, parent)
    }

    fn insert_attributes(&mut self, id: ElementId, attributes: Attributes<'bp>) {
        self.attributes.insert(id, attributes);
    }

    pub(crate) fn new(
        elements: &'frame mut Elements<'bp>,
        attributes: &'frame mut AllAttributes<'bp>,
        components: &'frame mut Components,
        variables: &'frame Variables,
        expressions: &'bp Expressions,
        functions: &'bp FunctionTable,
        scope: &'frame mut Scope<'bp>,
    ) -> Self {
        Self {
            elements,
            attributes,
            components,
            variables,
            expressions,
            functions,
            scope,
        }
    }

    fn lookup_function(&self, fun: &str) -> Option<&'bp Function> {
        self.functions.lookup(fun)
    }

    fn get_attributes(&self, el: ElementId) -> &Attributes<'bp> {
        todo!()
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
        for (key, expr) in input.attributes.iter() {}

        let attributes = Attributes::empty();

        let el = match factory.make(&input.ident, &attributes) {
            Ok(el) => Element::Widget(RefCell::new(el)),
            Err(e) => panic!(), //return Err(ctx.error(e)),
        };

        let parent = ctx.insert_element(el, parent);
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
    use crate::testing::{with_blueprint, TestWidget};

    #[test]
    fn forloop() {
        let tpl = "
            for x in y
                node x
        ";
        with_blueprint(tpl, |ctx, _bp| {
            panic!("{:#?}", ctx.elements);
        });
    }
}
