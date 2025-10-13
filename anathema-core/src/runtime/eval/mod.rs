use std::cell::RefCell;

use self::scope::Scope;
use crate::attributes::{AllAttributes, Attributes};
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::widgets::RegisteredWidgets;
use crate::templates::{Blueprint, ExpressionId, For, Single};
use crate::ui::Document;

mod scope;

struct EvalCtx<'a, 'bp> {
    elements: &'a mut Elements<'bp>,
    attributes: &'a mut AllAttributes,
}

impl<'a, 'bp> EvalCtx<'a, 'bp> {
    fn insert(&mut self, element: Element<'bp>, parent: Option<ElementId>) -> ElementId {
        self.elements.insert(element, parent)
    }

    fn new(elements: &'a mut Elements<'bp>, attributes: &'a mut AllAttributes) -> Self {
        Self { elements, attributes }
    }
}

trait Evaluator {
    type Input<'bp>;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        ctx: &mut EvalCtx<'a, 'bp>,
        factory: &RegisteredWidgets,
        scope: &mut Scope,
        parent: Option<ElementId>,
    ) -> Result<(), ()>;
}

pub fn eval<'a, 'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalCtx<'a, 'bp>,
    factory: &RegisteredWidgets,
    scope: &mut Scope,
    parent: Option<ElementId>,
) -> Result<(), ()> {
    match blueprint {
        Blueprint::Single(stmt) => SingleEval.eval(stmt, ctx, factory, scope, parent),
        Blueprint::For(stmt) => ForEval.eval(stmt, ctx, factory, scope, parent),
        Blueprint::With(with) => todo!(),
        Blueprint::ControlFlow(control_flow) => todo!(),
        Blueprint::Component(component) => todo!(),
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
        scope: &mut Scope,
        parent: Option<ElementId>,
    ) -> Result<(), ()> {
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
        scope: &mut Scope,
        parent: Option<ElementId>,
    ) -> Result<(), ()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::attributes::AllAttributes;
    use crate::layout::Layout;
    use crate::testing::TestElement;

    #[test]
    fn forloop() {
        let mut doc = crate::templates::Document::new(
            "
            for x in y
                node x
        ",
        );
        let blueprint = doc.compile(&mut crate::templates::Variables::new()).unwrap();

        let mut statements = Statements::empty();
        let mut attributes = AllAttributes::empty();

        let mut eval_tree = EvalCtx::new(&mut statements, &mut attributes);

        let mut factory = RegisteredElements::empty();
        factory.register_default::<TestElement>("node");
        let mut scope = Scope::empty();

        eval(&blueprint, &mut eval_tree, &factory, &mut scope, None);

        panic!("{statements:#?}");
    }
}
