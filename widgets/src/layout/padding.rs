use crate::Padding;

pub fn layout(ctx: LayoutCtx, padding: Padding, child: Option<&mut WidgetContainer>) -> Size {
    let constraints = Constraints::new(
        ctx.constraints.max_width.saturating_sub(padding.right + padding.left),
        ctx.constraints.max_height.saturating_sub(padding.top + padding.bottom),
    );

    match child {
        Some(child) => {
            let mut size = child.layout(LayoutCtx::new(constraints, ctx.force_layout));
            size.width += padding.right + padding.left;
            size.height += padding.top + padding.bottom;
            size
        }
        None => Size::new(padding.left + padding.right, padding.top + padding.bottom),
    }
}
