use std::ops::ControlFlow;

use anathema_geometry::{LocalPos, Size};
use anathema_state::CommonVal;
use anathema_value_resolver::{AttributeStorage, ValueKind};
use anathema_widgets::layout::text::{ProcessResult, Segment, Strings};
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{Glyphs, PaintCtx, SizePos};
use anathema_widgets::{ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::{LEFT, RIGHT};

pub(crate) const WRAP: &str = "wrap";
pub(crate) const TEXT_ALIGN: &str = "text_align";

/// Text alignment aligns the text inside its parent.
///
/// Given a border with a width of nine and text alignment set to [`TextAlignment::Right`]:
/// ```text
/// ┌───────┐
/// │I would│
/// │ like a│
/// │ lovely│
/// │ cup of│
/// │    tea│
/// │ please│
/// └───────┘
/// ```
///
/// The text will only align it self within the parent widget.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum TextAlignment {
    /// Align the to the left inside the parent
    #[default]
    Left,
    /// Align the text in the centre of the parent
    Centre,
    /// Align the to the right inside the parent
    Right,
}

impl TryFrom<&ValueKind<'_>> for TextAlignment {
    type Error = ();

    fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
        let s = value.as_str().ok_or(())?;
        match s {
            LEFT => Ok(TextAlignment::Left),
            RIGHT => Ok(TextAlignment::Right),
            "centre" | "center" => Ok(TextAlignment::Centre),
            _ => Err(()),
        }
    }
}

/// Text widget
/// ```ignore
/// Attributes:
/// * background
/// * foreground
/// * text-align
/// * wrap
/// ```
///
/// Note: Spans, unlike other widgets, does not require a widget id
///
/// A `Text` widget will be as wide as its text.
#[derive(Debug, Default)]
pub struct Text {
    strings: Strings,
}

impl Widget for Text {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attributes = ctx.attributes(id);
        let wrap = attributes.get_as(WRAP).unwrap_or_default();
        let size = constraints.max_size();
        self.strings = Strings::new(size, wrap);
        self.strings.set_style(id);

        // // Layout text
        attributes.value().map(|text| {
            text.strings(&mut |s| match self.strings.add_str(s) {
                ProcessResult::Break => false,
                ProcessResult::Continue => true,
            });
        });

        // Layout text of all the sub-nodes
        children.each(ctx, |ctx, child, _| {
            let Some(_span) = child.try_to_ref::<Span>() else {
                return ControlFlow::Continue(());
            };
            self.strings.set_style(child.id());

            let attributes = ctx.attributes(child.id());
            if let Some(text) = attributes.value() {
                text.strings(&mut |s| match self.strings.add_str(s) {
                    ProcessResult::Break => false,
                    ProcessResult::Continue => true,
                });
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        });

        self.strings.finish()
    }

    fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        let lines = self.strings.lines();
        let alignment = attribute_storage.get(id).get_as(TEXT_ALIGN).unwrap_or_default();

        let mut pos = LocalPos::ZERO;
        let mut style = attribute_storage.get(id);

        for line in lines {
            let x = match alignment {
                TextAlignment::Left => 0,
                TextAlignment::Centre => ctx.local_size.width as u16 / 2 - line.width / 2,
                TextAlignment::Right => ctx.local_size.width as u16 - line.width,
            };

            pos.x = x;

            for entry in line.entries {
                match entry {
                    Segment::Str(s) => {
                        let glyphs = Glyphs::new(s);
                        if let Some(new_pos) = ctx.place_glyphs(glyphs, pos) {
                            // TODO: 
                            // In the future there should probably be a way to
                            // provide both style and glyph at the same time.
                            for x in pos.x..new_pos.x {
                                ctx.set_style(style, (x, pos.y).into());
                            }
                            pos = new_pos;
                        }
                    }
                    Segment::SetStyle(attribute_id) => style = attribute_storage.get(attribute_id),
                }
            }
            pos.y += 1;
            pos.x = 0;
        }
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, 'bp>,
        _attributes: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        _ctx: PositionCtx,
    ) {
        // NOTE
        // No positioning is done in here, it's all done when painting
    }
}

#[derive(Default, Copy, Clone)]
pub struct Span;

impl Widget for Span {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        // Everything is handled by the parent text
        panic!("this should never be called");
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PositionCtx,
    ) {
        // Everything is handled by the parent text
        panic!("this should never be called");
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn word_wrap_excessive_space() {
        let src = "text 'hello      how are     you'";
        let expected = "
           ╔════════════════╗
           ║hello      how  ║
           ║are     you     ║
           ║                ║
           ║                ║
           ║                ║
           ║                ║
           ╚════════════════╝";

        TestRunner::new(src, (16, 6)).instance().render_assert(expected);
    }

    #[test]
    fn word_wrap() {
        let src = "text 'hello how are you'";
        let expected = r#"
           ╔════════════════╗
           ║hello how are   ║
           ║you             ║
           ║                ║
           ╚════════════════╝
           "#;

        TestRunner::new(src, (16, 3)).instance().render_assert(expected);
    }

    #[test]
    fn break_word_wrap() {
        let src = "text [wrap: 'break'] 'hello howareyoudoing'";
        let expected = r#"
           ╔════════════════╗
           ║hello howareyoud║
           ║oing            ║
           ║                ║
           ╚════════════════╝
           "#;

        TestRunner::new(src, (16, 3)).instance().render_assert(expected);
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let src = r#"
            text 'one'
                span 'two'
                span ' averylongword'
                span ' bunny'
        "#;

        let expected = r#"
           ╔═══════════════════╗
           ║onetwo             ║
           ║averylongword bunny║
           ║                   ║
           ╚═══════════════════╝
       "#;

        TestRunner::new(src, (19, 3)).instance().render_assert(expected);
    }

    #[test]
    fn multi_line_with_span() {
        let src = r#"
            border [width: 5 + 2]
                text 'one'
                    span 'two'
        "#;

        let expected = r#"
            ╔═════════╗
            ║┌─────┐  ║
            ║│onetw│  ║
            ║│o    │  ║
            ║└─────┘  ║
            ╚═════════╝
       "#;

        TestRunner::new(src, (9, 4)).instance().render_assert(expected);
    }

    #[test]
    fn right_alignment() {
        let src = "text [text_align: 'right'] 'a one xxxxxxxxxxxxxxxxxx'";
        let expected = r#"
               ╔══════════════════╗
               ║            a one ║
               ║xxxxxxxxxxxxxxxxxx║
               ║                  ║
               ╚══════════════════╝
           "#;

        TestRunner::new(src, (18, 3)).instance().render_assert(expected);
    }

    #[test]
    fn centre_alignment() {
        let src = "text [text_align: 'centre'] 'a one xxxxxxxxxxxxxxxxxx'";
        let expected = r#"
               ╔══════════════════╗
               ║      a one       ║
               ║xxxxxxxxxxxxxxxxxx║
               ║                  ║
               ╚══════════════════╝
           "#;

        TestRunner::new(src, (18, 3)).instance().render_assert(expected);
    }

    #[test]
    fn line_break() {
        let src = "text 'What have you'";
        let expected = r#"
               ╔═════════╗
               ║What have║
               ║you      ║
               ║         ║
               ╚═════════╝
           "#;

        TestRunner::new(src, (9, 3)).instance().render_assert(expected);
    }
}
