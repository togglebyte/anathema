use crate::display::Size;

use super::{expanded, spacers};
use super::{Constraints, Padding};
use crate::widgets::ctx::{LayoutCtx, PositionCtx};
use crate::widgets::{Direction, Expand, Spacer, WidgetContainer};

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx, ignore_spacers_and_expansions: bool) -> Size {
    let mut used_height = 0;
    let mut width = 0;

    let constraints = ctx.padded_constraints();
    let max_height = constraints.max_height;

    for widget in widgets.iter_mut() {
        // Ignore spacers
        if widget.kind() == Spacer::KIND {
            continue;
        }

        // Ignore expanded
        if widget.kind() == Expand::KIND {
            continue;
        }

        let constraints = Constraints::new(constraints.max_width, max_height - used_height);
        let size = widget.layout(constraints, ctx.force_layout);

        width = width.max(size.width);
        used_height = (used_height + size.height).min(max_height);
    }

    let expanded_size = match ignore_spacers_and_expansions {
        false => expanded::layout(
            widgets,
            LayoutCtx::new(
                Constraints::new(ctx.constraints.max_width, max_height - used_height),
                ctx.force_layout,
                Padding::ZERO,
            ),
            Direction::Vertical,
        ),
        true => Size::ZERO,
    };

    width = width.max(expanded_size.width).max(ctx.constraints.min_width) + ctx.padding.left + ctx.padding.right;
    width = width.min(ctx.constraints.max_width);

    let spacers_size = spacers::layout(
        widgets,
        Constraints::new(width, max_height - used_height),
        ctx.force_layout,
        Direction::Vertical,
    );

    let mut height = used_height + expanded_size.height + spacers_size.height;
    height = height.max(ctx.constraints.min_height).min(ctx.constraints.max_height);

    Size::new(width, height.min(ctx.constraints.max_height)) + ctx.padding_size()
}

pub fn position(widgets: &mut [WidgetContainer], mut ctx: PositionCtx) {
    for widget in widgets {
        widget.position(ctx.padded_position());
        ctx.pos.y += widget.size.height as i32;
    }
}
