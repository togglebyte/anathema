use crate::widgets::Widget;

pub(crate) struct Root;

impl Widget for Root {
    fn layout<'bp>(
        &mut self,
        _: crate::ElementId,
        children: &super::Children<'_, 'bp>,
        _: &crate::WidgetAttributes<'_, 'bp>,
        layouts: &mut super::Layouts,
        constraints: crate::Constraints,
    ) -> super::LayoutSize {
        match children.iter().next().map(|mut child| child.layout(layouts, constraints)) {
            Some(size) => super::LayoutSize::same(size),
            None => super::LayoutSize::ZERO,
        }
    }

    fn position<'bp>(
        &mut self,
        _: crate::ElementId,
        children: &super::Children<'_, 'bp>,
        _: &crate::WidgetAttributes<'_, 'bp>,
        layouts: &mut super::Layouts,
        pos: anathema_geometry::Pos,
    ) {
        let Some(mut child) = children.iter().next() else { return };
        child.position(layouts, pos);
    }

    fn paint<'bp>(
        &mut self,
        _: crate::ElementId,
        _: anathema_geometry::Region,
        mut children: super::Children<'_, 'bp>,
        _: crate::WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn anathema_frontend::Frontend,
        layouts: &super::Layouts,
    ) {
        let Some(child) = children.iter().next() else { return };
        child.paint(frontend, layouts);
    }
}
