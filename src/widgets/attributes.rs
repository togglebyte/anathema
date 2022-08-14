use std::collections::HashMap;
use std::time::Duration;

use crate::display::{Color, Style};

use super::value::Path;
use super::value::{Easing, Value};
use super::{
    Align, BorderStyle, Direction, Display, HorzEdge, NodeId, Offset, Padding, Sides, TextAlignment, VertEdge, Wrap,
};

// -----------------------------------------------------------------------------
//     - Attribute names -
// -----------------------------------------------------------------------------
pub mod fields {
    pub const ALIGNMENT: &str = "align";
    pub const ANIMATE: &str = "animate";
    pub const AUTO_SCROLL: &str = "auto-scroll";
    pub const BACKGROUND: &str = "background";
    pub const BINDING: &str = "binding";
    pub const BORDER_CHARS: &str = "border-chars";
    pub const BORDER_STYLE: &str = "border-style";
    pub const BOTTOM: &str = "bottom";
    pub const COLLAPSE_SPACES: &str = "collapse-spaces";
    pub const DATA: &str = "data";
    pub const DIRECTION: &str = "direction";
    pub const DISPLAY: &str = "display";
    pub const FACTOR: &str = "factor";
    pub const FILL: &str = "fill";
    pub const FOREGROUND: &str = "foreground";
    pub const HEIGHT: &str = "height";
    pub const H_OFFSET: &str = "h-offset";
    pub const H_OFFSET_EDGE: &str = "h-offset-edge";
    pub const ID: &str = "id";
    pub const LEFT: &str = "left";
    pub const MAX_CHILDREN: &str = "max-children";
    pub const MAX_HEIGHT: &str = "max-height";
    pub const MAX_WIDTH: &str = "max-width";
    pub const MIN_HEIGHT: &str = "min-height";
    pub const MIN_WIDTH: &str = "min-width";
    pub const NAME: &str = "name";
    pub const OFFSET: &str = "offset";
    pub const PADDING: &str = "padding";
    pub const PADDING_BOTTOM: &str = "padding-bottom";
    pub const PADDING_LEFT: &str = "padding-left";
    pub const PADDING_RIGHT: &str = "padding-right";
    pub const PADDING_TOP: &str = "padding-top";
    pub const POSITION: &str = "position";
    pub const REVERSE: &str = "reverse";
    pub const RIGHT: &str = "right";
    pub const SIDES: &str = "sides";
    pub const TAB_STOP: &str = "tab";
    pub const TEXT_ALIGN: &str = "text-align";
    pub const TOP: &str = "top";
    pub const TRIM_END: &str = "trim-end";
    pub const TRIM_START: &str = "trim-start";
    pub const V_OFFSET: &str = "v-offset";
    pub const V_OFFSET_EDGE: &str = "v-offset-edge";
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

impl std::ops::Index<&str> for Attributes {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        &self.inner[index]
    }
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

    pub fn offset(&self) -> Offset {
        let h_offset = self.get_signed_int(fields::H_OFFSET).unwrap_or(0) as i32;
        let v_offset = self.get_signed_int(fields::V_OFFSET).unwrap_or(0) as i32;

        let h_offset_edge = match self.get_str(fields::H_OFFSET_EDGE) {
            Some(fields::LEFT) => Some(HorzEdge::Left(h_offset)),
            Some(fields::RIGHT) => Some(HorzEdge::Right(h_offset)),
            _ => None,
        };

        let v_offset_edge = match self.get_str(fields::V_OFFSET_EDGE) {
            Some(fields::TOP) => Some(VertEdge::Top(v_offset)),
            Some(fields::BOTTOM) => Some(VertEdge::Bottom(v_offset)),
            _ => None,
        };

        match (h_offset_edge, v_offset_edge) {
            (Some(h), Some(v)) => Offset { h_edge: Some(h), v_edge: Some(v) },
            (None, Some(v)) => Offset { h_edge: None, v_edge: Some(v) },
            (Some(h), None) => Offset { h_edge: Some(h), v_edge: None },
            (None, None) => Offset::new(),
        }
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

    pub fn direction(&self) -> Option<Direction> {
        match self.value(fields::DIRECTION) {
            Some(Value::Direction(val)) => Some(*val),
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
