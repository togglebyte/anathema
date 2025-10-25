use std::cell::RefCell;

use anathema_geometry::{Pos, Size};
use anathema_state::{State, StateId, States};

use crate::attributes::{AllAttributes, Attributes};
use crate::layout::Layout;
use crate::runtime::components::{Component, ComponentId, Components, FnComp, FnState};
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::runtime::eval::expression::{eval_by_id, RuntimeExpressions};
use crate::runtime::eval::scope::Scope;
use crate::runtime::eval::values::TemplateValue;
use crate::runtime::eval::{eval, EvalCtx};
use crate::runtime::functions::FunctionTable;
use crate::runtime::widgets::iter::Children;
use crate::runtime::widgets::{RegisteredWidgets, Widget};
use crate::templates::expressions::Expressions;
use crate::templates::{
    Blueprint, ComponentBlueprintId, Document, Expression, ExpressionId, SourceKind, TemplateSource, Variables,
};

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

// -----------------------------------------------------------------------------
//   - Generic test runner -
// -----------------------------------------------------------------------------
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

    // pub fn add_component(&mut self, comp: impl Component, state: impl State) -> ComponentId {
    //     panic!()
    //     // self.components.insert_component(comp, state)
    // }

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

impl RunBuilder<(Document, Option<Blueprint>)> {
    pub fn from_src(src: &str) -> Self {
        let template = src.to_string();
        let variables = Variables::new();
        let doc = Document::new(template);

        Self {
            inner: (doc, None),
            states: Default::default(),
            components: Default::default(),
            functions: Default::default(),
            variables,
            widget_registry: test_widgets(),
            dirty_elements: vec![],
        }
    }

    pub fn add_component(
        &mut self,
        name: &str,
        template: impl Into<SourceKind>,
        comp: impl Component,
        state: impl State,
    ) -> ComponentId {
        let component_bp_id = self.inner.0.add_component(name, template.into()).unwrap();
        let comp_id = self.components.insert_component(component_bp_id, comp, state);
        comp_id
    }

    pub fn add_prototype(&mut self, name: &str, template: impl Into<SourceKind>, component: FnComp, state: FnState) {
        let component_bp_id = self.inner.0.add_component(name, template.into()).unwrap();
        let comp_id = self.components.insert_prototype(component_bp_id, component, state);
    }

    pub(crate) fn finish(&mut self) -> Instance<'_, '_> {
        let bp = self.inner.0.compile(&mut self.variables).unwrap();
        self.inner.1 = Some(bp);

        Instance::new(
            &self.states,
            &mut self.components,
            &self.variables,
            &self.inner.0.expressions,
            &self.functions,
            self.inner.1.as_ref(),
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
//   - Expression evaluator -
// -----------------------------------------------------------------------------

struct ExpressionEvaluatorComponent<S>(std::marker::PhantomData<S>);
impl<S: State> Component for ExpressionEvaluatorComponent<S> {
    type Message = ();
    type State = S;
}

pub struct ExpressionEvaluator {
    variables: Variables,
    expressions: Expressions,
    components: Components,
    functions: FunctionTable,
}

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
            expressions: Expressions::empty(),
            components: Components::empty(),
            functions: FunctionTable::new(),
        }
    }

    pub fn register_global(&mut self, ident: &str, value: impl Into<Expression>) {
        self.variables.register_global(ident, value, &mut self.expressions);
    }

    pub fn eval<S: State, F>(&mut self, expr: Expression, state: S, f: F)
    where
        F: Fn(&TemplateValue<'_>),
    {
        let mut elements = Elements::empty();

        let component = ExpressionEvaluatorComponent::<S>(Default::default());
        let component = self
            .components
            .insert_component(ComponentBlueprintId::ZERO, component, state);
        let parent = elements.insert(Element::Component(component), None);
        let element = elements.insert(Element::Widget(test_widget("")), Some(parent));

        let mut scope = Scope::empty();
        scope.push_component(parent, component);
        let expr_id = self.expressions.insert_at_root(expr);
        let mut runtime_expressions = RuntimeExpressions::empty();
        let mut attributes = AllAttributes::empty();

        let mut dirty_elements = vec![];

        let mut ctx = EvalCtx::new(
            &mut elements,
            &mut attributes,
            &mut self.components,
            &self.variables,
            &self.expressions,
            &self.functions,
            &mut scope,
            &mut runtime_expressions,
            &mut dirty_elements,
        );

        let value = eval_by_id(expr_id, element, Some(parent), &mut ctx);
        f(&value);
    }
}

// -----------------------------------------------------------------------------
//   - Test widget -
// -----------------------------------------------------------------------------

fn test_widget(s: impl Into<String>) -> RefCell<Box<dyn Widget>> {
    RefCell::new(Box::new(TestWidget(s.into())))
}

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
