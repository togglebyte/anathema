use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// A remote cell allows the value to be updated from another location.
/// It's important that a `RemoteCell` is never available while its corresponding `RemoteHandle`
/// is, as that could lead to UB.
///
/// NOTE: This is designed to be used with Anathema attributes (and the value types in there), and nothing else.
///
/// # SAFETY:
///
/// This is not even remotely safe.
/// The remote cell can **not ever** be used at the same time as the remote handle.
///
/// NOTE: Since the remote cell is clonable it's not safe to ever implement any kind of 
/// mutable access for the remote cell.
pub struct RemoteCell<T> {
    value: Rc<UnsafeCell<T>>,
}

impl<T> Clone for RemoteCell<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone() }
    }
}

impl<T> RemoteCell<T> {
    /// Create a new instance of a remote cell and its corresponding handle
    pub fn new(value: T) -> (Self, RemoteHandle<T>) {
        let value = Rc::new(UnsafeCell::new(value));
        let handle = RemoteHandle::new(value.clone());
        let inst = Self { value };
        (inst, handle)
    }
}

impl<T> Deref for RemoteCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value.get() }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RemoteCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RC<")?;
        self.deref().fmt(f);
        write!(f, " ({:p})>", Rc::as_ptr(&self.value))
    }
}

/// A remote handle to update the underlying value.
///
/// This can never be used at the same time as the corresponding remote cell.
pub struct RemoteHandle<T> {
    value: Rc<UnsafeCell<T>>,
}

impl<T> std::fmt::Debug for RemoteHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<remote handle>")
    }
}

impl<T> RemoteHandle<T> {
    fn new(value: Rc<UnsafeCell<T>>) -> Self {
        Self { value }
    }

    /// Update the value of the remote cell.
    pub fn set(&mut self, new_value: T) {
        _ = std::mem::replace(unsafe { &mut *self.value.get() }, new_value);
    }

    pub fn value(&self) -> RemoteCell<T> {
        RemoteCell {
            value: self.value.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update_cell() {
        let (mut cell, mut handle) = RemoteCell::new("hello".to_string());

        assert_eq!(&*cell, "hello");

        handle.set("updated".to_string());
        assert_eq!(&*cell, "updated");
    }
}
