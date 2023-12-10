use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;

use anathema_render::Size;
use anathema_values::{Context, NodeId};

pub use self::container::WidgetContainer;
use super::contexts::{PaintCtx, PositionCtx, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::nodes::Nodes;
use crate::LayoutNodes;

mod container;

// Layout:
// 1. Receive constraints
// 2. Layout children
// 3. Get children's suggested size
// 4. Apply offset to children
// 5. Get children's computed size
// ... paint

pub trait Widget {
    /// This should only be used for debugging, as there
    /// is nothing preventing one widget from having the same `kind` as another
    fn kind(&self) -> &'static str {
        "[widget]"
    }

    // -----------------------------------------------------------------------------
    //     - Layout -
    // -----------------------------------------------------------------------------
    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size>;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx);

    fn paint(&mut self, children: &mut Nodes<'_>, mut ctx: PaintCtx<'_, WithSize>) {
        for (widget, children) in children.iter_mut() {
            let ctx = ctx.sub_context(None);
            widget.paint(children, ctx);
        }
    }

    /// Called when a value the widget subscribes to has changed.
    fn update(&mut self, _context: &Context<'_, '_>, _node_id: &NodeId) {}
}

impl Widget for Box<dyn Widget> {
    fn kind(&self) -> &'static str {
        self.as_ref().kind()
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        self.as_mut().layout(nodes)
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.as_mut().position(children, ctx)
    }

    fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.as_mut().paint(children, ctx)
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.as_mut().update(context, node_id)
    }
}

pub trait AnyWidget: Debug {
    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn layout_any<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size>;

    fn kind_any(&self) -> &'static str;

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx);

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>);

    fn update_any(&mut self, context: &Context<'_, '_>, node_id: &NodeId);
}

impl Widget for Box<dyn AnyWidget> {
    fn kind(&self) -> &'static str {
        self.deref().kind_any()
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        self.deref_mut().layout_any(nodes)
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.deref_mut().position_any(children, ctx)
    }

    fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.deref_mut().paint_any(children, ctx)
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.deref_mut().update_any(context, node_id)
    }
}

impl<T: Debug + Widget + 'static> AnyWidget for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_any<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        self.layout(nodes)
    }

    fn kind_any(&self) -> &'static str {
        self.kind()
    }

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.position(children, ctx)
    }

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.paint(children, ctx)
    }

    fn update_any(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.update(context, node_id)
    }
}
