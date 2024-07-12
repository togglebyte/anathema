use anathema_state::CommonVal;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Display {
    #[default]
    Show,
    Hide,
    Exclude,
}

impl TryFrom<CommonVal<'_>> for Display {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        let disp = match value.to_common_str().as_ref() {
            "show" => Self::Show,
            "hide" => Self::Hide,
            "exclude" => Self::Exclude,
            _ => return Err(()),
        };
        Ok(disp)
    }
}

impl From<Display> for CommonVal<'_> {
    fn from(value: Display) -> Self {
        let s = match value {
            Display::Show => "show",
            Display::Hide => "hide",
            Display::Exclude => "exclude",
        };

        CommonVal::Str(s)
    }
}
