use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Size};
use anathema_runtime::widgets::{Children, Layout, Widget};
use anathema_runtime::{Attributes, TemplateValue};

#[derive(Debug)]
pub struct Border<'bp> {
    width: TemplateValue<'bp>,
    height: TemplateValue<'bp>,
    min_width: TemplateValue<'bp>,
    min_height: TemplateValue<'bp>,
    foreground: TemplateValue<'bp>,
    background: TemplateValue<'bp>,
}

impl<'bp> Border<'bp> {
    pub(crate) fn new(attrs: &Attributes<'bp>) -> Self {
        Self {
            width: attrs.get("width").clone(),
            height: attrs.get("height").clone(),
            min_width: attrs.get("min_width").clone(),
            min_height: attrs.get("min_height").clone(),
            foreground: attrs.get("foreground").clone(),
            background: attrs.get("background").clone(),
        }
    }
}

impl<'bp> Widget<'bp> for Border<'bp> {
    fn layout(&mut self, mut children: Children<'_, '_>, layout: &mut Layout) -> Size {
        let mut size = children.next().map(|child| child.layout(layout)).unwrap_or(Size::ZERO);
        size
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self, children: Children<'_, '_>, frontend: &mut dyn Frontend) {
        // let foreground = attributes.get("foreground");
        // let background = attributes.get("background");
        // let width = attributes.get("width");
        // let height = attributes.get("height");
        // let min_width = attributes.get("min_width");
        // let min_height = attributes.get("min_height");

        todo!()
    }

    fn describe(&self) -> &str {
        "border"
    }
}

pub fn border<'bp>(attr: &Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp> {
    Box::new(Border::new(attr))
}
