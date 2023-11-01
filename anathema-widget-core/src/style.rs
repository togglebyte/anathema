use anathema_render::{Color, Attributes, Style as RenderStyle};
use anathema_values::Value;

#[derive(Debug)]
pub struct WidgetStyle {
    pub(crate) fg: Value<Color>,
    pub(crate) bg: Value<Color>,
    pub(crate) bold: Value<bool>,
    pub(crate) dim: Value<bool>,
    pub(crate) italic: Value<bool>,
    pub(crate) underlined: Value<bool>,
    pub(crate) crossed_out: Value<bool>,
    pub(crate) overlined: Value<bool>,
    pub(crate) inverse: Value<bool>,
}

impl WidgetStyle {
    pub fn style(&self) -> RenderStyle {
        let mut attributes: Attributes = Attributes::empty();

        if let Some(true) = self.bold.value() {
            attributes |= Attributes::BOLD;
        }

        if let Some(true) = self.dim.value() {
            attributes |= Attributes::DIM;
        }

        if let Some(true) = self.italic.value() {
            attributes |= Attributes::ITALIC;
        }

        if let Some(true) = self.underlined.value() {
            attributes |= Attributes::UNDERLINED;
        }

        if let Some(true) = self.crossed_out.value() {
            attributes |= Attributes::CROSSED_OUT;
        }

        if let Some(true) = self.overlined.value() {
            attributes |= Attributes::OVERLINED;
        }

        if let Some(true) = self.inverse.value() {
            attributes |= Attributes::INVERSE;
        }

        RenderStyle {
            fg: self.fg.value().cloned(),
            bg: self.bg.value().cloned(),
            attributes,
        }
    }
}
