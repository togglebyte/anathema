use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Constraints, ElementId, WidgetAttributes};

pub struct Container;

impl Widget for Container {
    fn layout<'bp>(
        &mut self,
        _: ElementId,
        children: &Children<'_, 'bp>,
        attributes: &WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        mut constraints: Constraints,
    ) -> LayoutSize {
        crate::update_constraints(attributes, &mut constraints);
        let size = children
            .first()
            .map(|mut child| child.layout(layout, constraints))
            .unwrap_or(constraints.min);

        LayoutSize::same(size)
    }

    fn position<'bp>(
        &mut self,
        _: ElementId,
        children: &Children<'_, 'bp>,
        attributes: &WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        pos: Pos,
    ) {
        if let Some(mut child) = children.first() {
            child.position(layout, pos);
        }
    }

    fn paint<'bp>(
        &mut self,
        _: ElementId,
        region: Region,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        frontend.apply_brush_to_region(&attributes, region);

        if let Some(child) = children.first() {
            child.paint(frontend, layout);
        }
    }

    fn describe(&self) -> &str {
        "container"
    }
}
