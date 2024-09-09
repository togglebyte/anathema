use std::fmt;
use std::str::FromStr;

use crate::{CommonVal, Hex, State};

pub trait FromColor {
    fn from_color<T>(color: Color) -> T;
}

/// Representation of terminal colors, following the ANSI spec
///
/// [ANSI color table](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Grey,
    DarkGrey,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    /// 24bit color, expressed as rgb, following the spec
    ///
    /// See [24bit colors](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit)
    Rgb(u8, u8, u8),
    /// 8bit color.
    ///
    /// See [256 colors](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit)
    AnsiVal(u8),
}

impl State for Color {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Color(*self))
    }
}

impl From<Hex> for Color {
    fn from(value: Hex) -> Self {
        Self::Rgb(value.r, value.g, value.b)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::Rgb(r, g, b)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reset => write!(f, "Reset"),
            Self::Black => write!(f, "Black"),
            Self::Red => write!(f, "Red"),
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Blue => write!(f, "Blue"),
            Self::Magenta => write!(f, "Magenta"),
            Self::Cyan => write!(f, "Cyan"),
            Self::Grey => write!(f, "Grey"),
            Self::DarkGrey => write!(f, "DarkGrey"),
            Self::LightRed => write!(f, "LightRed"),
            Self::LightGreen => write!(f, "LightGreen"),
            Self::LightYellow => write!(f, "LightYellow"),
            Self::LightBlue => write!(f, "LightBlue"),
            Self::LightMagenta => write!(f, "LightMagenta"),
            Self::LightCyan => write!(f, "LightCyan"),
            Self::White => write!(f, "White"),
            Self::Rgb(r, g, b) => write!(f, "#{r:02X}{g:02X}{b:02X}"),
            Self::AnsiVal(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ColorParseError;

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse color")
    }
}

impl std::error::Error for ColorParseError {}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        let color = match s.as_ref() {
            "reset" => Self::Reset,
            "black" => Self::Black,
            "red" => Self::Red,
            "green" => Self::Green,
            "yellow" => Self::Yellow,
            "blue" => Self::Blue,
            "magenta" => Self::Magenta,
            "cyan" => Self::Cyan,
            "grey" => Self::Grey,
            "dark_grey" => Self::DarkGrey,
            "light_red" => Self::LightRed,
            "light_green" => Self::LightGreen,
            "light_yellow" => Self::LightYellow,
            "light_blue" => Self::LightBlue,
            "light_magenta" => Self::LightMagenta,
            "light_cyan" => Self::LightCyan,
            "white" => Self::White,
            _ => {
                if let Ok(ansi_value) = s.parse::<u8>() {
                    Self::AnsiVal(ansi_value)
                } else if let Ok(hex) = Hex::try_from(s.as_ref()) {
                    Self::from(hex)
                } else {
                    return Err(ColorParseError);
                }
            }
        };

        Ok(color)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_hex_str() {
        let color = Color::from_str("#FF0000").unwrap();
        assert_eq!(color, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn from_ansi_index() {
        let color = Color::from_str("10").unwrap();
        assert_eq!(color, Color::AnsiVal(10));
    }

    #[test]
    fn from_ansi_color() {
        assert_eq!(Color::from_str("reset").unwrap(), Color::Reset);
        assert_eq!(Color::from_str("black").unwrap(), Color::Black);
        assert_eq!(Color::from_str("red").unwrap(), Color::Red);
        assert_eq!(Color::from_str("green").unwrap(), Color::Green);
        assert_eq!(Color::from_str("yellow").unwrap(), Color::Yellow);
        assert_eq!(Color::from_str("blue").unwrap(), Color::Blue);
        assert_eq!(Color::from_str("magenta").unwrap(), Color::Magenta);
        assert_eq!(Color::from_str("cyan").unwrap(), Color::Cyan);
        assert_eq!(Color::from_str("grey").unwrap(), Color::Grey);
        assert_eq!(Color::from_str("dark_grey").unwrap(), Color::DarkGrey);
        assert_eq!(Color::from_str("light_red").unwrap(), Color::LightRed);
        assert_eq!(Color::from_str("light_green").unwrap(), Color::LightGreen);
        assert_eq!(Color::from_str("light_yellow").unwrap(), Color::LightYellow);
        assert_eq!(Color::from_str("light_blue").unwrap(), Color::LightBlue);
        assert_eq!(Color::from_str("light_magenta").unwrap(), Color::LightMagenta);
        assert_eq!(Color::from_str("light_cyan").unwrap(), Color::LightCyan);
        assert_eq!(Color::from_str("white").unwrap(), Color::White);
        assert_eq!(Color::from_str("#242424").unwrap(), Color::Rgb(36, 36, 36));
        assert_eq!(Color::from_str("10").unwrap(), Color::AnsiVal(10));
    }

    #[test]
    fn to_string() {
        assert_eq!(Color::from_str("#242424").unwrap().to_string(), "#242424");
        assert_eq!(Color::from_str("10").unwrap().to_string(), "10");
    }
}
