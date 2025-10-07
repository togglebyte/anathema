use crate::ValueKind;

pub(super) fn to_int<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    match &args[0] {
        ValueKind::Int(i) => ValueKind::Int(*i),
        ValueKind::Float(f) => ValueKind::Int(*f as i64),
        ValueKind::Bool(b) if !b => ValueKind::Int(0),
        ValueKind::Bool(_) => ValueKind::Int(1),
        ValueKind::Char(c) => match c.to_digit(10) {
            Some(i) => ValueKind::Int(i as i64),
            None => ValueKind::Null,
        },
        ValueKind::Hex(hex) => ValueKind::Int(hex.as_u32() as i64),
        ValueKind::Str(s) => match s.parse() {
            Ok(i) => ValueKind::Int(i),
            Err(_) => ValueKind::Null,
        },
        ValueKind::Color(_)
        | ValueKind::Null
        | ValueKind::Map
        | ValueKind::Attributes
        | ValueKind::List(_)
        | ValueKind::DynList(_)
        | ValueKind::DynMap(_)
        | ValueKind::Range(..)
        | ValueKind::Composite(_) => ValueKind::Null,
    }
}

pub(super) fn to_float<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    match &args[0] {
        ValueKind::Int(i) => ValueKind::Float(*i as f64),
        ValueKind::Float(f) => ValueKind::Float(*f),
        ValueKind::Bool(b) if !b => ValueKind::Float(0.0),
        ValueKind::Bool(_) => ValueKind::Float(1.0),
        ValueKind::Char(c) => match c.to_digit(10) {
            Some(i) => ValueKind::Float(i as f64),
            None => ValueKind::Null,
        },
        ValueKind::Hex(hex) => ValueKind::Float(hex.as_u32() as f64),
        ValueKind::Str(s) => match s.parse() {
            Ok(i) => ValueKind::Float(i),
            Err(_) => ValueKind::Null,
        },
        _ => ValueKind::Null,
    }
}

pub(super) fn round<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.is_empty() || args.len() > 2 {
        return ValueKind::Null;
    }

    let precision = match to_int(&args[1..]) {
        ValueKind::Int(i) => i as usize,
        _ => 0,
    };

    match &args[0] {
        ValueKind::Float(f) => ValueKind::Str(format!("{f:.precision$}").into()),
        _ => ValueKind::Null,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::functions::test::value;

    #[test]
    fn str_to_int() {
        let arg = value("123");
        let output = to_int(&[arg]);
        assert_eq!(ValueKind::Int(123), output);

        let arg = value("not a number");
        let output = to_int(&[arg]);
        assert_eq!(ValueKind::Null, output);
    }

    #[test]
    fn char_to_int() {
        let arg = value('4');
        let output = to_int(&[arg]);
        assert_eq!(ValueKind::Int(4), output);

        let arg = value('G');
        let output = to_int(&[arg]);
        assert_eq!(ValueKind::Null, output);
    }

    #[test]
    fn float_to_int() {
        let arg = value(1.1234);
        let output = to_int(&[arg]);
        assert_eq!(ValueKind::Int(1), output);
    }

    #[test]
    fn rounding_default_zero() {
        let args = [value(1.1234)];
        let output = round(&args);
        assert_eq!(value("1"), output);
    }

    #[test]
    fn rounding_with_arg() {
        let args = [value(1.1234), value(2)];
        let output = round(&args);
        assert_eq!(value("1.12"), output);
    }

    #[test]
    fn invalid_rounding() {
        let args = [value("1.1234"), value(2)];
        let output = round(&args);
        assert_eq!(ValueKind::Null, output);
    }
}
