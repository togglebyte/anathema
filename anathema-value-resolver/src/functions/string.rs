use crate::ValueKind;

pub(super) fn to_upper<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(&s.to_uppercase());
        true
    });

    ValueKind::Str(buffer.into())
}

pub(super) fn to_lower<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(&s.to_lowercase());
        true
    });

    ValueKind::Str(buffer.into())
}

pub(super) fn to_str<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }
    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(s);
        true
    });
    ValueKind::Str(buffer.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::functions::test::{list, value};

    fn to_string(value: ValueKind<'_>) -> String {
        let ValueKind::Str(s) = to_str(&[value]) else { panic!() };
        s.to_string()
    }

    #[test]
    fn string_to_upper() {
        let val = value("hello");
        let val = to_upper(&[val]);
        let val = to_string(val);
        assert_eq!(val, "HELLO");
    }

    #[test]
    fn string_to_lower() {
        let val = value("HELLO");
        let val = to_lower(&[val]);
        let val = to_string(val);
        assert_eq!(val, "hello");
    }

    #[test]
    fn int_to_string() {
        let val = value(123);
        let val = to_str(&[val]);
        assert_eq!(val, value("123"));
    }

    #[test]
    fn bool_to_string() {
        let val = value(true);
        let val = to_str(&[val]);
        assert_eq!(val, value("true"));
    }

    #[test]
    fn list_to_string() {
        let val = list([1, 2, 3]);
        let val = to_str(&[val]);
        assert_eq!(val, value("123"));
    }
}
