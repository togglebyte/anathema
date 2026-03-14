use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Constraints, WidgetAttributes};

struct PaddingValues {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

impl PaddingValues {
    fn new(attributes: WidgetAttributes<'_, '_>) -> Self {
        let padding = attributes.get_as::<u32>("padding").unwrap_or(0);
        let left = attributes.get_as::<u32>("left").unwrap_or(padding);
        let right = attributes.get_as::<u32>("right").unwrap_or(padding);
        let top = attributes.get_as::<u32>("top").unwrap_or(padding);
        let bottom = attributes.get_as::<u32>("bottom").unwrap_or(padding);
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    fn size(&self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    fn top_left(&self) -> Pos {
        Pos::new(self.left as i32, self.top as i32)
    }
}

pub struct Padding;

impl Widget for Padding {
    fn layout<'bp>(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        mut constraints: Constraints,
    ) -> LayoutSize {
        let pad = PaddingValues::new(attributes);
        let padding_size = pad.size();

        crate::update_constraints(attributes, &mut constraints);

        let inner = children
            .next()
            .map(|child| child.layout(layout, constraints - padding_size))
            .unwrap_or(Size::ZERO);

        let outer = crate::fix_size(inner + padding_size, attributes, constraints);

        LayoutSize::new(inner, outer)
    }

    fn position<'bp>(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        pos: Pos,
    ) {
        let pad = PaddingValues::new(attributes);
        if let Some(child) = children.next() {
            child.position(layout, pos + pad.top_left());
        }
    }

    fn paint<'bp>(
        &mut self,
        region: Region,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        frontend.apply_brush_to_region(&attributes, region);

        if let Some(child) = children.next() {
            child.paint(frontend, layout);
        }
    }

    fn describe(&self) -> &str {
        "padding"
    }
}
