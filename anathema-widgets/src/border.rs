use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Size};
use anathema_runtime::widgets::{Children, Layout, Widget};
use anathema_runtime::{Attributes, TemplateValue, WidgetAttributes};

#[derive(Debug, Default)]
pub struct Border {
    border_style: (),
}

impl Border {
    pub(crate) fn new<'bp>(attrs: &Attributes<'bp>) -> Self {
        Self {
            border_style: panic!("border style should be set on layout, not here"),
        }
    }
}

impl<'bp> Widget<'bp> for Border {
    fn layout(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layout,
    ) -> Size {
        let mut size = children.next().map(|child| child.layout(layout)).unwrap_or(Size::ZERO);
        size
    }

    fn position(&mut self, mut children: Children<'_, '_>, attributes: WidgetAttributes<'_, 'bp>, pos: Pos) {
        if let Some(child) = children.next() {
            child.position(pos);
        }
    }

    fn paint(
        &mut self,
        mut children: Children<'_, '_>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
    ) {
        // let foreground = attributes.get("foreground");
        // let background = attributes.get("background");
        // let width = attributes.get("width");
        // let height = attributes.get("height");
        // let min_width = attributes.get("min_width");
        // let min_height = attributes.get("min_height");

        if let Some(child) = children.next() {
            child.paint(frontend);
        }

        todo!()
    }

    fn describe(&self) -> &str {
        "border"
    }
}

pub fn border<'bp>(attr: &Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp> {
    Box::new(Border::new(attr))
}
