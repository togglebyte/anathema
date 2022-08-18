use crate::display::Size;

use super::{expanded, spacers};
use super::{Constraints, Padding};
use crate::widgets::ctx::{LayoutCtx, PositionCtx};
use crate::widgets::{Direction, Expand, Spacer, WidgetContainer};

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx, ignore_spacers_and_expansions: bool) -> Size {
    let mut used_width = 0;
    let mut height = 0;

    let constraints = ctx.padded_constraints();
    let max_width = constraints.max_width;

    for widget in widgets.iter_mut() {
        // Ignore spacers
        if widget.kind() == Spacer::KIND {
            continue;
        }

        // Ignore expanded widgets
        if widget.kind() == Expand::KIND {
            continue;
        }

        let constraints = Constraints::new(max_width - used_width, constraints.max_height);
        let size = widget.layout(constraints, ctx.force_layout);

        height = height.max(size.height);
        used_width = (used_width + size.width).min(max_width);
    }

    let expanded_size = match ignore_spacers_and_expansions {
        false => expanded::layout(
            widgets,
            LayoutCtx::new(
                Constraints::new(max_width - used_width, ctx.constraints.max_height),
                ctx.force_layout,
                Padding::ZERO,
            ),
            Direction::Horizontal,
        ),
        true => Size::ZERO,
    };

    height = height.max(expanded_size.height) + ctx.padding.top + ctx.padding.bottom;
    height = height.min(ctx.constraints.max_height);

    let spacers_size = spacers::layout(
        widgets,
        Constraints::new(max_width - used_width, height),
        ctx.force_layout,
        Direction::Horizontal,
    );

    let mut width = used_width + expanded_size.width + spacers_size.width;
    width = width.max(ctx.constraints.min_width).min(ctx.constraints.max_width);

    Size::new(width, height)
}

pub fn position(widgets: &mut [WidgetContainer], ctx: PositionCtx) {
    let mut pos = ctx.padded_position();
    for widget in widgets {
        widget.position(pos);
        pos.x += widget.size.width as i32;
    }
}
