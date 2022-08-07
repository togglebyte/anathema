use crate::display::Size;
use crate::widgets::ctx::{LayoutCtx, PositionCtx};
use crate::widgets::{Spacer, WidgetContainer};

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx) -> Size {
    let constraints = ctx.padded_constraints();

    let mut height = 0;
    let mut width = 0;

    for widget in widgets.iter_mut() {
        // Ignore spacers
        if widget.kind() == Spacer::KIND {
            continue;
        }

        let size = widget.layout(constraints, ctx.force_layout);

        width = width.max(size.width);
        height = height.max(size.height);
    }

    Size::new(width, height)
}

pub fn position(widgets: &mut [WidgetContainer], ctx: PositionCtx) {
    for widget in widgets {
        widget.position(ctx.padded_position());
    }
}
