use display::Size;

use super::{expanded, spacers};
use crate::ctx::{LayoutCtx, PositionCtx};
use crate::layout::{Constraints, Padding};
use crate::widgets::expanded::Expand;
use crate::widgets::spacer::Spacer;
use crate::widgets::{Axis, WidgetContainer};

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx) -> Size {
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

    let expanded_size = expanded::layout(
        widgets,
        LayoutCtx::new(
            Constraints::new(max_width - used_width, ctx.constraints.max_height),
            ctx.force_layout,
            Padding::ZERO,
        ),
        Axis::Horizontal,
    );

    height = height.max(expanded_size.height);

    let spacers_size =
        spacers::layout(widgets, Constraints::new(max_width - used_width, height), ctx.force_layout, Axis::Horizontal);

    Size::new(used_width + expanded_size.width + spacers_size.width, height) + ctx.padding_size()
}

pub fn position(widgets: &mut [WidgetContainer], ctx: PositionCtx) {
    let mut widget_pos = ctx.padded_position();

    for widget in widgets {
        widget.position(widget_pos);
        widget_pos.x += widget.size.width as i32;
    }
}
