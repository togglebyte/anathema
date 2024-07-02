use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::text::StringSession;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::layout::many::Many;
use crate::layout::{Axis, Direction, AXIS, DIRECTION};

#[derive(Debug, Default)]
pub struct Viewport {
    // Store the offset as a u32 so it can never go below 0
    offset: u32,
    // The size of the children since the last layout call
    inner_size: Size,

    direction: Direction,
}

impl Viewport {
    pub fn scroll(&mut self, direction: Direction, amount: u32) {
        match (self.direction, direction) {
            (Direction::Forward, Direction::Forward) => self.offset = self.offset.saturating_sub(amount),
            (Direction::Forward, Direction::Backward) => self.offset += amount,
            (Direction::Backward, Direction::Forward) => self.offset += amount,
            (Direction::Backward, Direction::Backward) => self.offset = self.offset.saturating_sub(amount),
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll(Direction::Forward, 1);
    }

    pub fn scroll_up_by(&mut self, amount: u32) {
        self.scroll(Direction::Forward, amount);
    }

    pub fn scroll_down(&mut self) {
        self.scroll(Direction::Backward, 1);
    }

    pub fn scroll_down_by(&mut self, amount: u32) {
        self.scroll(Direction::Backward, amount);
    }

    pub fn scroll_right(&mut self) {
        self.scroll(Direction::Forward, 1);
    }

    pub fn scroll_right_by(&mut self, amount: u32) {
        self.scroll(Direction::Forward, amount);
    }

    pub fn scroll_left(&mut self) {
        self.scroll(Direction::Backward, 1);
    }

    pub fn scroll_left_by(&mut self, amount: u32) {
        self.scroll(Direction::Backward, amount);
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    fn clamp(&mut self, children: usize, parent: usize) {
        let children = children as u32;
        let parent = parent as u32;

        if children < parent {
            let clamp = parent.saturating_sub(children);
            self.offset = self.offset.saturating_sub(clamp);
        }
    }
}

impl Widget for Viewport {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        let attributes = ctx.attribs.get(id);
        let axis = attributes.get(AXIS).unwrap_or(Axis::Vertical);

        if let Some(width) = attributes.get("width") {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attributes.get("height") {
            constraints.make_height_tight(height);
        }

        let output_size: Size = (constraints.max_width(), constraints.max_height()).into();

        match axis {
            Axis::Horizontal => constraints.unbound_width(),
            Axis::Vertical => constraints.unbound_height(),
        }

        self.direction = attributes.get(DIRECTION).unwrap_or_default();

        // Make `unconstrained` an enum instead of a `bool`
        let unconstrained = true;
        let mut many = Many::new(self.direction, axis, self.offset, unconstrained);

        let mut size = many.layout(children, constraints, ctx);
        self.inner_size = many.used_size.inner_size();

        match axis {
            Axis::Vertical => size.height = output_size.height,
            Axis::Horizontal => size.width = output_size.width,
        }

        size
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
        let axis = attributes.get(AXIS).unwrap_or(Axis::Vertical);
        let mut pos = ctx.pos;

        // If the value is clamped, update the offset
        match axis {
            Axis::Horizontal => {
                self.clamp(self.inner_size.width, ctx.inner_size.width);
            }
            Axis::Vertical => {
                self.clamp(self.inner_size.height, ctx.inner_size.height);
            }
        };

        if let Direction::Backward = direction {
            match axis {
                Axis::Horizontal => pos.x += ctx.inner_size.width as i32,
                Axis::Vertical => pos.y += ctx.inner_size.height as i32,
            }
        }

        let offset = match direction {
            Direction::Forward => -(self.offset as i32),
            Direction::Backward => self.offset as i32,
        };

        match axis {
            Axis::Horizontal => pos.x += offset,
            Axis::Vertical => pos.y += offset,
        }

        children.for_each(|node, children| {
            match direction {
                Direction::Forward => {
                    node.position(children, pos, attribute_storage);
                    match axis {
                        Axis::Horizontal => pos.x += node.size().width as i32,
                        Axis::Vertical => pos.y += node.size().height as i32,
                    }
                }
                Direction::Backward => {
                    match axis {
                        Axis::Horizontal => pos.x -= node.size().width as i32,
                        Axis::Vertical => pos.y -= node.size().height as i32,
                    }
                    node.position(children, pos, attribute_storage);
                }
            }

            ControlFlow::Continue(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: anathema_widgets::PaintChildren<'_, '_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
        text: &mut StringSession<'_>,
    ) {
        let region = ctx.create_region();
        children.for_each(|widget, children| {
            let mut ctx = ctx.to_unsized();
            ctx.set_clip_region(region);
            widget.paint(children, ctx, text, attribute_storage);
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;
    use crate::Viewport;

    #[test]
    fn viewport() {
        let tpl = "
    viewport
        for i in [0, 1, 2]
            border
                text i
";

        let expected_first = "
    ╔═══╗
    ║┌─┐║
    ║│0│║
    ║└─┘║
    ║┌─┐║
    ║│1│║
    ║└─┘║
    ╚═══╝
";

        let expected_second = "
    ╔═══╗
    ║│0│║
    ║└─┘║
    ║┌─┐║
    ║│1│║
    ║└─┘║
    ║┌─┐║
    ╚═══╝
";

        TestRunner::new(tpl, (3, 6))
            .instance()
            .render_assert(expected_first)
            .with_widget(|query| {
                query.by_tag("viewport").first(|el, _| {
                    let viewport = el.to::<Viewport>().unwrap();
                    viewport.scroll_down();
                });
            })
            .render_assert(expected_second);
    }
}
