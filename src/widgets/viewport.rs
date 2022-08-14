use std::any::Any;

use super::{fields, Direction, LayoutCtx, NodeId, PaintCtx, PositionCtx, UpdateCtx, VertEdge, WithSize};
use crate::display::Size;
use crate::widgets::layout::{horizontal, vertical};
use crate::widgets::{Offset, Widget, WidgetContainer};

#[derive(Debug)]
pub struct Viewport {
    pub offset: Offset,
    pub direction: Direction,
    pub children: Vec<WidgetContainer>,
}

impl Viewport {
    const KIND: &'static str = "Viewport";

    pub fn new(offset: Offset, direction: Direction) -> Self {
        Self { offset, children: vec![], direction }
    }

    pub fn swap_edges(&mut self, size: Size) {
        let total_child_height = self.children.iter().map(|c| c.size.height).sum::<usize>() as i32;
        let start = total_child_height - size.height as i32;

        match self.offset.v_edge {
            Some(VertEdge::Top(offset)) => {
                let new_offset = -(offset + start);
                self.offset.v_edge = Some(VertEdge::Bottom(new_offset));
                #[cfg(feature = "log")]
                {
                    log::info!("swapping from top to bottom, from {offset} to {new_offset} (start: {start})");
                }
            }
            Some(VertEdge::Bottom(offset)) => {
                let new_offset = offset + start;
                self.offset.v_edge = Some(VertEdge::Top(new_offset));
                #[cfg(feature = "log")]
                {
                    log::info!("swapping from bottom to top, from {offset} to {new_offset} (start: {start})");
                }
            }
            None => {}
        }
    }
}

impl Widget for Viewport {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        let mut child_ctx = ctx;
        match self.direction {
            Direction::Horizontal => child_ctx.constraints.make_width_unbounded(),
            Direction::Vertical => child_ctx.constraints.make_height_unbounded(),
        }

        // Make sure at least one axis is bounded
        assert!(!ctx.constraints.is_unbounded());

        // Layout children with one unbounded axis
        let size = match self.direction {
            Direction::Horizontal => horizontal::layout(&mut self.children, child_ctx, true),
            Direction::Vertical => vertical::layout(&mut self.children, child_ctx, true),
        };

        Size { width: size.width.min(ctx.constraints.max_width), height: size.height.min(ctx.constraints.max_height) }
    }

    fn position(&mut self, mut ctx: PositionCtx) {
        // TODO: swap the edges around and calculate the offset to freeze scroll
        match self.offset.v_edge {
            Some(VertEdge::Top(offset)) => ctx.pos.y += offset,
            Some(VertEdge::Bottom(offset)) => {
                let total_child_height = self.children.iter().map(|c| c.size.height).sum::<usize>();
                let start = total_child_height - ctx.size.height;
                ctx.pos.y -= start as i32 + offset;
            }
            None => {}
        }
        match self.direction {
            Direction::Vertical => vertical::position(&mut self.children, ctx),
            Direction::Horizontal => horizontal::position(&mut self.children, ctx),
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let clip = ctx.create_region();
        #[cfg(feature = "log")]
        {
            // log::info!("height: {}", ctx.local_size.height);
        }
        for child in self.children.iter_mut() {
            let ctx = ctx.sub_context(Some(&clip));
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

    fn update(&mut self, ctx: UpdateCtx) {
        if ctx.attributes.has(fields::V_OFFSET_EDGE) {
            let val = match self.offset.v_edge {
                None => 0,
                Some(VertEdge::Top(val) | VertEdge::Bottom(val)) => val,
            };

            match ctx.attributes.get_str(fields::V_OFFSET_EDGE) {
                Some(fields::TOP) => {
                    if let Some(VertEdge::Bottom(_)) = self.offset.v_edge {
                        self.swap_edges(ctx.size);
                    } else {
                        self.offset.v_edge = Some(VertEdge::Top(val));
                        #[cfg(feature = "log")]
                        {
                            log::info!("updating offset (top)");
                        }
                    }
                }
                Some(fields::BOTTOM) => {
                    if let Some(VertEdge::Top(_)) = self.offset.v_edge {
                        self.swap_edges(ctx.size);
                    } else {
                        self.offset.v_edge = Some(VertEdge::Bottom(val));
                        #[cfg(feature = "log")]
                        {
                            log::info!("updating offset (bottom)");
                        }
                    }
                }
                _ => {}
            }
        }

        if ctx.attributes.has(fields::V_OFFSET) {
            match self.offset.v_edge {
                None => {
                    self.offset.v_edge =
                        Some(VertEdge::Top(ctx.attributes.get_signed_int(fields::V_OFFSET).unwrap_or(0) as i32))
                }
                Some(VertEdge::Top(ref mut val) | VertEdge::Bottom(ref mut val)) => {
                    *val = ctx.attributes.get_signed_int(fields::V_OFFSET).unwrap_or(0) as i32
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
