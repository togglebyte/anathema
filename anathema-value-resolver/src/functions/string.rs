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
