use anathema_render::Size;
use anathema_values::testing::TestState;
use anathema_values::{Context, State, Value};

use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::expressions::Expression;
use crate::layout::{Constraints, Layout};
use crate::nodes::Node;
use crate::{
    AnyWidget, Factory, FactoryContext, LayoutNodes, Nodes, Padding, Widget, WidgetFactory,
};

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct TestLayoutMany;

impl Layout for TestLayoutMany {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        let mut size = Size::ZERO;

        let mut constraints = nodes.constraints;
        nodes.for_each(|mut node| {
            let s = node.layout(constraints)?;
            size.height += s.height;
            size.width = size.width.max(s.width);
            constraints.max_height -= size.height;
            Ok(())
        })?;

        Ok(size)
    }
}

// -----------------------------------------------------------------------------
//   - Widgets -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct TestWidget(pub Value<String>);

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "text"
    }

    fn layout<'e>(&mut self, _nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        match self.0.value_ref() {
            Some(s) => Ok(Size::new(s.len(), 1)),
            None => Ok(Size::ZERO),
        }
    }

    fn position<'tpl>(&mut self, _children: &mut Nodes, _ctx: PositionCtx) {}
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TestWidget(context.text);
        Ok(Box::new(widget))
    }
}

#[derive(Debug)]
struct TestListWidget;

impl Widget for TestListWidget {
    fn kind(&self) -> &'static str {
        "list"
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        TestLayoutMany.layout(nodes)
    }

    fn position<'tpl>(&mut self, _children: &mut Nodes, _ctx: PositionCtx) {
        todo!()
    }
}

struct TestListWidgetFactory;

impl WidgetFactory for TestListWidgetFactory {
    fn make(&self, _context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TestListWidget;
        Ok(Box::new(widget))
    }
}

// -----------------------------------------------------------------------------
//   - Expressions -
// -----------------------------------------------------------------------------

pub struct TestExpression<S> {
    pub state: S,
    pub expr: Box<Expression>,
    pub layout: LayoutCtx,
}

impl<S: State> TestExpression<S> {
    pub fn ctx(&self) -> Context<'_, '_> {
        let ctx = Context::root(&self.state);
        ctx
    }

    pub fn eval(&self) -> Result<Node<'_>> {
        self.expr.eval(&self.ctx(), 0.into())
    }
}

// -----------------------------------------------------------------------------
//   - Test node -
// -----------------------------------------------------------------------------
pub struct TestNodes<'e> {
    pub nodes: Nodes<'e>,
    state: TestState,
}

impl<'e> TestNodes<'e> {
    pub fn new(exprs: &'e [Expression]) -> Self {
        register_test_widget();
        let nodes = Nodes::new(exprs, 0.into());
        Self {
            nodes,
            state: TestState::new(),
        }
    }

    pub fn layout(&mut self) -> Result<Size> {
        let context = Context::root(&self.state);
        let mut layout_nodes = LayoutNodes::new(
            &mut self.nodes,
            Constraints::new(120, 40),
            Padding::ZERO,
            &context,
        );
        TestLayoutMany.layout(&mut layout_nodes)
    }
}

pub(crate) fn register_test_widget() {
    let _ = Factory::register("test", TestWidgetFactory);
    let _ = Factory::register("list", TestListWidgetFactory);
}

