use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Constraints, ElementId, WidgetAttributes};
use unicode_width::UnicodeWidthStr;

use crate::string::SegString;
use crate::textlayout::{Instruction, PerformLayout, TextLayout};

pub struct Text {}

impl Text {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Text {
    fn layout<'bp>(
        &mut self,
        _: ElementId,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layouts: &mut Layouts,
        mut constraints: Constraints,
    ) -> LayoutSize {
        let Some(text) = attributes.value_as::<&str>() else { return LayoutSize::ZERO };

        let mut size = Size::ZERO;
        let mut layout = SegString::empty();
        layout.push(text, attributes);

        for mut child in children {
            let Some(text) = child.attributes.value_as::<&str>() else { continue };
            layout.push(text, child.attributes);
        }

        for line in layout.lines(constraints.max) {
            let mut width = 0;
            for (string, _attribs) in line {
                width += string.width() as u32;
            }
            size.width = size.width.max(width);
            size.height += 1;
        }

        LayoutSize::same(size)
    }

    fn position<'bp>(
        &mut self,
        _: ElementId,
        _: Children<'_, 'bp>,
        _: WidgetAttributes<'_, 'bp>,
        _: &mut Layouts,
        _: Pos,
    ) {
    }

    fn paint<'bp>(
        &mut self,
        id: ElementId,
        region: Region,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layouts: &Layouts,
    ) {
        let mut text_offset = region.from;
        let mut string = SegString::empty();

        let attributes = std::iter::once(attributes).chain(children.map(|c| c.attributes));
        for attrs in attributes {
            let Some(text) = attrs.value_as::<&str>() else { return };
            string.push(text, attrs);
        }

        let layout = layouts[id];
        let size = layout.outer_size();

        for (y_offset, line) in string.lines(size).enumerate() {
            let mut start = text_offset + Pos::new(0, y_offset as i32);

            for (text, attribs) in line {
                let end = Pos::new(start.x + text.width() as i32, start.y + 1);
                let region = Region::new(start, end);
                frontend.apply_brush_to_region(&attribs, region);
                frontend.set_text(text, start);
                start.x = end.x;
            }

            start.y += 1;
        }
    }

    fn describe(&self) -> &str {
        "text"
    }
}

pub struct Span {}

impl Span {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Span {
    fn layout<'bp>(
        &mut self,
        _: ElementId,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        constraints: Constraints,
    ) -> LayoutSize {
        todo!()
    }

    fn position<'bp>(
        &mut self,
        _: ElementId,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        pos: Pos,
    ) {
        todo!()
    }

    fn paint<'bp>(
        &mut self,
        _: ElementId,
        region: Region,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        todo!()
    }

    fn describe(&self) -> &'static str {
        "span"
    }
}
