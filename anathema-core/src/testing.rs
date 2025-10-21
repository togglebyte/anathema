use std::cell::RefCell;

use anathema_geometry::{Pos, Size};
use anathema_state::{State, StateId, States};

use crate::attributes::{AllAttributes, Attributes};
use crate::layout::Layout;
use crate::runtime::components::{Component, ComponentId, Components};
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::eval::scope::Scope;
use crate::runtime::eval::EvalCtx;
use crate::runtime::functions::FunctionTable;
use crate::runtime::widgets::iter::Children;
use crate::runtime::widgets::{RegisteredWidgets, Widget};
use crate::templates::expressions::Expressions;
use crate::templates::{Blueprint, Document, Expression, ExpressionId, Variables};

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
    blueprint: Option<Blueprint>,
}

impl RunBuilder<NoDoc> {
    pub fn new() -> Self {
        Self::default()
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
        )
    }
}

impl RunBuilder<Document> {
    pub fn from_src(src: &str) -> Self {
        let template = src.to_string();
        let doc = Document::new(template);

        Self {
            inner: doc,
            states: Default::default(),
            components: Default::default(),
            functions: Default::default(),
            variables: Default::default(),
            blueprint: Default::default(),
        }
    }

    pub fn compile(mut self) -> () {
        panic!()
    }

    pub(crate) fn finish(&mut self) -> Instance<'_, '_> {
        Instance::new(
            &self.states,
            &mut self.components,
            &self.variables,
            &self.inner.expressions,
            &self.functions,
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
}

impl<'frame, 'bp> Instance<'frame, 'bp> {
    fn new(
        states: &'frame States,
        components: &'frame mut Components,
        variables: &'frame Variables,
        expressions: &'bp Expressions,
        functions: &'bp FunctionTable,
    ) -> Self {
        Self {
            scope: Scope::empty(),
            attributes: AllAttributes::empty(),
            elements: Elements::empty(),

            components,
            variables,
            expressions,
            functions,
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

pub fn with_ctx<F>(f: F)
where
    F: for<'a, 'bp> Fn(EvalCtx<'a, 'bp>),
{
    let mut elements = Elements::empty();
    let mut expressions = Expressions::empty();
    let mut attributes = AllAttributes::empty();
    let mut components = Components::empty();
    let mut variables = Variables::new();
    let mut functions = FunctionTable::new();
    let mut scope = Scope::empty();

    let eval_ctx = EvalCtx::new(
        &mut elements,
        &mut attributes,
        &mut components,
        &variables,
        &expressions,
        &functions,
        &mut scope,
    );

    f(eval_ctx);
}

pub fn with_blueprint<F>(src: &str, f: F)
where
    F: for<'a, 'bp> Fn(&mut EvalCtx<'a, 'bp>, &'a Blueprint),
{
    with_ctx(|mut ctx| {
        // let mut doc = crate::templates::Document::new(src.to_string());

        // let mut variables = Variables::new();
        // let blueprint = doc.compile(&mut variables).unwrap();

        // ctx.expressions = &doc.expressions;
        // ctx.variables = &variables;

        // f(&mut ctx, &blueprint);
    });

    // let mut factory = RegisteredWidgets::empty();
    // factory.register_default::<TestWidget>("node");

    // let mut doc = crate::templates::Document::new(src.to_string());

    // let mut elements = Elements::empty();
    // let mut attributes = AllAttributes::empty();
    // let mut components = Components::empty();
    // let mut variables = Variables::new();
    // let mut functions = FunctionTable::new();
    // let mut scope = Scope::empty();
    // let mut states = States::new();

    // let blueprint = doc.compile(&mut variables).unwrap();

    // let mut eval_ctx = EvalCtx::new(
    //     &mut elements,
    //     &mut attributes,
    //     &mut components,
    //     &variables,
    //     &doc.expressions,
    //     &functions,
    //     &mut scope,
    //     &states,
    // );

    // crate::runtime::eval::eval(&blueprint, &mut eval_ctx, &factory, None).unwrap();

    // f(&mut eval_ctx, &blueprint);
}

// pub(crate) fn test_widget(value: impl Into<String>) -> InsertNode {
//     let el = TestWidget(value.into());
//     InsertNode::from(el)
// }
