use anathema_store::slab::Element;
use anathema_store::store::{OwnedKey, SharedKey};

use super::{ValueKey, OWNED, SHARED, SUBSCRIBERS};
use crate::states::AnyState;

// Write a new value into the `OWNED` store and associate
// a subscriber key with the value.
pub(crate) fn new_value(value: Box<dyn AnyState>) -> ValueKey {
    let owned_key = OWNED.with(|owned| owned.push(value));
    let sub_key = SUBSCRIBERS.with_borrow_mut(|subscribers| subscribers.push_empty());
    ValueKey(owned_key, sub_key)
}

pub(crate) fn with_owned<F, T>(key: OwnedKey, f: F) -> T
where
    F: Fn(&dyn AnyState) -> T,
{
    let val = get_unique(key);
    let ret = f(&val);
    return_owned(key, val);
    ret
}

// Get access to the owned value. This allows mutating the value.
//
// This checks out the value, making impossible to call `get_unique` again
// until the value has been returned (using `return_owned`).
pub(crate) fn get_unique(key: OwnedKey) -> Box<dyn AnyState> {
    OWNED.with(|owned| owned.unique(key))
}

// Try to make an owned value into a shared value, if it isn't already.
// To get access to another shared instance of the value, call this function again.
pub(crate) fn try_make_shared(owned_key: OwnedKey) -> Option<(SharedKey, Element<Box<dyn AnyState>>)> {
    fn lookup_shared(key: SharedKey) -> Element<Box<dyn AnyState>> {
        SHARED.with(|shared| shared.get(key))
    }

    OWNED.with(|owned| {
        match owned.get_shared_key(owned_key) {
            Some(key) => Some((key, lookup_shared(key))),
            None => {
                // Transfer value from OWNED to SHARED
                let value = owned.try_unique(owned_key)?;
                SHARED.with(|shared| {
                    let key = shared.insert(owned_key, value);
                    owned.try_set_as_shared(owned_key, key).then(|| {
                        let value = lookup_shared(key);
                        Some((key, value))
                    })?
                })
            }
        }
    })
}

// Make an owned value into a shared value, if it isn't already.
// Mutation is not possible while the value is shared.
//
// To get access to another shared instance of the value, call this function again.
//
// This function assumes the value exists and should be limited to `Value<T>`.
// If there is a chance the value is no longer present use `try_make_shared` instead.
pub(crate) fn make_shared(owned_key: OwnedKey) -> Option<(SharedKey, Element<Box<dyn AnyState>>)> {
    fn lookup_shared(key: SharedKey) -> Element<Box<dyn AnyState>> {
        SHARED.with(|shared| shared.get(key))
    }

    OWNED.with(|owned| {
        match owned.get_shared_key(owned_key) {
            Some(key) => Some((key, lookup_shared(key))),
            None => {
                // Transfer value from OWNED to SHARED
                let value = owned.unique(owned_key);
                SHARED.with(|shared| {
                    let key = shared.insert(owned_key, value);
                    if owned.try_set_as_shared(owned_key, key) {
                        let value = lookup_shared(key);
                        Some((key, value))
                    } else {
                        None
                    }
                })
            }
        }
    })
}

// Return an owned value back into `OWNED`.
pub(crate) fn return_owned(key: OwnedKey, value: Box<dyn AnyState>) {
    OWNED.with(|owned| owned.return_unique_borrow(key, value));
}

// Return a shared value.
// If the value reference count reaches zero then the value is removed
// and returned as an owned value.
pub(crate) fn return_shared(key: SharedKey) {
    if let Some(value) = SHARED.with(|shared| shared.try_evict(key)) {
        return_owned(key.into(), value);
    }
}

// Remove a value and it's associated subscribers
pub(crate) fn drop_value(key: ValueKey) {
    let _ = OWNED.with(|owned| owned.remove(key.0));
    let _ = SUBSCRIBERS.with_borrow_mut(|subscribers| subscribers.remove(key.1));
}

pub(crate) fn copy_val<T: 'static + Copy>(key: OwnedKey) -> T {
    OWNED
        .with(|owned| {
            owned.with(key, |val| {
                *val.to_any_ref()
                    .downcast_ref::<T>()
                    .expect("the value type is determined by the wrapping Value<T> and should not change")
            })
        })
        .expect("the value is assumed to not be checked out or borrowed")
}

// /// This should only be used for debugging.
// pub fn dump_state() -> String {
//     use std::fmt::Write;
//     let mut string = String::new();
//     let _ = writeln!(
//         &mut string,
//         "\n\n=== SHARED ===\n{}\n",
//         SHARED.with(|s| s.dump_state())
//     );
//     let _ = writeln!(
//         &mut string,
//         "=== OWNED ===\n{}\n",
//         OWNED.with(|s| s.dump_state())
//     );
//     string
// }
