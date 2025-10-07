use std::ops::Deref;

use anathema_state::{CommonVal, PendingValue};
use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::{SmallIndex, SmallMap};
use anathema_value_resolver::Value;

use crate::paint::CellAttributes;
use crate::widget::ValueKey;
use crate::WidgetId;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        let s = String::from("hello");
        attributes.set("str", s.as_ref());
        attributes.set("num", 123u32);

        assert_eq!("hello", attributes.get_ref::<&str>("str").unwrap());
        assert_eq!(123, attributes.get::<u32>("num").unwrap());
    }

    #[test]
    fn write_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        attributes.set("num", 1u32);
        assert_eq!(1, attributes.get::<u32>("num").unwrap());
    }

    #[test]
    fn remove_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        attributes.remove("num");
        assert!(attributes.get::<u32>("num").is_none());
    }

    #[test]
    fn contains_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        assert!(attributes.contains("num"));
    }
}
