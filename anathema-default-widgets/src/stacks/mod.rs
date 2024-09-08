use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, WidgetId};

pub use self::column::Column;
pub use self::hstack::HStack;
pub use self::row::Row;
pub use self::vstack::VStack;
pub use self::zstack::ZStack;
use crate::layout::many::Many;
use crate::layout::{Axis, Direction, DIRECTION};
use crate::{HEIGHT, MIN_HEIGHT, MIN_WIDTH, WIDTH};

mod column;
mod hstack;
mod row;
mod vstack;
mod zstack;

pub struct Stack(Axis);

impl Stack {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attributes = ctx.attribs.get(id);

        if let Some(width) = attributes.get_usize(MIN_WIDTH) {
            constraints.min_width = width;
        }

        if let Some(height) = attributes.get_usize(MIN_HEIGHT) {
            constraints.min_height = height;
        }

        if let Some(width) = attributes.get_usize(WIDTH) {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attributes.get_usize(HEIGHT) {
            constraints.make_height_tight(height);
        }

        let dir = attributes.get(DIRECTION).unwrap_or_default();
        // Make `unconstrained` an enum instead of a `bool`
        let unconstrained = false;
        let mut many = Many::new(dir, self.0, unconstrained);
        many.layout(children, constraints, ctx)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        let attributes = attribute_storage.get(id);
        let direction = attributes.get(DIRECTION).unwrap_or_default();
        let mut pos = ctx.pos;

        if let Direction::Backward = direction {
            match self.0 {
                Axis::Horizontal => pos.x += ctx.inner_size.width as i32,
                Axis::Vertical => pos.y += ctx.inner_size.height as i32,
            }
        }

        children.for_each(|node, children| {
            match direction {
                Direction::Forward => {
                    node.position(children, pos, attribute_storage, ctx.viewport);

                    match self.0 {
                        Axis::Horizontal => pos.x += node.size().width as i32,
                        Axis::Vertical => pos.y += node.size().height as i32,
                    }
                }
                Direction::Backward => {
                    match self.0 {
                        Axis::Horizontal => pos.x += node.size().width as i32,
                        Axis::Vertical => pos.y -= node.size().height as i32,
                    }

                    node.position(children, pos, attribute_storage, ctx.viewport);
                }
            }

            ControlFlow::Continue(())
        });
    }
}
