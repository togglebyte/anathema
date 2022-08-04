use std::collections::HashMap;

use display::Color;

use super::{Number, Value};
use crate::widgets::Display;
use crate::{Align, Axis, BorderStyle, Sides, Wrap};

#[cfg(feature = "serde-json")]
impl From<serde_json::Value> for Value {
    fn from(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Empty,
            serde_json::Value::Bool(value) => Value::Bool(value),
            serde_json::Value::Number(number) => {
                match number.as_f64() {
                    Some(number) => Value::Number(Number::Float(number)),
                    None => match number.as_i64() {
                        Some(number) => Value::Number(Number::Signed(number)),
                        // If a number isn't f64 or i64 then it has to be u64,
                        // see https://github.com/serde-rs/json/blob/5d2cbcdd4b146e98b5aa2200de7a8ae6231bf0ba/src/number.rs#L22-L34
                        None => Value::Number(Number::Unsigned(
                            number
                                .as_u64()
                                .expect("can't fail as it's either f64, i64 or u64"),
                        )),
                    },
                }
            }
            serde_json::Value::String(value) if value.contains(|c: char| c.is_whitespace()) => {
                Value::String(value)
            }
            serde_json::Value::String(value) => value_from_json_string(value),
            serde_json::Value::Array(values) => Value::List(
                values
                    .into_iter()
                    .filter_map(|v| v.try_into().ok())
                    .collect(),
            ),
            serde_json::Value::Object(json_values) => {
                let mut values = HashMap::<_, Value>::new();

                for (k, v) in json_values {
                    values.insert(k, v.into());
                }

                Value::Map(values)
            }
        }
    }
}

#[cfg(feature = "serde-json")]
fn value_from_json_string(s: String) -> Value {
    // Try Colour
    if let Some(color) = colour_from_str(&s) {
        return Value::Color(color);
    }

    // Try alignment
    if let Some(align) = alignment_from_str(&s) {
        return Value::Alignment(align);
    }

    // Try axis
    if let Some(axis) = axis_from_str(&s) {
        return Value::Axis(axis);
    }

    // Try border style
    if let Some(bs) = border_style_from_str(&s) {
        return Value::BorderStyle(bs);
    }

    // Try display
    if let Some(disp) = display_from_str(&s) {
        return Value::Display(disp);
    }

    // Try sides
    if let Some(sides) = sides_from_str(&s) {
        return Value::Sides(sides);
    }

    // Try wrap
    if let Some(wrap) = wrap_from_str(&s) {
        return Value::Wrap(wrap);
    }

    Value::String(s)
}

fn axis_from_str(s: &str) -> Option<Axis> {
    match s {
        "horz" => Some(Axis::Horizontal),
        "horizontal" => Some(Axis::Horizontal),
        "vert" => Some(Axis::Vertical),
        "vertical" => Some(Axis::Vertical),
        _ => None,
    }
}

fn alignment_from_str(s: &str) -> Option<Align> {
    match s {
        "top" => Some(Align::Top),
        "top-right" => Some(Align::TopRight),
        "right" => Some(Align::Right),
        "bottom-right" => Some(Align::BottomRight),
        "bottom" => Some(Align::Bottom),
        "bottom-left" => Some(Align::BottomLeft),
        "left" => Some(Align::Left),
        "top-left" => Some(Align::TopLeft),
        "centre" => Some(Align::Centre),
        _ => None,
    }
}

fn colour_from_str(s: &str) -> Option<Color> {
    if s.starts_with('#') && [4, 7].contains(&s.len()) {
        let hex = &s[1..];
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;

            let r = r << 4 | r;
            let g = g << 4 | g;
            let b = b << 4 | b;
            return Some(Color::Rgb { r, g, b });
        } else {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb { r, g, b });
        }
    }

    match s {
        "black" => Some(Color::Black),
        "blue" => Some(Color::Blue),
        "cyan" => Some(Color::Cyan),
        "dark-blue" => Some(Color::DarkBlue),
        "dark-cyan" => Some(Color::DarkCyan),
        "dark-green" => Some(Color::DarkGreen),
        "dark-grey" => Some(Color::DarkGrey),
        "dark-magenta" => Some(Color::DarkMagenta),
        "dark-red" => Some(Color::DarkRed),
        "dark-yellow" => Some(Color::DarkYellow),
        "green" => Some(Color::Green),
        "grey" => Some(Color::Grey),
        "magenta" => Some(Color::Magenta),
        "red" => Some(Color::Red),
        "reset" => Some(Color::Reset),
        "white" => Some(Color::White),
        "yellow" => Some(Color::Yellow),
        _ => None,
    }
}

fn border_style_from_str(s: &str) -> Option<BorderStyle> {
    match s {
        "thin" => Some(BorderStyle::Thin),
        "thick" => Some(BorderStyle::Thick),
        _ => None,
    }
}

fn display_from_str(s: &str) -> Option<Display> {
    match s {
        "show" => Some(Display::Show),
        "hide" => Some(Display::Hide),
        "exclude" => Some(Display::Exclude),
        _ => None,
    }
}

fn sides_from_str(s: &str) -> Option<Sides> {
    let input = s.split('|').map(|s| s.trim()).collect::<Vec<_>>();
    if input.is_empty() {
        return None;
    }

    let mut sides = Sides::EMPTY;

    for _side in input {
        match s {
            "left" => sides |= Sides::LEFT,
            "right" => sides |= Sides::RIGHT,
            "top" => sides |= Sides::TOP,
            "bottom" => sides |= Sides::BOTTOM,
            _ => return None,
        }
    }

    Some(sides)
}

fn wrap_from_str(s: &str) -> Option<Wrap> {
    match s {
        "no-wrap" => Some(Wrap::NoWrap),
        "word" => Some(Wrap::Word),
        "break" => Some(Wrap::Break),
        _ => None,
    }
}

#[cfg(test)]
#[cfg(feature = "serde-json")]
mod json_test {
    use super::*;

    #[test]
    fn colour() {
        let inputs = vec![
            (
                "#00FF00",
                Value::Color(Color::Rgb {
                    r: 0,
                    g: u8::MAX,
                    b: 0,
                }),
            ),
            (
                "#00ff00",
                Value::Color(Color::Rgb {
                    r: 0,
                    g: u8::MAX,
                    b: 0,
                }),
            ),
            ("red", Value::Color(Color::Red)),
            ("blue", Value::Color(Color::Blue)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn alignment() {
        let inputs = vec![
            (Align::Top.to_string(), Value::Alignment(Align::Top)),
            (
                Align::TopRight.to_string(),
                Value::Alignment(Align::TopRight),
            ),
            (Align::Right.to_string(), Value::Alignment(Align::Right)),
            (
                Align::BottomRight.to_string(),
                Value::Alignment(Align::BottomRight),
            ),
            (Align::Bottom.to_string(), Value::Alignment(Align::Bottom)),
            (
                Align::BottomLeft.to_string(),
                Value::Alignment(Align::BottomLeft),
            ),
            (Align::Left.to_string(), Value::Alignment(Align::Left)),
            (Align::TopLeft.to_string(), Value::Alignment(Align::TopLeft)),
            (Align::Centre.to_string(), Value::Alignment(Align::Centre)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn axis() {
        let inputs = vec![
            ("horz", Value::Axis(Axis::Horizontal)),
            ("horizontal", Value::Axis(Axis::Horizontal)),
            ("vert", Value::Axis(Axis::Vertical)),
            ("vertical", Value::Axis(Axis::Vertical)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn border() {
        let inputs = vec![
            ("thin", Value::BorderStyle(BorderStyle::Thin)),
            ("thick", Value::BorderStyle(BorderStyle::Thick)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn display() {
        let inputs = vec![
            ("show", Value::Display(Display::Show)),
            ("hide", Value::Display(Display::Hide)),
            ("exclude", Value::Display(Display::Exclude)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn wrap() {
        let inputs = vec![
            ("no-wrap", Value::Wrap(Wrap::NoWrap)),
            ("break", Value::Wrap(Wrap::Break)),
            ("word", Value::Wrap(Wrap::Word)),
        ];

        for (input, expected) in inputs {
            let actual = value_from_json_string(input.to_string());
            assert_eq!(actual, expected);
        }
    }
}
