use std::ops::ControlFlow;

use anathema_geometry::{Pos, Size};
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::layout::many::Many;
use crate::layout::{Axis, Direction, AXIS, DIRECTION};
use crate::{HEIGHT, WIDTH};

const UNCONSTRAINED: &str = "unconstrained";
const CLAMP: &str = "clamp";

#[derive(Debug, Default)]
pub struct Overflow {
    offset: Pos,
    // The size of the children since the last layout call
    inner_size: Size,

    direction: Direction,
    is_dirty: bool,
}

impl Overflow {
    pub fn scroll(&mut self, direction: Direction, amount: Pos) {
        self.is_dirty = true;

        match (self.direction, direction) {
            (Direction::Forward, Direction::Forward) => self.offset += amount,
            (Direction::Forward, Direction::Backward) => self.offset -= amount,
            (Direction::Backward, Direction::Backward) => self.offset += amount,
            (Direction::Backward, Direction::Forward) => self.offset -= amount,
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll(Direction::Backward, Pos { x: 0, y: 1 });
    }

    pub fn scroll_up_by(&mut self, amount: i32) {
        self.scroll(Direction::Backward, Pos { x: 0, y: amount });
    }

    pub fn scroll_down(&mut self) {
        self.scroll(Direction::Forward, Pos { x: 0, y: 1 });
    }

    pub fn scroll_down_by(&mut self, amount: i32) {
        self.scroll(Direction::Forward, Pos { x: 0, y: amount });
    }

    pub fn scroll_right(&mut self) {
        self.scroll(Direction::Forward, Pos { x: 1, y: 0 });
    }

    pub fn scroll_right_by(&mut self, amount: i32) {
        self.scroll(Direction::Forward, Pos { x: amount, y: 0 });
    }

    pub fn scroll_left(&mut self) {
        self.scroll(Direction::Backward, Pos { x: 1, y: 0 });
    }

    pub fn scroll_left_by(&mut self, amount: i32) {
        self.scroll(Direction::Backward, Pos { x: amount, y: 0 });
    }

    pub fn scroll_to(&mut self, pos: Pos) {
        self.offset = pos;
    }

    pub fn offset(&self) -> Pos {
        self.offset
    }

    fn clamp(&mut self, children: Size, parent: Size) {
        if self.offset.x < 0 {
            self.offset.x = 0;
        }

        if self.offset.y < 0 {
            self.offset.y = 0;
        }

        if children.height <= parent.height {
            self.offset.y = 0;
        } else {
            let max_y = children.height as i32 - parent.height as i32;
            if self.offset.y > max_y {
                self.offset.y = max_y;
            }
        }

        if children.width <= parent.width {
            self.offset.x = 0;
        } else {
            let max_x = children.width as i32 - parent.width as i32;
            if self.offset.x > max_x {
                self.offset.x = max_x
            }
        }
    }
}

impl Widget for Overflow {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attributes = ctx.attribs.get(id);
        let axis = attributes.get(AXIS).unwrap_or(Axis::Vertical);

        let output_size: Size = (constraints.max_width(), constraints.max_height()).into();

        match axis {
            Axis::Horizontal => constraints.unbound_width(),
            Axis::Vertical => constraints.unbound_height(),
        }

        if attributes.get_bool(UNCONSTRAINED) {
            constraints.unbound_width();
            constraints.unbound_height();
        }

        if let Some(width) = attributes.get_usize(WIDTH) {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attributes.get_usize(HEIGHT) {
            constraints.make_height_tight(height);
        }

        self.direction = attributes.get(DIRECTION).unwrap_or_default();

        // Make `unconstrained` an enum instead of a `bool`
        let unconstrained = true;
        let mut many = Many::new(self.direction, axis, unconstrained);

        let _size = many.layout(children, constraints, ctx);

        self.inner_size = many.used_size.inner_size();

        output_size
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
        match attributes.get(CLAMP) {
            Some(false) => {}
            _ => self.clamp(self.inner_size, ctx.inner_size),
        }

        if let Direction::Backward = direction {
            match axis {
                Axis::Horizontal => pos.x += ctx.inner_size.width as i32,
                Axis::Vertical => pos.y += ctx.inner_size.height as i32,
            }
        }

        let mut pos = match direction {
            Direction::Forward => pos - self.offset,
            Direction::Backward => pos + self.offset,
        };

        children.for_each(|node, children| {
            match direction {
                Direction::Forward => {
                    node.position(children, pos, attribute_storage, ctx.viewport);
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
                    node.position(children, pos, attribute_storage, ctx.viewport);
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
    ) {
        let region = ctx.create_region();
        children.for_each(|widget, children| {
            ctx.set_clip_region(region);
            let ctx = ctx.to_unsized();
            widget.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }

    fn needs_reflow(&self) -> bool {
        self.is_dirty
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;
    use crate::Overflow;

    #[test]
    fn overflow() {
        let tpl = "
    overflow
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
            .with_widget(|mut query| {
                query.by_tag("overflow").first(|el, _| {
                    let overflow = el.to::<Overflow>();
                    overflow.scroll_down();
                });
            })
            .render_assert(expected_second);
    }

    #[test]
    fn clamp_prevents_scrolling() {
        let tpl = "
    overflow
        text '0'";

        let expected_first = "
    ╔═══╗
    ║0  ║
    ║   ║
    ╚═══╝
";

        TestRunner::new(tpl, (3, 2))
            .instance()
            .with_widget(|mut query| {
                query.by_tag("overflow").first(|el, _| {
                    let overflow = el.to::<Overflow>();
                    overflow.scroll_down();
                });
            })
            .render_assert(expected_first);
    }
}
