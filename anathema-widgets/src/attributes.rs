use std::collections::HashMap;
use std::time::Duration;

use crate::values::{Easing, Value};

// -----------------------------------------------------------------------------
//     - Attribute names -
// -----------------------------------------------------------------------------
pub mod fields {
    pub const ALIGNMENT: &str = "align";
    pub const ANIMATE: &str = "animate";
    pub const AUTO_SCROLL: &str = "auto-scroll";
    pub const AXIS: &str = "axis";
    pub const BACKGROUND: &str = "background";
    pub const BG: &str = "bg";
    pub const BINDING: &str = "binding";
    pub const BORDER_CHARS: &str = "border-chars";
    pub const BORDER_STYLE: &str = "border-style";
    pub const BOTTOM: &str = "bottom";
    pub const COLLAPSE_SPACES: &str = "collapse-spaces";
    pub const DATA: &str = "data";
    pub const DISPLAY: &str = "display";
    pub const DIRECTION: &str = "direction";
    pub const FACTOR: &str = "factor";
    pub const FG: &str = "fg";
    pub const FILL: &str = "fill";
    pub const FOREGROUND: &str = "foreground";
    pub const HEIGHT: &str = "height";
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
        Self {
            inner: HashMap::new(),
        }
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
            Value::Transition(val, duration, easing) if k != fields::ALIGNMENT => val
                .to_signed_int()
                .map(|val| (k.as_ref(), val as f32, *duration, *easing)),
            _ => None,
        })
    }

    pub fn has(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.inner.get(name)
    }

    pub fn set(&mut self, name: impl AsRef<str>, value: impl Into<Value>) {
        self.inner.insert(name.as_ref().into(), value.into());
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;
    type Item = (&'a String, &'a Value);

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Attributes {
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Value>;
    type Item = (&'a String, &'a mut Value);

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
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