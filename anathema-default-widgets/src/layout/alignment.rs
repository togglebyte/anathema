use anathema::CommonVal;

pub const ALIGNMENT: &str = "alignment";

/// Word wrapping strategy
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Centre,
}

impl TryFrom<CommonVal<'_>> for Alignment {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Str(wrap) => match wrap {
                "top_left" => Ok(Alignment::TopLeft),
                "top" => Ok(Alignment::Top),
                "top_right" => Ok(Alignment::TopRight),
                "right" => Ok(Alignment::Right),
                "left" => Ok(Alignment::Left),
                "bottom_left" => Ok(Alignment::BottomLeft),
                "bottom" => Ok(Alignment::Bottom),
                "bottom_right" => Ok(Alignment::BottomRight),
                "centre" | "center" => Ok(Alignment::Centre),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
