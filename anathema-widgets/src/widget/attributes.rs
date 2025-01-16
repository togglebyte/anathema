use std::ops::Deref;

use anathema_state::{CommonVal, PendingValue};
use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::{SmallIndex, SmallMap};
use anathema_value_resolver::Value;

use crate::paint::CellAttributes;
use crate::widget::ValueKey;
use crate::WidgetId;

// impl CellAttributes for Attributes<'_> {
//     fn with_str(&self, key: &str, f: &mut dyn FnMut(&str)) {
//         panic!("maybe this can be removed?");
//         // let Some(value) = self.get_val(key).and_then(|value| value.load_common_val()) else { return };
//         // let Some(value) = value.to_common() else { return };
//         // let CommonVal::Str(s) = value else { return };
//         // f(s);
//     }

//     fn get_i64(&self, key: &str) -> Option<i64> {
//         self.get_int(key)
//     }

//     fn get_u8(&self, key: &str) -> Option<u8> {
//         self.get_int(key).map(|i| i as u8)
//     }

//     fn get_hex(&self, key: &str) -> Option<anathema_state::Hex> {
//         let value = self.get_val(key)?;
//         let value = value.load_common_val()?;
//         match value.to_common()? {
//             CommonVal::Hex(hex) => Some(hex),
//             _ => None,
//         }
//     }

//     fn get_color(&self, key: &str) -> Option<anathema_state::Color> {
//         let value = self.get_val(key)?;
//         let value = value.load_common_val()?;
//         match value.to_common()? {
//             CommonVal::Color(color) => Some(color),
//             _ => None,
//         }
//     }

//     fn get_bool(&self, key: &str) -> bool {
//         Attributes::get_bool(self, key)
//     }
// }

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
