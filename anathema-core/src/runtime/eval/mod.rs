use std::cell::RefCell;

use self::scope::Scope;
use super::error::Result;
use crate::attributes::{AllAttributes, Attributes};
use crate::runtime::components::Components;
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::widgets::RegisteredWidgets;
use crate::templates::{Blueprint, Component, ExpressionId, For, Single};
use crate::ui::Document;

mod scope;
mod values;

struct EvalCtx<'a, 'bp> {
    elements: &'a mut Elements<'bp>,
    attributes: &'a mut AllAttributes,
    components: &'a mut Components,
}

impl<'a, 'bp> EvalCtx<'a, 'bp> {
    fn insert(&mut self, element: Element<'bp>, parent: Option<ElementId>) -> ElementId {
        self.elements.insert(element, parent)
    }

    fn new(elements: &'a mut Elements<'bp>, attributes: &'a mut AllAttributes, components: &'a mut Components) -> Self {
        Self {
            elements,
            attributes,
            components,
        }
    }
}

trait Evaluator {
    type Input<'bp>;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        scope: &mut Scope<'bp>,
        parent: Option<ElementId>,
    ) -> Result<()>;
}

pub fn eval<'a, 'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalCtx<'a, 'bp>,
    factory: &RegisteredWidgets,
    scope: &mut Scope<'bp>,
    parent: Option<ElementId>,
) -> Result<()> {
    match blueprint {
        Blueprint::Single(stmt) => SingleEval.eval(stmt, ctx, factory, scope, parent),
        Blueprint::For(stmt) => ForEval.eval(stmt, ctx, factory, scope, parent),
        Blueprint::With(with) => todo!(),
        Blueprint::ControlFlow(control_flow) => todo!(),
        Blueprint::Component(stmt) => ComponentEval.eval(stmt, ctx, factory, scope, parent),
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
        scope: &mut Scope<'bp>,
        parent: Option<ElementId>,
    ) -> Result<()> {
        for (key, expr) in input.attributes.iter() {}

        let attributes = Attributes::empty();

        let stmt = match factory.make(&input.ident, &attributes) {
            Ok(el) => Element::Widget(RefCell::new(el)),
            Err(e) => panic!(), //return Err(ctx.error(e)),
        };

        let parent = ctx.insert(stmt, parent);

        for child in &input.children {
            eval(child, ctx, factory, scope, Some(parent));
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
        scope: &mut Scope<'bp>,
        parent: Option<ElementId>,
    ) -> Result<()> {
        // Resolve collection
        // resolve(forloop.data);
        let collection = [1];

        let el = Element::For {
            binding: &input.binding,
        };

        let parent = ctx.insert(el, parent);

        for val in collection {
            // scope.scope(forloop.binding, val);
            for child in &input.body {
                eval(child, ctx, factory, scope, Some(parent));
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
        scope: &mut Scope<'bp>,
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
    use crate::testing::TestWidget;

    #[test]
    fn forloop() {
        let mut doc = crate::templates::Document::new(
            "
            for x in y
                node x
        ",
        );
        let blueprint = doc.compile(&mut crate::templates::Variables::new()).unwrap();

        let mut elements = Elements::empty();
        let mut attributes = AllAttributes::empty();
        let mut components = Components::empty();

        let mut eval_tree = EvalCtx::new(&mut elements, &mut attributes, &mut components);

        let mut factory = RegisteredWidgets::empty();
        factory.register_default::<TestWidget>("node");
        let mut scope = Scope::empty();

        eval(&blueprint, &mut eval_tree, &factory, &mut scope, None);

        panic!("{elements:#?}");
    }
}
