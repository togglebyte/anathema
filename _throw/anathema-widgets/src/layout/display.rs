use anathema_value_resolver::ValueKind;

pub const DISPLAY: &str = "display";

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Display {
    #[default]
    Show,
    Hide,
    Exclude,
}

impl TryFrom<&ValueKind<'_>> for Display {
    type Error = ();

    fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
        let Some(s) = value.as_str() else { return Err(()) };
        let disp = match s {
            "show" => Self::Show,
            "hide" => Self::Hide,
            "exclude" => Self::Exclude,
            _ => return Err(()),
        };
        Ok(disp)
    }
}

impl From<Display> for ValueKind<'_> {
    fn from(value: Display) -> Self {
        let value = match value {
            Display::Show => "show",
            Display::Hide => "hide",
            Display::Exclude => "exclude",
        };
        ValueKind::Str(value.into())
    }
}

#[cfg(test)]
mod test {
    use anathema_value_resolver::Attributes;

    use super::*;

    #[test]
    fn to_and_from_attributes() {
        let mut attribs = Attributes::empty();
        attribs.set("disp", Display::Show);
        assert_eq!(Display::Show, attribs.get_as::<Display>("disp").unwrap());

        attribs.set("disp", Display::Hide);
        assert_eq!(Display::Hide, attribs.get_as::<Display>("disp").unwrap());

        attribs.set("disp", Display::Exclude);
        assert_eq!(Display::Exclude, attribs.get_as::<Display>("disp").unwrap());
    }
}
