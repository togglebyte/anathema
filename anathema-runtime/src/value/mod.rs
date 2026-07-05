use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use anathema_store::slab::RcElement;

pub use self::changes::{Change, Changes, ValueIndex};
pub use self::list::List;
pub use self::map::Map;
pub use self::maybe::Maybe;
use crate::states::State;

cfg_select! {
    feature = "multithread" => {
        mod arcval;
        pub use arcval::{drain_changes, AnonValue, Value, ValueMut, ValueRef, Subs};
    }
    not(feature = "multithread") => {
        mod rcval;
        pub use rcval::{drain_changes, changed_one, changed_many, AnonValue, Value, ValueMut, ValueRef, Subs};
    }
}

mod changes;
mod list;
mod map;
mod maybe;

impl<V: Default + State> Default for Value<V> {
    fn default() -> Self {
        Self::new(V::default())
    }
}

impl<V: State> From<V> for Value<V> {
    fn from(value: V) -> Self {
        Value::new(value)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Type {
    Int = 1,
    Float = 2,
    Char = 3,
    String = 4,
    Bool = 5,
    Hex = 6,
    Map = 7,
    List = 8,
    Composite = 9,
    Unit = 10,
    Color = 11,
    Maybe = 12,
}

#[cfg(test)]
mod test {
    use anathema_store::slab::Key;
    use anathema_store::stack::Stack;

    use super::*;

    #[test]
    fn new_value() {
        let mut value = Value::new("hello world");
        let unique = value.to_mut();
        assert_eq!("hello world", *unique);
    }

    #[test]
    fn mutable_access() {
        let mut value = Value::new(String::new());
        {
            let mut unique = value.to_mut();
            unique.push_str("updated");
        }

        let unique = value.to_mut();
        assert_eq!("updated", *unique);
    }

    #[test]
    fn shared_access() {
        let expected = "hello world";
        let value = Value::new(expected);
        let s1 = value.to_ref();
        let s2 = value.to_ref();

        assert_eq!(*s1, expected);
        assert_eq!(*s2, expected);
    }

    #[test]
    #[should_panic(expected = "RefCell already borrowed")]
    fn mutable_shared_panic() {
        // This should panic because of mutable access
        // is held while also having a value reference.
        let mut value = Value::new(String::new());
        let s1 = value.reference();
        let _r1 = s1.value::<String>();
        let _m1 = value.to_mut();
    }

    #[test]
    fn value_ref_to_shared_state() {
        let value = Value::new(1);
        let r1 = value.reference();
        let r2 = value.reference();

        let s1 = r1.as_state();
        let s2 = r2.as_state();

        let val = s1.as_int().unwrap() + s2.as_int().unwrap();

        assert_eq!(val, 2);
    }
}
