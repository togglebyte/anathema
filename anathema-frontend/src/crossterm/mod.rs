use anathema_compiler::Color;
use compact_str::CompactString;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::crossterm::attributes::Attributes;
use crate::screenbuffer::Screen;

mod attributes;
pub(crate) mod buffer;
pub mod screen;

/// The style for a cell in a [`crate::Buffer`]
/// A style is applied to ever single cell in a [`crate::Buffer`].
///
/// Styles do not cascade (and don't behave like CSS).
/// So giving a style to a parent widget does not automatically apply it to the child.
///
/// The following template would draw a red border with white text inside:
///
/// ```text
/// border [foreground: red]:
///     text: "hi"
/// ```
///
/// In the following example, if the condition is ever true, and then false the text `is_false`
/// will be rendered with a red foreground.
///
/// The way to reset the foreground is to apply `Color::Reset` to the text.
///
/// ```text
/// if [cond: {{ is_true }}]:
///     text [foreground: red]: "is true"
/// else:
///     text: "is false"
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Style {
    /// Foreground colour.
    pub fg: Option<Color>,
    /// Background colour.
    pub bg: Option<Color>,
    /// Attributes.
    pub attributes: Attributes,
}

impl Style {
    /// Merge two styles:
    /// if `self` has no foreground the foreground from the other style is copied to self.
    /// if `self` has no background the background from the other style is copied to self.
    pub fn merge(&mut self, other: Style) {
        if let (None, Some(fg)) = (self.fg, other.fg) {
            self.fg = Some(fg);
        }

        if let (None, Some(bg)) = (self.bg, other.bg) {
            self.bg = Some(bg);
        }

        self.attributes |= other.attributes;
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            attributes: Attributes::NORMAL,
        }
    }
}

impl From<&dyn crate::Brush> for Style {
    fn from(e: &dyn crate::Brush) -> Self {
        panic!()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
enum State {
    #[default]
    Empty,
    Continuation,
    Char(char),
    Cluster(CompactString),
}

impl State {
    fn width(&self) -> usize {
        match self {
            Self::Empty | Self::Continuation => 1,
            Self::Char(c) => c.width().expect("zero width chars are never written to the buffer"),
            Self::Cluster(cluster) => cluster.width(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Cell {
    style: Style,
    state: State,
}
