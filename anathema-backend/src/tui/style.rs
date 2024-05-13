use std::io::{Result, Write};

use anathema_state::Hex;
use anathema_widgets::Attributes as Attribs;
pub use crossterm::style::Attribute as CrossAttrib;
use crossterm::style::{Color, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::QueueableCommand;

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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Style {
    /// Foreground colour.
    pub fg: Option<Color>,
    /// Background colour.
    pub bg: Option<Color>,
    /// Attributes.
    pub attributes: Attributes,
}

impl Style {
    /// Create a new instance of a `Style`:
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            attributes: Attributes::empty(),
        }
    }

    pub(crate) fn write(&self, w: &mut impl Write) -> Result<()> {
        if let Some(fg) = self.fg {
            w.queue(SetForegroundColor(fg))?;
        }

        if let Some(bg) = self.bg {
            w.queue(SetBackgroundColor(bg))?;
        }

        // Dim and bold are a special case, as they are both
        // reset through `NormalIntensity` (22).
        // This means the reset has to happen before setting
        // bold or dim
        if !self.attributes.contains(Attributes::BOLD | Attributes::DIM) {
            w.queue(SetAttribute(CrossAttrib::NormalIntensity))?;
        }

        if self.attributes.contains(Attributes::BOLD) {
            w.queue(SetAttribute(CrossAttrib::Bold))?;
        }

        if self.attributes.contains(Attributes::DIM) {
            w.queue(SetAttribute(CrossAttrib::Dim))?;
        }

        if self.attributes.contains(Attributes::ITALIC) {
            w.queue(SetAttribute(CrossAttrib::Italic))?;
        } else {
            w.queue(SetAttribute(CrossAttrib::NoItalic))?;
        }

        if self.attributes.contains(Attributes::UNDERLINED) {
            w.queue(SetAttribute(CrossAttrib::Underlined))?;
        } else {
            w.queue(SetAttribute(CrossAttrib::NoUnderline))?;
        }

        if self.attributes.contains(Attributes::OVERLINED) {
            w.queue(SetAttribute(CrossAttrib::OverLined))?;
        } else {
            w.queue(SetAttribute(CrossAttrib::NotOverLined))?;
        }

        if self.attributes.contains(Attributes::CROSSED_OUT) {
            w.queue(SetAttribute(CrossAttrib::CrossedOut))?;
        } else {
            w.queue(SetAttribute(CrossAttrib::NotCrossedOut))?;
        }

        if self.attributes.contains(Attributes::INVERSE) {
            w.queue(SetAttribute(CrossAttrib::Reverse))?;
        } else {
            w.queue(SetAttribute(CrossAttrib::NoReverse))?;
        }

        Ok(())
    }

    /// Set the foreground colour
    pub fn set_fg(&mut self, fg: Color) {
        self.fg = Some(fg);
    }

    /// Set the background colour
    pub fn set_bg(&mut self, bg: Color) {
        self.bg = Some(bg);
    }

    /// Set the style to bold
    pub fn set_bold(&mut self, bold: bool) {
        if bold {
            self.attributes |= Attributes::BOLD;
        } else {
            self.attributes &= !Attributes::BOLD;
        }
    }

    /// Set the style to italic
    pub fn set_italic(&mut self, italic: bool) {
        if italic {
            self.attributes |= Attributes::ITALIC;
        } else {
            self.attributes &= !Attributes::ITALIC;
        }
    }

    /// Make the cell dim
    pub fn set_dim(&mut self, dim: bool) {
        if dim {
            self.attributes |= Attributes::DIM;
        } else {
            self.attributes &= !Attributes::DIM;
        }
    }

    /// Make the cell underlined as long as it's supported
    pub fn set_underlined(&mut self, underlined: bool) {
        if underlined {
            self.attributes |= Attributes::UNDERLINED;
        } else {
            self.attributes &= !Attributes::UNDERLINED;
        }
    }

    /// Make the cell overlined as long as it's supported
    pub fn set_overlined(&mut self, overlined: bool) {
        if overlined {
            self.attributes |= Attributes::OVERLINED;
        } else {
            self.attributes &= !Attributes::OVERLINED;
        }
    }

    /// Make the cell crossed out as long as it's supported
    pub fn set_crossed_out(&mut self, crossed_out: bool) {
        if crossed_out {
            self.attributes |= Attributes::CROSSED_OUT;
        } else {
            self.attributes &= !Attributes::CROSSED_OUT;
        }
    }

    /// Invert the foreground and background
    pub fn set_inverse(&mut self, inverse: bool) {
        if inverse {
            self.attributes |= Attributes::INVERSE;
        } else {
            self.attributes &= !Attributes::INVERSE;
        }
    }

    /// Reset the style
    pub fn reset() -> Self {
        let mut style = Self::new();
        style.fg = Some(Color::Reset);
        style.bg = Some(Color::Reset);
        style
    }

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

impl<'bp> From<&Attribs<'bp>> for Style {
    fn from(attributes: &Attribs<'bp>) -> Self {
        let mut style = Self::new();

        if let Some(fg) = attributes.get_val("foreground").and_then(|val| val.load_common_val()) {
            if let Some(val) = fg.to_common() {
                let val = *val;
                let colour = match val.to_hex() {
                    None => Color::try_from(val.to_common_str().as_ref()).ok(),
                    Some(Hex { r, g, b }) => Some(Color::from((r, g, b))),
                };
                style.fg = colour;
            }
        }

        if let Some(bg) = attributes.get_val("background").and_then(|val| val.load_common_val()) {
            if let Some(val) = bg.to_common() {
                let val = *val;
                let colour = match val.to_hex() {
                    None => Color::try_from(val.to_common_str().as_ref()).ok(),
                    Some(Hex { r, g, b }) => Some(Color::from((r, g, b))),
                };
                style.bg = colour;
            }
        }

        if attributes.get_bool("bold") {
            style.attributes |= Attributes::BOLD;
        }

        if attributes.get_bool("dim") {
            style.attributes |= Attributes::DIM;
        }

        if attributes.get_bool("italic") {
            style.attributes |= Attributes::ITALIC;
        }

        if attributes.get_bool("underline") {
            style.attributes |= Attributes::UNDERLINED;
        }

        if attributes.get_bool("crossed-out") {
            style.attributes |= Attributes::CROSSED_OUT;
        }

        if attributes.get_bool("overline") {
            style.attributes |= Attributes::OVERLINED;
        }

        if attributes.get_bool("inverse") {
            style.attributes |= Attributes::INVERSE;
        }

        style
    }
}

bitflags::bitflags! {
    /// Style attributes
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Attributes: u8 {
        /// Make the characters bold (in supported output)
        const BOLD =        0b0000_0001;
        /// Make the characters dim (in supported output)
        const DIM =         0b0000_0010;
        /// Make the characters italic (in supported output)
        const ITALIC =      0b0000_0100;
        /// Make the characters underlined (in supported output)
        const UNDERLINED =  0b0000_1000;
        /// Make the characters crossed out (in supported output)
        const CROSSED_OUT = 0b0001_0000;
        /// Make the characters overlined (in supported output)
        const OVERLINED =   0b0010_0000;
        /// Make the characters inverse (in supported output)
        const INVERSE =     0b0100_0000;
    }
}
