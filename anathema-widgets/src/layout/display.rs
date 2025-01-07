use anathema_state::{CommonVal, State};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Display {
    #[default]
    Show,
    Hide,
    Exclude,
}

// impl State for Display {
//     fn to_common(&self) -> Option<CommonVal> {
//         let val = match self {
//             Display::Show => CommonVal::Str("show"),
//             Display::Hide => CommonVal::Str("hide"),
//             Display::Exclude => CommonVal::Str("exclude"),
//         };
//         Some(val)
//     }
// }

// impl TryFrom<CommonVal> for Display {
//     type Error = ();

//     fn try_from(value: CommonVal) -> Result<Self, Self::Error> {
//         let disp = match value.to_common_str().as_ref() {
//             "show" => Self::Show,
//             "hide" => Self::Hide,
//             "exclude" => Self::Exclude,
//             _ => return Err(()),
//         };
//         Ok(disp)
//     }
// }

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
