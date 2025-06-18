use std::io::{Result, Write};

use anathema_state::Color;
use anathema_widgets::{Attributes, Style};
use crossterm::QueueableCommand;
pub use crossterm::style::{Attribute as CrossAttrib, Color as CTColor};
use crossterm::style::{SetAttribute, SetBackgroundColor, SetForegroundColor};

struct ColorWrapper(Color);

impl From<ColorWrapper> for CTColor {
    fn from(color: ColorWrapper) -> CTColor {
        match color.0 {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::DarkRed,
            Color::Green => Self::DarkGreen,
            Color::Yellow => Self::DarkYellow,
            Color::Blue => Self::DarkBlue,
            Color::Magenta => Self::DarkMagenta,
            Color::Cyan => Self::DarkCyan,
            Color::Grey => Self::Grey,
            Color::DarkGrey => Self::DarkGrey,
            Color::LightRed => Self::Red,
            Color::LightGreen => Self::Green,
            Color::LightYellow => Self::Yellow,
            Color::LightBlue => Self::Blue,
            Color::LightMagenta => Self::Magenta,
            Color::LightCyan => Self::Cyan,
            Color::White => Self::White,
            Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
            Color::AnsiVal(v) => Self::AnsiValue(v),
        }
    }
}

pub(crate) fn write_style(style: &Style, w: &mut impl Write) -> Result<()> {
    if let Some(fg) = style.fg {
        w.queue(SetForegroundColor(ColorWrapper(fg).into()))?;
    }

    if let Some(bg) = style.bg {
        w.queue(SetBackgroundColor(ColorWrapper(bg).into()))?;
    }

    // Dim and bold are a special case, as they are both
    // reset through `NormalIntensity` (22).
    // This means the reset has to happen before setting
    // bold or dim
    if !style.attributes.contains(Attributes::BOLD | Attributes::DIM) {
        w.queue(SetAttribute(CrossAttrib::NormalIntensity))?;
    }

    if style.attributes.contains(Attributes::NORMAL) {
        w.queue(SetAttribute(CrossAttrib::NormalIntensity))?;
    } else if style.attributes.contains(Attributes::BOLD) {
        w.queue(SetAttribute(CrossAttrib::Bold))?;
    }

    if style.attributes.contains(Attributes::DIM) {
        w.queue(SetAttribute(CrossAttrib::Dim))?;
    }

    if style.attributes.contains(Attributes::NOT_ITALIC) {
        w.queue(SetAttribute(CrossAttrib::NoItalic))?;
    } else if style.attributes.contains(Attributes::ITALIC) {
        w.queue(SetAttribute(CrossAttrib::Italic))?;
    }

    if style.attributes.contains(Attributes::NOT_UNDERLINED) {
        w.queue(SetAttribute(CrossAttrib::NoUnderline))?;
    } else if style.attributes.contains(Attributes::UNDERLINED) {
        w.queue(SetAttribute(CrossAttrib::Underlined))?;
    }

    if style.attributes.contains(Attributes::NOT_OVERLINED) {
        w.queue(SetAttribute(CrossAttrib::NotOverLined))?;
    } else if style.attributes.contains(Attributes::OVERLINED) {
        w.queue(SetAttribute(CrossAttrib::OverLined))?;
    }

    if style.attributes.contains(Attributes::NOT_CROSSED_OUT) {
        w.queue(SetAttribute(CrossAttrib::NotCrossedOut))?;
    } else if style.attributes.contains(Attributes::CROSSED_OUT) {
        w.queue(SetAttribute(CrossAttrib::CrossedOut))?;
    }

    if style.attributes.contains(Attributes::NOT_REVERSED) {
        w.queue(SetAttribute(CrossAttrib::NoReverse))?;
    } else if style.attributes.contains(Attributes::REVERSED) {
        w.queue(SetAttribute(CrossAttrib::Reverse))?;
    }

    Ok(())
}
