use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region, Size};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::layout::many::Many;
use crate::layout::{AXIS, Axis, DIRECTION, Direction};
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
        children: LayoutForEach<'_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let attributes = ctx.attribute_storage.get(id);
        let axis = attributes.get_as(AXIS).unwrap_or(Axis::Vertical);
        let offset = attributes.get_as::<usize>("offest").unwrap_or_default();

        let output_size: Size = (constraints.max_width(), constraints.max_height()).into();

        match axis {
            Axis::Horizontal => constraints.unbound_width(),
            Axis::Vertical => constraints.unbound_height(),
        }

        if attributes.get_as::<bool>(UNCONSTRAINED).unwrap_or_default() {
            constraints.unbound_width();
            constraints.unbound_height();
        }

        if let Some(width) = attributes.get_as::<u16>(WIDTH) {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attributes.get_as::<u16>(HEIGHT) {
            constraints.make_height_tight(height);
        }

        self.direction = attributes.get_as(DIRECTION).unwrap_or_default();

        // Make `unconstrained` an enum instead of a `bool`
        let unconstrained = true;
        let mut many = Many::new(self.direction, axis, unconstrained);

        // NOTE: we use the inner size here from many.layout
        _ = many.layout(children, constraints, ctx, offset)?;

        self.inner_size = many.used_size.inner_size();

        Ok(output_size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        let attributes = attribute_storage.get(id);
        let direction = attributes.get_as(DIRECTION).unwrap_or_default();
        let axis = attributes.get_as(AXIS).unwrap_or(Axis::Vertical);
        let mut pos = ctx.pos;

        // If the value is clamped, update the offset
        match attributes.get_as::<bool>(CLAMP).unwrap_or_default() {
            false => (),
            true => self.clamp(self.inner_size, ctx.inner_size),
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

        let mut count = 0;
        _ = children.each(|node, children| {
            // TODO
            // ----
            // this should stop doing layout once the children are no longer
            // visible. Take the offset into consideration to skip widgets as well
            //
            // This should be done on this type if possible `PositionChildren<'_, '_, 'bp>`,
            // so all widgets can benefit from this.
            match direction {
                Direction::Forward => {
                    let region = Region::from((pos, node.size()));
                    let self_region = ctx.region();
                    let intersects = self_region.intersects(&region);
                    // awful_debug!(
                    //     "reg: {intersects:?} | self: f: {} y: {} | child f: {} t: {}",
                    //     self_region.from.y,
                    //     self_region.to.y,
                    //     region.from.y,
                    //     region.to.y
                    // );

                    match intersects {
                        true => node.position(children, pos, attribute_storage, ctx.viewport),
                        false => {}
                    }

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

            count += 1;
            ControlFlow::Continue(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        let region = ctx.create_region();

        _ = children.each(|widget, children| {
            ctx.set_clip_region(region);
            let ctx = ctx.to_unsized();
            widget.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }

    fn needs_reflow(&mut self) -> bool {
        let needs_reflow = self.is_dirty;
        self.is_dirty = false;
        needs_reflow
    }
}

#[cfg(test)]
mod test {

    use crate::Overflow;
    use crate::testing::TestRunner;

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
