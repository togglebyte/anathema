use anathema_geometry::Pos;

#[derive(Debug, Copy, Clone, PartialEq)]
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

    /// Returns true if the left mouse button is pressed
    pub fn left_down(&self) -> bool {
        matches!(self.state, MouseState::Down(MouseButton::Left))
    }

    /// Returns true if the right mouse button is pressed
    pub fn right_down(&self) -> bool {
        matches!(self.state, MouseState::Down(MouseButton::Right))
    }

    /// Returns true if the middle mouse button is pressed
    pub fn middle_down(&self) -> bool {
        matches!(self.state, MouseState::Down(MouseButton::Middle))
    }

    /// Returns true if the left mouse button is down
    pub fn is_left_down(&self) -> bool {
        matches!(
            self.state,
            MouseState::Down(MouseButton::Left) | MouseState::Drag(MouseButton::Left)
        )
    }

    /// Returns true if the right mouse button is down
    pub fn is_right_down(&self) -> bool {
        matches!(
            self.state,
            MouseState::Down(MouseButton::Right) | MouseState::Drag(MouseButton::Right)
        )
    }

    /// Returns true if the middle mouse button is down
    pub fn is_middle_down(&self) -> bool {
        matches!(
            self.state,
            MouseState::Down(MouseButton::Middle) | MouseState::Drag(MouseButton::Middle)
        )
    }

    /// Returns true if the left mouse button is released
    pub fn left_up(&self) -> bool {
        matches!(self.state, MouseState::Up(MouseButton::Left))
    }

    /// Returns true if the right mouse button is released
    pub fn right_up(&self) -> bool {
        matches!(self.state, MouseState::Up(MouseButton::Right))
    }

    /// Returns true if the middle mouse button is released
    pub fn middle_up(&self) -> bool {
        matches!(self.state, MouseState::Up(MouseButton::Middle))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
