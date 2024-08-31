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

    /// Returns true if the left mouse button is down
    pub fn lsb_down(&self) -> bool {
        matches!(
            self.state,
            MouseState::Down(MouseButton::Left) | MouseState::Drag(MouseButton::Left)
        )
    }

    /// Returns true if the right mouse button is down
    pub fn rsb_down(&self) -> bool {
        matches!(
            self.state,
            MouseState::Down(MouseButton::Right) | MouseState::Drag(MouseButton::Right)
        )
    }

    /// Returns true if the left mouse button is released
    pub fn lsb_up(&self) -> bool {
        matches!(self.state, MouseState::Up(MouseButton::Left))
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
