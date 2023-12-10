use anathema_render::Size;
use anathema_values::testing::TestState;
use anathema_values::{Attributes, Context, Path, State, Value, ValueExpr};

use super::nodes::Node;
use super::{ControlFlow, ElseExpr, IfExpr};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::generator::expressions::{Expression, LoopExpr, SingleNode};
use crate::layout::{Constraints, Layout, Layouts};
use crate::{AnyWidget, Factory, FactoryContext, Nodes, Padding, Widget, WidgetFactory};

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct TestLayoutMany;

impl Layout for TestLayoutMany {
    fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        layout: &LayoutCtx,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        let mut size = Size::ZERO;

        children.for_each(data, layout, |widget, children, ctx| {
            let s = widget.layout(children, layout.constraints, ctx)?;
            size.height += s.height;
            size.width = size.width.max(s.width);
            Ok(())
        })?;

        Ok(size)
    }
}

// -----------------------------------------------------------------------------
//   - Widgets -
// -----------------------------------------------------------------------------

pub struct TestWidget(pub Value<String>);

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "text"
    }

    fn layout(
        &mut self,
        _children: &mut Nodes<'_>,
        _layout: &LayoutCtx,
        _data: &Context<'_, '_>,
    ) -> Result<Size> {

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

struct TestListWidget;

impl Widget for TestListWidget {
    fn kind(&self) -> &'static str {
        "list"
    }

    fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        layout: &LayoutCtx,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        let mut layout = Layouts::new(TestLayoutMany, layout);
        layout.layout(children, data)
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
        let constraints = Constraints::new(120, 40);
        let layout = LayoutCtx::new(constraints, Padding::ZERO);

        TestLayoutMany.layout(&mut self.nodes, &layout, &context)
    }
}

pub(crate) fn register_test_widget() {
    Factory::register("test", TestWidgetFactory);
    Factory::register("list", TestListWidgetFactory);
}
