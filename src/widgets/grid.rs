use display::Size;

use crate::attributes::Attributes;


use crate::layout::grid;

use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

#[derive(Debug)]
pub struct Grid {
    pub children: Vec<WidgetContainer>,
}

impl Grid {
    pub const KIND: &'static str = "Grid";

    pub fn new() -> Self {
        Self { children: Vec::new() }
    }
}

impl Widget for Grid {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        grid::layout(&mut self.children, ctx)
    }

    fn position(&mut self, _ctx: PositionCtx) {
        panic!()
        // stacked::position(&mut self.children, ctx)
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        for child in self.children.iter_mut() {
            let ctx = ctx.sub_context(None);
            child.paint(ctx);
        }
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        self.children.iter_mut().collect()
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.children.push(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(pos) = self.children.iter().position(|c| c.id.eq(child_id)) {
            return Some(self.children.remove(pos));
        }

        None
    }

    fn update(&mut self, _: Attributes) {}
}

