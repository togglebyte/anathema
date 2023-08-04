use std::marker::PhantomData;

use flume::Sender;

use crate::hashmap::HashMap;
use crate::{Value, ValueRef};

pub(crate) struct ChangeBanana<T>(ValueRef<Value<T>>, Change<T>);

pub struct Notifier<T> {
    sender: Sender<ChangeBanana<T>>,
}

impl<T> Notifier<T> {
    pub(crate) fn new(sender: Sender<ChangeBanana<T>>) -> Self {
        Self { 
            sender 
        }
    }

    pub fn notify(&self, value_ref: ValueRef<Value<T>>, change: Change<T>) {
        let banana = ChangeBanana(value_ref, change);
        self.sender.send(banana);
    }
}

pub enum Change<T> {
    Modified,
    Add,
    Remove(ValueRef<T>),
    Swap(ValueRef<T>, ValueRef<T>),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ValueRef;

    #[test]
    fn notify_subscriber() {
        let value = ValueRef::<()>::new(0, 0);
    }
}
