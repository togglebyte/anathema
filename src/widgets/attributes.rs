use std::collections::HashMap;
use std::time::Duration;

use crate::display::{Color, Style};

use super::value::Path;
use super::value::{Easing, Value};
use super::{Align, Axis, BorderStyle, Display, NodeId, Padding, Sides, TextAlignment, Wrap};

// -----------------------------------------------------------------------------
//     - Attribute names -
// -----------------------------------------------------------------------------
pub mod fields {
    pub const ANIMATE: &str = "animate";
    pub const ALIGNMENT: &str = "align";
    pub const AUTO_SCROLL: &str = "auto-scroll";
    pub const AXIS: &str = "axis";
    pub const BACKGROUND: &str = "background";
    pub const BINDING: &str = "binding";
    pub const BORDER_CHARS: &str = "border-chars";
    pub const BORDER_STYLE: &str = "border-style";
    pub const BOTTOM: &str = "bottom";
    pub const COLLAPSE_SPACES: &str = "collapse-spaces";
    pub const MAX_HEIGHT: &str = "max-height";
    pub const MAX_WIDTH: &str = "max-width";
    pub const MIN_HEIGHT: &str = "min-height";
    pub const MIN_WIDTH: &str = "min-width";
    pub const DATA: &str = "data";
    pub const DIRECTION: &str = "direction";
    pub const DISPLAY: &str = "display";
    pub const FACTOR: &str = "factor";
    pub const FILL: &str = "fill";
    pub const FOREGROUND: &str = "foreground";
    pub const HEIGHT: &str = "height";
    pub const ID: &str = "id";
    pub const LEFT: &str = "left";
    pub const MAX_CHILDREN: &str = "max-children";
    pub const NAME: &str = "name";
    pub const OFFSET: &str = "offset";
    pub const PADDING: &str = "padding";
    pub const PADDING_TOP: &str = "padding-top";
    pub const PADDING_RIGHT: &str = "padding-right";
    pub const PADDING_BOTTOM: &str = "padding-bottom";
    pub const PADDING_LEFT: &str = "padding-left";
    pub const POSITION: &str = "position";
    pub const REVERSE: &str = "reverse";
    pub const RIGHT: &str = "right";
    pub const SIDES: &str = "sides";
    pub const TAB_STOP: &str = "tab";
    pub const TEXT_ALIGN: &str = "text-align";
    pub const TOP: &str = "top";
    pub const TRIM_END: &str = "trim-end";
    pub const TRIM_START: &str = "trim-start";
    pub const WIDTH: &str = "width";
    pub const WRAP: &str = "wrap";
}

// -----------------------------------------------------------------------------
//     - Attributes -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Attributes {
    pub(crate) inner: HashMap<String, Value>,
}

impl From<Vec<Attribute<'_>>> for Attributes {
    fn from(attributes: Vec<Attribute<'_>>) -> Self {
        let mut inner = HashMap::new();

        for attr in attributes {
            inner.insert(attr.key.to_owned(), attr.val);
        }

        Self { inner }
    }
}

impl Attributes {
    pub fn empty() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn new(key: &str, value: impl Into<Value>) -> Self {
        let mut inner = Self::empty();
        inner.set(key, value.into());
        inner
    }

    /// Get all transitions except `Alignment`
    /// as alignment affects the child rather than self.
    pub fn transitions(&self) -> impl Iterator<Item = (&str, f32, Duration, Easing)> {
        self.inner.iter().filter_map(|(k, v)| match v {
            Value::Transition(val, duration, easing) if k != fields::ALIGNMENT => {
                val.to_signed_int().map(|val| (k.as_ref(), val as f32, *duration, *easing))
            }
            _ => None,
        })
    }

    pub fn has(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn diff(&self, old: &Attributes) -> Attributes {
        let mut new = HashMap::new();

        for (k, v) in &self.inner {
            match old.inner.get(k) {
                Some(old_val) if v.ne(old_val) => drop(new.insert(k.to_string(), v.clone())),
                Some(_) | None => continue,
            }
        }

        Attributes { inner: new }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.inner.get(name).cloned()
    }

    pub fn take_value(&mut self, name: &str) -> Option<Value> {
        self.inner.remove(name)
    }

    pub fn set(&mut self, name: impl AsRef<str>, value: impl Into<Value>) {
        self.inner.insert(name.as_ref().into(), value.into());
    }

    pub fn height(&self) -> Option<usize> {
        self.get_int(fields::HEIGHT).map(|i| i as usize)
    }

    pub fn width(&self) -> Option<usize> {
        self.get_int(fields::WIDTH).map(|i| i as usize)
    }

    pub fn min_width(&self) -> Option<usize> {
        self.get_int(fields::MIN_WIDTH).map(|i| i as usize)
    }

    pub fn min_height(&self) -> Option<usize> {
        self.get_int(fields::MIN_HEIGHT).map(|i| i as usize)
    }

    pub fn max_width(&self) -> Option<usize> {
        self.get_int(fields::MAX_WIDTH).map(|i| i as usize)
    }

    pub fn max_height(&self) -> Option<usize> {
        self.get_int(fields::MAX_HEIGHT).map(|i| i as usize)
    }

    pub fn offset(&self) -> i32 {
        self.get_int(fields::OFFSET).map(|i| i as i32).unwrap_or(0)
    }

    pub fn factor(&self) -> Option<usize> {
        self.get_int(fields::FACTOR).map(|i| i as usize)
    }

    pub fn fill(&self) -> Option<&str> {
        self.get_str(fields::FILL)
    }

    pub fn padding_all(&self) -> Option<Padding> {
        let left = self.padding_left();
        let right = self.padding_right();
        let top = self.padding_top();
        let bottom = self.padding_bottom();

        let padding = self.get_int(fields::PADDING).map(|p| p as usize);

        left.or_else(|| right.or_else(|| top.or_else(|| bottom.or(padding))))?;

        let padding = padding.unwrap_or(0);

        Some(Padding {
            left: left.unwrap_or(padding),
            right: right.unwrap_or(padding),
            top: top.unwrap_or(padding),
            bottom: bottom.unwrap_or(padding),
        })
    }

    pub fn padding(&self) -> Option<usize> {
        self.get_int(fields::PADDING).map(|p| p as usize)
    }

    pub fn padding_top(&self) -> Option<usize> {
        self.get_int(fields::PADDING_TOP).map(|i| i as usize).or_else(|| self.padding())
    }

    pub fn padding_right(&self) -> Option<usize> {
        self.get_int(fields::PADDING_RIGHT).map(|i| i as usize).or_else(|| self.padding())
    }

    pub fn padding_bottom(&self) -> Option<usize> {
        self.get_int(fields::PADDING_BOTTOM).map(|i| i as usize).or_else(|| self.padding())
    }

    pub fn padding_left(&self) -> Option<usize> {
        self.get_int(fields::PADDING_LEFT).map(|i| i as usize).or_else(|| self.padding())
    }

    pub fn left(&self) -> Option<i32> {
        self.get_signed_int(fields::LEFT).map(|i| i as i32)
    }

    pub fn right(&self) -> Option<i32> {
        self.get_signed_int(fields::RIGHT).map(|i| i as i32)
    }

    pub fn top(&self) -> Option<i32> {
        self.get_signed_int(fields::TOP).map(|i| i as i32)
    }

    pub fn bottom(&self) -> Option<i32> {
        self.get_signed_int(fields::BOTTOM).map(|i| i as i32)
    }

    pub fn reverse(&self) -> bool {
        self.get_bool(fields::REVERSE).unwrap_or(false)
    }

    pub fn axis(&self) -> Option<Axis> {
        match self.value(fields::AXIS) {
            Some(Value::Axis(val)) => Some(*val),
            None | Some(_) => None,
        }
    }

    pub fn direction(&self) -> Option<Axis> {
        match self.value(fields::DIRECTION) {
            Some(Value::Axis(val)) => Some(*val),
            None | Some(_) => None,
        }
    }

    pub fn alignment(&self) -> Option<Align> {
        match self.value(fields::ALIGNMENT) {
            Some(Value::Alignment(val)) => Some(*val),
            Some(Value::Transition(val, _, _)) => match val.as_ref() {
                Value::Alignment(ref val) => Some(*val),
                _ => None,
            },
            None | Some(_) => None,
        }
    }

    pub fn sides(&self) -> Sides {
        match self.value(fields::SIDES) {
            Some(Value::Sides(val)) => *val,
            None | Some(_) => Sides::ALL,
        }
    }

    pub fn text_alignment(&self) -> TextAlignment {
        match self.value(fields::TEXT_ALIGN) {
            Some(Value::TextAlignment(val)) => *val,
            None | Some(_) => TextAlignment::Left,
        }
    }

    pub fn background(&self) -> Option<Color> {
        self.get_color(fields::BACKGROUND)
    }

    pub fn foreground(&self) -> Option<Color> {
        self.get_color(fields::FOREGROUND)
    }

    pub fn max_children(&self) -> Option<usize> {
        self.get_int(fields::MAX_CHILDREN).map(|i| i as usize)
    }

    pub fn word_wrap(&self) -> Wrap {
        match self.value(fields::WRAP) {
            Some(Value::Wrap(wrap)) => *wrap,
            None | Some(_) => Wrap::Word,
        }
    }

    pub fn trim_start(&self) -> bool {
        self.get_bool(fields::TRIM_START).unwrap_or(true)
    }

    pub fn trim_end(&self) -> bool {
        self.get_bool(fields::TRIM_END).unwrap_or(true)
    }

    pub fn collapse_spaces(&self) -> bool {
        self.get_bool(fields::COLLAPSE_SPACES).unwrap_or(true)
    }

    pub fn auto_scroll(&self) -> bool {
        self.get_bool(fields::AUTO_SCROLL).unwrap_or(false)
    }

    pub fn style(&self) -> Style {
        let mut inst = Style::new();
        inst.fg = self.foreground();
        inst.bg = self.background();

        if self.get_bool("bold").unwrap_or(false) {
            inst.set_bold(true);
        }

        if self.get_bool("italic").unwrap_or(false) {
            inst.set_italic(true);
        }

        if self.get_bool("dim").unwrap_or(false) {
            inst.set_dim(true);
        }

        if self.get_bool("underlined").unwrap_or(false) {
            inst.set_underlined(true);
        }

        if self.get_bool("overlined").unwrap_or(false) {
            inst.set_overlined(true);
        }

        if self.get_bool("inverse").unwrap_or(false) {
            inst.set_inverse(true);
        }

        if self.get_bool("crossed-out").unwrap_or(false) {
            inst.set_crossed_out(true);
        }

        inst
    }

    pub fn update_style(&self, style: &mut Style) {
        if self.has(fields::FOREGROUND) {
            style.fg = self.foreground();
        }

        if self.has(fields::BACKGROUND) {
            style.bg = self.background();
        }

        if self.has("bold") {
            style.set_bold(self.get_bool("bold").unwrap_or(false));
        }

        if self.has("italic") {
            style.set_italic(self.get_bool("italic").unwrap_or(false));
        }

        if self.has("dim") {
            style.set_dim(self.get_bool("dim").unwrap_or(false));
        }

        if self.has("underlined") {
            style.set_underlined(self.get_bool("underlined").unwrap_or(false));
        }

        if self.has("inverse") {
            style.set_inverse(self.get_bool("inverse").unwrap_or(false));
        }

        if self.has("crossed-out") {
            style.set_crossed_out(self.get_bool("crossed-out").unwrap_or(false));
        }
    }

    pub fn take_style(&self) -> Attributes {
        let mut attributes = Attributes::empty();
        // let mut inst = Style::new();
        // inst.fg = self.foreground();
        // inst.bg = self.background();

        if let Some(color) = self.get_color("foreground") {
            attributes.set("foreground", color);
        }

        if let Some(color) = self.get_color("background") {
            attributes.set("background", color);
        }

        if let Some(true) = self.get_bool("bold") {
            attributes.set("bold", true);
        }

        if let Some(true) = self.get_bool("italic") {
            attributes.set("italic", true);
        }

        if let Some(true) = self.get_bool("dim") {
            attributes.set("dim", true);
        }

        if let Some(true) = self.get_bool("underlined") {
            attributes.set("underline", true);
        }

        if let Some(true) = self.get_bool("overlined") {
            attributes.set("overlined", true);
        }

        if let Some(true) = self.get_bool("inverse") {
            attributes.set("inverse", true);
        }

        if let Some(true) = self.get_bool("crossed-out") {
            attributes.set("crossed-out", true);
        }

        attributes
    }

    pub fn display(&self) -> Display {
        match self.value(fields::DISPLAY) {
            Some(Value::Display(display)) => *display,
            None | Some(_) => Display::Show,
        }
    }

    pub fn border_style(&self) -> &BorderStyle {
        match self.value(fields::BORDER_STYLE) {
            Some(Value::BorderStyle(style)) => style,
            None | Some(_) => &BorderStyle::Thin,
        }
    }

    pub fn id(&self) -> Option<NodeId> {
        self.value(fields::ID).map(|val| NodeId::Value(val.clone()))
    }

    pub fn value(&self, name: &str) -> Option<&Value> {
        self.inner.get(name)
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        let val = self.inner.get(name)?;
        val.to_bool()
    }

    pub fn get_int(&self, name: &str) -> Option<u64> {
        let val = self.inner.get(name)?;
        val.to_int()
    }

    pub fn get_signed_int(&self, name: &str) -> Option<i64> {
        let val = self.inner.get(name)?;
        val.to_signed_int()
    }

    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.inner.get(name)?.to_str()
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        self.inner.get(name)?.to_str().map(|s| s.to_string())
    }

    pub fn get_data(&self, name: &str) -> Option<&Path> {
        let val = self.inner.get(name)?;
        val.to_data_binding()
    }

    pub fn get_list(&self, name: &str) -> Option<&[Value]> {
        let val = self.inner.get(name)?;
        val.to_list()
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        match self.value(name) {
            Some(Value::Color(color)) => Some(*color),
            None | Some(_) => None,
        }
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = (&'a String, &'a Value);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

// -----------------------------------------------------------------------------
//     - Attribute -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Attribute<'src> {
    pub key: &'src str,
    pub val: Value,
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::template::{Lexer, Parser};
    // use crate::testing::root_from_template;
    // use crate::Size;
    // use proptest::prelude::*;

    // fn attribs(template: &str) -> Attributes {
    //     let lexer = Lexer::new(template);
    //     let mut parser = Parser::new(lexer);
    //     parser.next().unwrap().unwrap().attributes
    // }

    // #[test]
    // fn parse_quoted_value() {
    //     let mut attributes = attribs("container [attrib:\"with,commas,and space\"]:");
    //     assert_eq!(attributes.inner.len(), 1);

    //     let actual = attributes.get_value("attrib");
    //     let expected = Some(Value::String("with,commas,and space".into()));
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_value_bool() {
    //     let attributes = attribs("text [is_true:true]: \"\"");
    //     assert_eq!(attributes.inner.len(), 1);

    //     let attributes = attribs("row [is_true:true,   is_false   : false]:");
    //     assert_eq!(attributes.inner.len(), 2);

    //     let actual = attributes.value("is_true").unwrap();
    //     let expected = Value::Bool(true);
    //     assert_eq!(&expected, actual);

    //     let actual = attributes.value("is_false").unwrap();
    //     let expected = Value::Bool(false);
    //     assert_eq!(&expected, actual);
    // }

    // #[test]
    // fn empty_attributes() {
    //     let attributes = attribs("container []:");
    //     assert!(attributes.is_empty());

    //     let attributes = attribs("text: 'there are: \"no attributes\"'");
    //     assert!(attributes.is_empty());
    // }

    // #[test]
    // fn text_align() {
    //     let attributes = attribs("text [text-align: centre]: 'a bc'");
    //     let actual = attributes.value("text-align").unwrap();
    //     let expected = Value::TextAlignment(TextAlignment::Centre);
    //     assert_eq!(expected, *actual);
    // }

    // #[test]
    // fn colours() {
    //     let attributes =
    //         attribs("container [background: red, foreground: blue, col: green, res: reset, rgb: #0A0B0C]:");
    //     let background = attributes.background().unwrap();
    //     let foreground = attributes.foreground().unwrap();
    //     let green = attributes.value("col").unwrap().to_color().unwrap();
    //     let reset = attributes.value("res").unwrap().to_color().unwrap();
    //     let rgb = attributes.value("rgb").unwrap().to_color().unwrap();

    //     assert_eq!(background, Color::Red);
    //     assert_eq!(foreground, Color::Blue);
    //     assert_eq!(green, Color::Green);
    //     assert_eq!(reset, Color::Reset);
    //     assert_eq!(rgb, Color::Rgb { r: 10, g: 11, b: 12 });
    // }

    // #[test]
    // fn alignment() {
    //     let attributes = attribs("container [align: top-right]:");
    //     let expected = attributes.alignment().unwrap();
    //     let actual = Align::TopRight;
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn name() {
    //     let mut attributes = attribs("text [name: \"bob\"]:");
    //     let expected = attributes.name().unwrap();
    //     let actual = "bob";
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn axis() {
    //     let axis = [
    //         (Axis::Horizontal, "horz"),
    //         (Axis::Horizontal, "horizontal"),
    //         (Axis::Vertical, "vert"),
    //         (Axis::Vertical, "vertical"),
    //     ];

    //     for (val, text) in axis {
    //         let attributes = attribs(&format!("viewport [axis: {text}]:"));
    //         let expected = attributes.axis().unwrap();
    //         let actual = val;
    //         assert_eq!(expected, actual);
    //     }
    // }

    // #[test]
    // fn sides() {
    //     let sides = [
    //         (Sides::ALL, "all"),
    //         (Sides::LEFT, "left"),
    //         (Sides::TOP, "top"),
    //         (Sides::RIGHT, "right"),
    //         (Sides::BOTTOM, "bottom"),
    //         (Sides::LEFT | Sides::RIGHT, "left | right"),
    //         (Sides::TOP | Sides::BOTTOM, "top|bottom"),
    //         (Sides::TOP | Sides::LEFT | Sides::BOTTOM, "top | left | bottom"),
    //     ];

    //     for (val, text) in sides {
    //         let attributes = attribs(&format!("border [sides: {text}]:"));
    //         let expected = attributes.sides();
    //         let actual = val;
    //         assert_eq!(expected, actual);
    //     }
    // }

    // #[test]
    // fn text_with_no_attributes() {
    //     let template = r#"text: "[no attribute here]""#;
    //     let attributes = attribs(template);
    //     assert!(attributes.is_empty());
    // }

    // #[test]
    // fn display() {
    //     let actual = attribs("container [display: exclude]:").display();
    //     let expected = Display::Exclude;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("container [display: hide]:").display();
    //     let expected = Display::Hide;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("container [display: show]:").display();
    //     let expected = Display::Show;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("container:").display();
    //     let expected = Display::Show;
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn border_style() {
    //     let actual = attribs("border [border-style: thick]").border_style().clone();
    //     let expected = BorderStyle::Thick;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("border [border-style: thin]").border_style().clone();
    //     let expected = BorderStyle::Thin;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("border").border_style().clone();
    //     let expected = BorderStyle::Thin;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("border [border-style: 'abcd1234']").border_style().clone();
    //     let expected = BorderStyle::Custom("abcd1234".to_string());
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn word_wrap() {
    //     let actual = attribs("text [wrap: word]").word_wrap();
    //     let expected = Wrap::Word;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("text [wrap: no-wrap]").word_wrap();
    //     let expected = Wrap::NoWrap;
    //     assert_eq!(expected, actual);

    //     let actual = attribs("text [wrap: break]").word_wrap();
    //     let expected = Wrap::Break;
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn whitespace_attributes() {
    //     assert_eq!(attribs("text [trim-start: true]").trim_start(), true);
    //     assert_eq!(attribs("text").trim_start(), true);
    //     assert_eq!(attribs("text [trim-start: false]").trim_start(), false);

    //     assert_eq!(attribs("text [trim-end: true]").trim_end(), true);
    //     assert_eq!(attribs("text").trim_end(), true);
    //     assert_eq!(attribs("text [trim-end: false]").trim_end(), false);

    //     assert_eq!(attribs("text [collapse-spaces: true]").collapse_spaces(), true);
    //     assert_eq!(attribs("text").collapse_spaces(), true);
    //     assert_eq!(attribs("text [collapse-spaces: false]").collapse_spaces(), false);
    // }

    // // -----------------------------------------------------------------------------
    // //     - prop tests -
    // // -----------------------------------------------------------------------------
    // proptest! {
    //     #[test]
    //     fn parse_random_string_attribs(attrib in any::<String>()) {
    //         let attrib = attrib.replace('"', "");
    //         let attrib = attrib.replace('\\', "");
    //         let mut attributes = attribs(&format!("container [attrib:\"{attrib}\"]:"));
    //         let actual = attributes.get_value("attrib");
    //         let expected = Some(Value::String(attrib));
    //         assert_eq!(expected, actual);
    //     }
    // }
}
