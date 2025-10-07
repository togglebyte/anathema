use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

pub(super) fn truncate<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 2 {
        return ValueKind::Null;
    }

    let Some(mut remainder) = args[1].as_int() else { return ValueKind::Null };

    let mut buffer = String::new();
    args[0].strings(|s| {
        let width = s.width() as i64;

        if width < remainder {
            remainder = remainder.saturating_sub(width);
            buffer.push_str(s);
        } else {
            let mut chars = s.chars();
            while remainder > 0 {
                let Some(c) = chars.next() else { break };
                let width = c.width().unwrap_or(0) as i64;
                if width > remainder {
                    break;
                };
                buffer.push(c);
                remainder = remainder.saturating_sub(width);
            }
        }
        true
    });
    ValueKind::Str(buffer.into())
}

pub(super) fn pad<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 2 {
        return ValueKind::Null;
    }

    let Some(width) = args[1].as_int() else { return ValueKind::Null };
    if width < 0 {
        return ValueKind::Null;
    }
    let width = width as usize;

    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(s);
        true
    });

    while width > buffer.width() {
        buffer.push(' ');
    }

    ValueKind::Str(buffer.into())
}

pub(super) fn width<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut width = 0;

    args[0].strings(|s| {
        width += s.width();
        true
    });

    ValueKind::Int(width as i64)
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

    #[test]
    fn truncate_long_string() {
        let val = value("this is a longer string");
        let val = truncate(&[val, value(4)]);
        assert_eq!(val, value("this"));
    }

    #[test]
    fn truncate_short_string() {
        let val = value("hi");
        let val = truncate(&[val, value(4)]);
        assert_eq!(val, value("hi"));
    }

    #[test]
    fn truncate_wide_cells() {
        let val = value("🐇🐇");
        let val = truncate(&[val, value(3)]);
        assert_eq!(val, value("🐇"));
    }

    #[test]
    fn pad_string() {
        let val = value("hi");
        let val = pad(&[val, value(3)]);
        assert_eq!(val, value("hi "));

        let val = value("bye");
        let val = pad(&[val, value(3)]);
        assert_eq!(val, value("bye"));

        let val = value("hello");
        let val = pad(&[val, value(3)]);
        assert_eq!(val, value("hello"));
    }

    #[test]
    fn pad_non_string() {
        let val = value(1);
        let val = pad(&[val, value(3)]);
        assert_eq!(val, value("1  "));
    }

    #[test]
    fn width_of_string() {
        let val = value("one");
        let val = width(&[val]);
        assert_eq!(val, value(3));
    }

    #[test]
    fn width_of_int() {
        let val = value(1);
        let val = width(&[val]);
        assert_eq!(val, value(1));
    }
}
