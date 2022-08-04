use crate::ctx::{LayoutCtx, PositionCtx};
use crate::widgets::spacer::Spacer;
use crate::widgets::WidgetContainer;
use display::Size;

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx) -> Size {
    let _max_height = ctx.constraints.max_height;

    let mut height = 0;
    let mut width = 0;

    for widget in widgets.iter_mut() {
        // Ignore spacers
        if widget.kind() == Spacer::KIND {
            continue;
        }

        let size = widget.layout(ctx.constraints, ctx.force_layout);

        width = width.max(size.width);
        height = height.max(size.height);
    }

    Size::new(width, height)
}

pub fn position(widgets: &mut [WidgetContainer], ctx: PositionCtx) {
    for widget in widgets {
        widget.position(ctx.pos);
    }
}
