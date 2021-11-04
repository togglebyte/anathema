use super::{Cursor, Pos};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("main window is already initialised")]
    InitMain,
    #[error("failed to initialise color pair")]
    InitPair,
    #[error("failed to initialise color")]
    InitColor,
    #[error("failed to set attribute")]
    AttributeSet,
    #[error("failed to create window")]
    CreateWindow,
    #[error("failed to print {0:?}")]
    Print(String),
    #[error("failed to print {0:?} at {1:?}")]
    PrintAt(String, Pos),
    #[error("failed to refresh window")]
    Refresh,
    #[error("failed to perform erase on window")]
    Erase,
    #[error("failed to add char: {0:?}")]
    AddChar(char),
    #[error("failed to move and add char: {0:?} | {1:?}")]
    MoveAddChar(char, Pos),
    #[error("failed to move {0:?}")]
    MoveCursor(Pos),
    #[error("failed to enable scrolling")]
    EnableScrolling,
    #[error("failed to disable scrolling")]
    DisableScrolling,
    #[error("failed to enable colors")]
    StartColor,
    #[error("failed to disable echo")]
    NoEcho,
    #[error("failed to enable no-delay")]
    NoDelay,
    #[error("failed to change cursor: {0:?}")]
    SetCursor(Cursor),
    #[error("failed to set nap")]
    Nap,
    #[error("failed to get color {0}")]
    NoColor(String),
    #[error("invalid color string: {0}")]
    InvalidColorString(String),
    #[error("failed to set scroll region")]
    SetScrollRegion,
    #[error("failed to draw a horizontal line")]
    HorizontalLine,
    #[error("failed to move window")]
    MoveWindow,
    #[error("failed to resize window")]
    Resize,
}
