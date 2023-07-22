// #![deny(missing_docs)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use anathema_render::Style;
use anathema_values::{GlobalBucket, StaticBucket};

use crate::gen::store::Values;
use crate::layout::{Align, Axis, Direction, Padding};
use crate::{fields, Attributes, Color, Display, Path, TextPath};

pub(crate) static GLOBAL_VALUES: StaticBucket<Value> = StaticBucket::new();

pub struct ValuesAttributes<'a, 'parent> {
    pub values: &'a Values<'parent>,
    attributes: &'a Attributes,
}

impl<'a, 'parent> ValuesAttributes<'a, 'parent> {
    pub fn text_to_string(&self, text: &'a TextPath) -> Cow<'a, str> {
        self.values.text_to_string(text)
    }

    pub fn new(values: &'a Values<'parent>, attributes: &'a Attributes) -> Self {
        Self { values, attributes }
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        let val = self.get_attrib(name)?;
        val.to_bool()
    }

    pub fn get_int(&self, name: &str) -> Option<u64> {
        let val = self.get_attrib(name)?;
        val.to_int()
    }

    pub fn get_signed_int(&self, name: &str) -> Option<i64> {
        let val = self.get_attrib(name)?;
        val.to_signed_int()
    }

    pub fn get_str(&self, name: &str) -> Option<Cow<'_, str>> {
        let value = self.get_attrib(name)?;
        match value {
            Value::String(s) => Some(Cow::from(s)),
            _ => None,
        }
    }

    pub fn get_attrib(&self, key: &str) -> Option<&'a Value> {
        let value = self.attributes.get(key)?;
        let Value::DataBinding(path) = value else {
            return Some(value);
        };
        path.lookup_value(self.values)
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

    pub fn factor(&self) -> Option<usize> {
        self.get_int(fields::FACTOR).map(|i| i as usize)
    }

    pub fn offset(&self) -> Option<i32> {
        self.get_signed_int(fields::OFFSET).map(|i| i as i32)
    }

    pub fn fill(&self) -> Option<Cow<'_, str>> {
        self.get_str(fields::FILL)
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        let value = self.get_attrib(name)?;
        match *value {
            Value::Color(color) => Some(color),
            _ => None,
        }
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
        self.get_int(fields::PADDING_TOP)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_right(&self) -> Option<usize> {
        self.get_int(fields::PADDING_RIGHT)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_bottom(&self) -> Option<usize> {
        self.get_int(fields::PADDING_BOTTOM)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_left(&self) -> Option<usize> {
        self.get_int(fields::PADDING_LEFT)
            .map(|i| i as usize)
            .or_else(|| self.padding())
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
        match *self.get_attrib(fields::AXIS)? {
            Value::Axis(axis) => Some(axis),
            _ => None,
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        match *self.get_attrib(fields::DIRECTION)? {
            Value::Direction(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn alignment(&self) -> Option<Align> {
        match &*self.get_attrib(fields::ALIGNMENT)? {
            Value::Alignment(val) => Some(*val),
            _ => None,
        }
    }

    pub fn max_children(&self) -> Option<usize> {
        self.get_int(fields::MAX_CHILDREN).map(|i| i as usize)
    }

    pub fn background(&self) -> Option<Color> {
        self.get_color(fields::BACKGROUND)
            .or_else(|| self.get_color(fields::BG))
    }

    pub fn foreground(&self) -> Option<Color> {
        self.get_color(fields::FOREGROUND)
            .or_else(|| self.get_color(fields::FG))
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

    pub fn display(&self) -> Display {
        match self.get_attrib(fields::DISPLAY).as_deref() {
            Some(Value::Display(display)) => *display,
            None | Some(_) => Display::Show,
        }
    }

    pub fn id(&self) -> Option<Cow<'_, str>> {
        self.get_str(fields::ID)
    }
}
