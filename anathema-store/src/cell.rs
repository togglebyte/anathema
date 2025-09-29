use std::cell::UnsafeCell;
use std::rc::Rc;

pub struct RemoteCell<T> {
    inner: Rc<UnsafeCell<T>>,
}

impl<T> RemoteCell<T> {
}
