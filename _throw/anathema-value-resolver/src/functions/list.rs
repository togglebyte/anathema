use crate::ValueKind;

pub(super) fn contains<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 2 {
        return ValueKind::Null;
    }

    let haystack = &args[0];
    let needle = &args[1];

    match haystack {
        ValueKind::Str(string) => match needle {
            ValueKind::Str(other) => ValueKind::Bool(string.as_ref().contains(other.as_ref())),
            _ => ValueKind::Bool(false),
        },
        ValueKind::List(elements) => elements
            .iter()
            .find_map(|el| needle.eq(el).then_some(ValueKind::Bool(true)))
            .unwrap_or(ValueKind::Bool(false)),
        ValueKind::DynList(haystack) => {
            let Some(haystack) = haystack.as_state() else { return ValueKind::Bool(false) };
            let Some(haystack) = haystack.as_any_list() else { return ValueKind::Bool(false) };
            for val in haystack.iter() {
                if needle.compare_pending(val) {
                    return ValueKind::Bool(true);
                }
            }
            ValueKind::Bool(false)
        }
        _ => ValueKind::Null,
    }
}

pub(super) fn len<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let len = match &args[0] {
        ValueKind::Str(string) => string.len(),
        ValueKind::List(list) => list.len(),
        ValueKind::Range(from, to) => to - from,
        ValueKind::DynList(pending) => match pending.as_state() {
            None => return ValueKind::Null,
            Some(state) => match state.as_any_list() {
                None => return ValueKind::Null,
                Some(list) => list.len(),
            },
        },
        _ => return ValueKind::Null,
    };

    ValueKind::Int(len as i64)
}

#[cfg(test)]
mod test {
    use anathema_state::{List, Value};

    use super::*;
    use crate::functions::test::{list, value};

    #[test]
    fn list_contains_int() {
        let haystack = list([1, 2]);
        let needle = value(2);
        let args = [haystack, needle];
        assert!(matches!(contains(&args), ValueKind::Bool(true)));
    }

    #[test]
    fn list_contains_list() {
        let haystack = list([list([1, 2]), list([4, 8])]);
        let needle = list([4, 8]);
        let args = [haystack, needle];
        assert!(matches!(contains(&args), ValueKind::Bool(true)));
    }

    #[test]
    fn dyn_list_contains() {
        let list = Value::new(List::from_iter(0..10));
        let haystack = ValueKind::DynList(list.reference());
        let needle = value(0);
        let args = [haystack, needle];
        contains(&args);
        assert!(matches!(contains(&args), ValueKind::Bool(true)));
    }

    #[test]
    fn static_string_contains() {
        let haystack = value("like looking for a needle in a");
        let needle = value("needle");
        let args = [haystack, needle];
        assert!(matches!(contains(&args), ValueKind::Bool(true)));
    }

    #[test]
    fn dyn_list_of_lists_contains() {
        let haystack = List::from_iter(vec![List::from_iter(0..10), List::from_iter(100..110)]);
        let haystack = Value::new(haystack);
        let haystack = ValueKind::DynList(haystack.reference());
        let needle = List::from_iter(100..110);
        let needle = Value::new(needle);
        let needle = ValueKind::DynList(needle.reference());
        let args = [haystack, needle];
        let result = contains(&args);
        assert!(matches!(result, ValueKind::Bool(true)));
    }

    #[test]
    fn list_len() {
        let list = value(vec![1, 2, 3]);
        let args = [list];
        let result = len(&args);
        assert!(matches!(result, ValueKind::Int(3)));
    }

    #[test]
    fn dyn_list_len() {
        let list = List::from_iter([1, 2, 3]);
        let list = Value::new(list);
        let list = ValueKind::DynList(list.reference());
        let args = [list];
        let result = len(&args);
        assert!(matches!(result, ValueKind::Int(3)));
    }

    #[test]
    fn string_len() {
        let value = value("hello");
        let args = [value];
        let result = len(&args);
        assert!(matches!(result, ValueKind::Int(5)));
    }

    #[test]
    fn int_len() {
        let value = value(123);
        let args = [value];
        let result = len(&args);
        assert!(matches!(result, ValueKind::Null));
    }
}
