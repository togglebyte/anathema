// #![deny(missing_docs)]

pub use anathema_render::Color;

/// Determine how a widget should be displayed and laid out
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Display {
    /// Show the widget, this is the default
    #[default]
    Show,
    /// Include the widget as part of the layout but don't render it
    Hide,
    /// Exclude the widget from the layout and paint step.
    Exclude,
}

impl TryFrom<&str> for Display {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let display = match value {
            "hide" => Self::Hide,
            "exclude" => Self::Exclude,
            _ => Self::Show,
        };
        Ok(display)
    }
}
