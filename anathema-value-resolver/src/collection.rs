use anathema_state::Subscriber;
use anathema_strings::HStrings;
use anathema_templates::Expression;

use crate::expression::ValueExpr;
use crate::value::{Value, ValueKind};

#[derive(Debug)]
pub struct Collection<'bp> {
    inner: Value<'bp>,
}

impl<'bp> Collection<'bp> {
    pub fn new(inner: Value<'bp>) -> Self {
        Self { inner }
    }

    pub fn len(&self) -> usize {
        match &self.inner.kind {
            ValueKind::List(l) => l.len(),
            ValueKind::DynList(list) => match list.as_state() {
                None => 0,
                Some(state) => match state.as_any_list() {
                    None => 0,
                    Some(list) => list.len(),
                },
            },
            ValueKind::Null => 0,
            _ => unreachable!(),
        }
    }

    pub fn reload(&mut self) {
        self.inner.reload();
    }
}

#[cfg(test)]
mod test {
    use anathema_state::List;
    use anathema_templates::expressions::{ident, index, num, strlit};

    use super::*;
    use crate::testing::setup;

    #[test]
    fn static_collection() {
        let expr = index(index(ident("state"), strlit("list")), num(1));
        let mut list = List::empty();
        list.push(123);
        list.push(456);

        let test = setup().finish(|mut test| {
            test.set_state("list", list);
            let value = test.eval(&*expr);
            let collection = Collection::new(value);
        });
    }
}
