use std::fmt;

use anathema_state::Color;
use crossterm::Command;

/// A command that sets the the background color of the entire terminal using OSC 11.
/// This might not work on all terminals. When it is not supported it will do nothing.
///
/// # Notes
///
/// Commands must be executed/queued for execution otherwise they do nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTerminalBackground(pub Color);

impl Command for SetTerminalBackground {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if let Color::Rgb(r, g, b) = self.0 {
            // OSC 11 format for RGB is 'rgb:RR/GG/BB' in hexadecimal format
            return write!(f, "\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x07", r, g, b);
        } else if let Color::AnsiVal(_) = self.0 {
            // Ansi values are not supported by OSC 11
            Ok(())
        } else {
            write!(f, "\x1b]11;{}\x07", self.0)
        }
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A command that resets the the background color with OSC 111.
///
/// # Notes
///
/// Commands must be executed/queued for execution otherwise they do nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetTerminalBackground();

impl Command for ResetTerminalBackground {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        // Some terminals require a ; at the end of the command
        // to reset the background color, while some do not work with it.
        let _ = write!(f, "\x1b]111\x07");
        write!(f, "\x1b]111;\x07")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::io::Result<()> {
        Ok(())
    }
}
