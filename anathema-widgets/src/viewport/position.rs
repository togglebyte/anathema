use crate::{PositionCtx, WidgetContainer, Direction};

pub(super) struct Position {
    direction: Direction,
}

impl Position {
    pub(super) fn new(direction: Direction) -> Self {
        Self {
            direction,
        }
    }

    pub(super) fn position(&self, ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
        match self.direction {
            Direction::Forward => self.bottom_to_top(ctx, children),
            Direction::Backward => self.top_to_bottom(ctx, children),
        }
    }

    fn bottom_to_top(&self, ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
        let mut pos = ctx.padded_position();
        for widget in children {
            widget.position(pos);
            pos.y += widget.size.height as i32;
        }
    }

    fn top_to_bottom(&self, ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
        let mut pos = ctx.padded_position();
        pos.y += ctx.size.height as i32;
        for widget in children {
            pos.y -= widget.size.height as i32;
            widget.position(pos);
        }
    }

}
