use std::cell::RefCell;

use anathema_geometry::{Pos, Size};
use anathema_state::{State, StateId, States};

use crate::attributes::{AllAttributes, Attributes};
use crate::layout::Layout;
use crate::runtime::components::{Component, ComponentId, Components};
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::eval::expression::RuntimeExpressions;
use crate::runtime::eval::scope::Scope;
use crate::runtime::eval::{eval, EvalCtx};
use crate::runtime::functions::FunctionTable;
use crate::runtime::widgets::iter::Children;
use crate::runtime::widgets::{RegisteredWidgets, Widget};
use crate::templates::expressions::Expressions;
use crate::templates::{Blueprint, Document, Expression, ExpressionId, Variables};

fn test_widgets() -> RegisteredWidgets {
    let mut factory = RegisteredWidgets::empty();
    factory.register_default::<TestWidget>("node");
    factory
}

#[derive(Debug)]
pub struct NoDoc {
    expressions: Expressions,
}

impl Default for NoDoc {
    fn default() -> Self {
        Self {
            expressions: Expressions::empty(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RunBuilder<T> {
    inner: T,
    states: States,
    components: Components,
    functions: FunctionTable,
    variables: Variables,
    widget_registry: RegisteredWidgets,
    dirty_elements: Vec<ElementId>,
}

impl RunBuilder<NoDoc> {
    pub fn new() -> Self {
        let mut inst = Self::default();
        inst.widget_registry = test_widgets();
        inst
    }

    pub fn insert_expression(&mut self, expression: impl Into<Expression>) -> ExpressionId {
        self.inner.expressions.insert_at_root(expression)
    }

    pub fn register_global(&mut self, ident: &str, expression: impl Into<Expression>) {
        self.variables
            .register_global(ident, expression, &mut self.inner.expressions)
            .unwrap()
    }

    pub fn add_component(&mut self, comp: impl Component, state: impl State) -> ComponentId {
        self.components.temporary_insert(comp, state)
        // self.states.insert(state)
    }

    pub(crate) fn finish(&mut self) -> Instance<'_, '_> {
        Instance::new(
            &self.states,
            &mut self.components,
            &self.variables,
            &self.inner.expressions,
            &self.functions,
            None,
            &self.widget_registry,
            &mut self.dirty_elements,
        )
    }
}

impl RunBuilder<(Document, Blueprint)> {
    pub fn from_src(src: &str) -> Self {
        let template = src.to_string();
        let mut variables = Variables::new();
        let mut doc = Document::new(template);
        let bp = doc.compile(&mut variables).unwrap();

        Self {
            inner: (doc, bp),
            states: Default::default(),
            components: Default::default(),
            functions: Default::default(),
            variables,
            widget_registry: test_widgets(),
            dirty_elements: vec![],
        }
    }

    pub(crate) fn finish(&mut self) -> Instance<'_, '_> {
        Instance::new(
            &self.states,
            &mut self.components,
            &self.variables,
            &self.inner.0.expressions,
            &self.functions,
            Some(&self.inner.1),
            &self.widget_registry,
            &mut self.dirty_elements,
        )
    }
}

impl<T> RunBuilder<T> {}

pub struct Instance<'frame, 'bp> {
    pub scope: Scope<'bp>,
    attributes: AllAttributes<'bp>,
    elements: Elements<'bp>,
    components: &'frame mut Components,
    variables: &'frame Variables,
    expressions: &'bp Expressions,
    functions: &'bp FunctionTable,
    blueprint: Option<&'bp Blueprint>,
    widget_registry: &'bp RegisteredWidgets,
    runtime_expressions: RuntimeExpressions<'bp>,
    dirty_elements: &'frame mut Vec<ElementId>,
}

impl<'frame, 'bp> Instance<'frame, 'bp> {
    fn new(
        states: &'frame States,
        components: &'frame mut Components,
        variables: &'frame Variables,
        expressions: &'bp Expressions,
        functions: &'bp FunctionTable,
        blueprint: Option<&'bp Blueprint>,
        widget_registry: &'bp RegisteredWidgets,
        dirty_elements: &'frame mut Vec<ElementId>,
    ) -> Self {
        Self {
            scope: Scope::empty(),
            attributes: AllAttributes::empty(),
            elements: Elements::empty(),
            runtime_expressions: RuntimeExpressions::empty(),
            components,
            variables,
            expressions,
            functions,
            blueprint,
            widget_registry,
            dirty_elements,
        }
    }

    pub fn run<F>(&mut self, f: F)
    where
        F: Fn(&mut EvalCtx<'_, '_>),
    {
        let mut eval_ctx = EvalCtx::new(
            &mut self.elements,
            &mut self.attributes,
            &mut self.components,
            &self.variables,
            &self.expressions,
            &self.functions,
            &mut self.scope,
            &mut self.runtime_expressions,
            &mut self.dirty_elements,
        );

        f(&mut eval_ctx);
    }

    pub fn add_widget(&mut self, widget: impl Widget, parent: Option<ElementId>) -> ElementId {
        let el = Element::Widget(RefCell::new(Box::new(widget)));
        let id = self.elements.insert(el, parent);
        self.attributes.insert(id, Attributes::empty());
        id
    }

    pub(crate) fn add_component(&mut self, comp_id: ComponentId, parent: Option<ElementId>) -> ElementId {
        let el = Element::Component(comp_id);
        let id = self.elements.insert(el, parent);
        self.attributes.insert(id, Attributes::empty());
        id
    }

    pub(crate) fn eval<F>(&mut self, f: F)
    where
        F: Fn(&mut EvalCtx<'_, '_>),
    {
        let mut eval_ctx = EvalCtx::new(
            &mut self.elements,
            &mut self.attributes,
            &mut self.components,
            &self.variables,
            &self.expressions,
            &self.functions,
            &mut self.scope,
            &mut self.runtime_expressions,
            &mut self.dirty_elements,
        );

        eval(self.blueprint.unwrap(), &mut eval_ctx, self.widget_registry, None).unwrap();
        f(&mut eval_ctx);
    }
}

// -----------------------------------------------------------------------------
//   - Old test jazz -
// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct TestWidget(pub String);

impl Widget for TestWidget {
    fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size {
        let mut size = Size::new(self.0.len() as u16, 1);

        for mut child in children {
            let child_size = child.layout(layout);
            size.width = size.width.max(child_size.width);
            size.height += child_size.height;
        }

        size
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self) {
        todo!()
    }

    fn describe(&self) -> &str {
        if self.0.is_empty() {
            return "<test>";
        }
        &self.0
    }
}
