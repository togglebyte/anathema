use anathema_geometry::Pos;

#[derive(Debug, Copy, Clone)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub state: MouseState,
}

impl MouseEvent {
    /// Translate the x and y position into a `Position`
    pub fn pos(&self) -> Pos {
        (self.x, self.y).into()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseState {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
