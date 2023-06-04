use antstring::AntString;
use unicode_width::UnicodeWidthStr;

use display::{Size, Style};

use crate::attributes::{fields, Attributes};
use crate::{Wrap, DEFAULT_TAB_SIZE};
use crate::layout::text::TextLayout;
use crate::{LocalPos, Pos};

use super::{LayoutCtx, NodeId, PaintCtx, Widget, WidgetContainer, WithSize};

struct Row(String);

pub struct Console;

impl Widget for Console {
    fn kind(&self) -> &'static str {
        "Console"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn needs_layout(&mut self) -> bool {
        true
    }

    fn needs_paint(&self) -> bool {
        true
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
    }

    fn position(&mut self, _: Pos, _: Size) {}

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        vec![]
    }

    fn add_child(&mut self, _: WidgetContainer) {}

    fn remove_child(&mut self, _: &NodeId) -> Option<WidgetContainer> {
        None
    }

    fn update(&mut self, attributes: Attributes) {}
}
