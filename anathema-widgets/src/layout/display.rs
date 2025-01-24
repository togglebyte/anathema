use anathema_state::State;
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
        let disp = match s.as_ref() {
            "show" => Self::Show,
            "hide" => Self::Hide,
            "exclude" => Self::Exclude,
            _ => return Err(()),
        };
        Ok(disp)
    }
}

// impl From<Display> for CommonVal {
//     fn from(value: Display) -> Self {
//         let s = match value {
//             Display::Show => "show",
//             Display::Hide => "hide",
//             Display::Exclude => "exclude",
//         };

//         CommonVal::Str(s)
//     }
// }
