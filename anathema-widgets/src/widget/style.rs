use std::str::FromStr;

use anathema_state::{Color, Hex};
use anathema_value_resolver::ValueKind;

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

    pub fn reset_attributes(&mut self) {
        self.attributes = Attributes::NOT_UNDERLINED
            | Attributes::NOT_CROSSED_OUT
            | Attributes::NOT_OVERLINED
            | Attributes::NOT_REVERSED
            | Attributes::NOT_ITALIC;
    }

    /// Create an instance of `Style` from `CellAttributes`.
    pub fn from_cell_attribs(attributes: &anathema_value_resolver::Attributes<'_>) -> Self {
        let mut style = Self::new();

        if let Some(color) = attributes.get("foreground") {
            let color = match color {
                ValueKind::Hex(Hex { r, g, b }) => Color::from((*r, *g, *b)),
                ValueKind::Str(cow) => Color::from_str(cow.as_ref()).unwrap_or(Color::Reset),
                ValueKind::Int(ansi) => Color::AnsiVal(*ansi as u8),
                ValueKind::Color(c) => *c,
                _ => Color::Reset,
            };

            style.fg = Some(color)
        }

        if let Some(color) = attributes.get("background") {
            let color = match color {
                ValueKind::Color(color) => *color,
                ValueKind::Hex(Hex { r, g, b }) => Color::from((*r, *g, *b)),
                ValueKind::Str(cow) => Color::from_str(cow.as_ref()).unwrap_or(Color::Reset),
                ValueKind::Int(ansi) => Color::AnsiVal(*ansi as u8),
                _ => Color::Reset,
            };

            style.bg = Some(color)
        }

        if let Some(true) = attributes.get_as::<bool>("bold") {
            style.attributes |= Attributes::BOLD;
        } else if let Some(false) = attributes.get_as::<bool>("bold") {
            style.attributes |= Attributes::NORMAL;
        }

        if let Some(true) = attributes.get_as::<bool>("dim") {
            style.attributes |= Attributes::DIM;
        } else if let Some(false) = attributes.get_as::<bool>("dim") {
            style.attributes |= Attributes::NOT_DIM;
        }

        if let Some(true) = attributes.get_as::<bool>("italic") {
            style.attributes |= Attributes::ITALIC;
        } else if let Some(false) = attributes.get_as::<bool>("italic") {
            style.attributes |= Attributes::NOT_ITALIC;
        }

        if let Some(true) = attributes.get_as::<bool>("underline") {
            style.attributes |= Attributes::UNDERLINED;
        } else if let Some(false) = attributes.get_as::<bool>("underline") {
            style.attributes |= Attributes::NOT_UNDERLINED;
        }

        if let Some(true) = attributes.get_as::<bool>("crossed_out") {
            style.attributes |= Attributes::CROSSED_OUT;
        } else if let Some(false) = attributes.get_as::<bool>("crossed_out") {
            style.attributes |= Attributes::NOT_CROSSED_OUT;
        }

        if let Some(true) = attributes.get_as::<bool>("overline") {
            style.attributes |= Attributes::OVERLINED;
        } else if let Some(false) = attributes.get_as::<bool>("overline") {
            style.attributes |= Attributes::NOT_OVERLINED;
        }

        if let Some(true) = attributes.get_as::<bool>("inverse") {
            style.attributes |= Attributes::REVERSED;
        } else if let Some(false) = attributes.get_as::<bool>("inverse") {
            style.attributes |= Attributes::NOT_REVERSED;
        }

        style
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
    pub fn set_reversed(&mut self, inverse: bool) {
        if inverse {
            self.attributes |= Attributes::REVERSED;
        } else {
            self.attributes &= !Attributes::REVERSED;
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

bitflags::bitflags! {
    /// Style attributes
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Attributes: u16 {
        // Turn style on
        /// Make the characters bold
        const BOLD =            0b0000_0000_0000_0001;
        /// Make the characters dim
        const DIM =             0b0000_0000_0000_0010;
        /// Make the characters italic
        const ITALIC =          0b0000_0000_0000_0100;
        /// Make the characters underlined
        const UNDERLINED =      0b0000_0000_0000_1000;
        /// Make the characters crossed out
        const CROSSED_OUT =     0b0000_0000_0001_0000;
        /// Make the characters overlined
        const OVERLINED =       0b0000_0000_0010_0000;
        /// Make the characters inverse
        const REVERSED =        0b0000_0000_0100_0000;

        // Turn style off
        /// Make the characters not bold
        const NORMAL =          0b0000_0000_1000_0000;
        /// Make the characters not dim
        const NOT_DIM =         0b0000_0001_0000_0000;
        /// Make the characters not italic
        const NOT_ITALIC =      0b0000_0010_0000_0000;
        /// Make the characters not underlined
        const NOT_UNDERLINED =  0b0000_0100_0000_0000;
        /// Make the characters not crossed out
        const NOT_CROSSED_OUT = 0b0000_1000_0000_0000;
        /// Make the characters not overlined
        const NOT_OVERLINED =   0b0001_0000_0000_0000;
        /// Make the characters not inverse
        const NOT_REVERSED =    0b0010_0000_0000_0000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merging_styles() {
        let mut right = Style::new();
        right.set_fg(Color::Green);
        right.set_bg(Color::Blue);

        let mut left = Style::new();
        left.set_fg(Color::Red);

        left.merge(right);

        assert_eq!(left.fg.unwrap(), Color::Red);
        assert_eq!(left.bg.unwrap(), Color::Blue);
    }
}
