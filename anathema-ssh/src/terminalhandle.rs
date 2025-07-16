use anathema_widgets::components::events::Event;
use russh::{ChannelId, server::Handle};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

#[derive(Clone)]
pub struct TerminalHandle {
    sender: UnboundedSender<Vec<u8>>,
    // The sink collects the data which is finally sent to sender.
    sink: Vec<u8>,
    // Event queue for processing input from SSH client
    events: std::collections::VecDeque<Event>,
}

impl TerminalHandle {
    pub async fn start(handle: Handle, channel_id: ChannelId) -> Self {
        let (sender, mut receiver) = unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                // println!("Sending {} bytes to SSH client", data.len());
                let result = handle.data(channel_id, data.into()).await;
                if result.is_err() {
                    eprintln!("Failed to send data: {:?}", result);
                }
            }
            println!("SSH data sender task ended");
        });
        Self {
            sender,
            sink: Vec::new(),
            events: std::collections::VecDeque::new(),
        }
    }

    pub fn push_input(&mut self, data: &[u8]) {
        eprintln!(
            "TerminalHandle::push_input called with {} bytes: {:?}",
            data.len(),
            data
        );
        // Convert raw input bytes to Anathema events
        for &byte in data {
            match byte {
                b'\x1b' => {
                    // Escape sequence - for now, treat as escape key
                    eprintln!("Processing escape key");
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Esc,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\r' | b'\n' => {
                    // Enter key
                    eprintln!("Processing enter key");
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Enter,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\x7f' => {
                    // Backspace
                    eprintln!("Processing backspace key");
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Backspace,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\t' => {
                    // Tab
                    eprintln!("Processing tab key");
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Tab,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b' '..=b'~' => {
                    // Printable ASCII characters
                    //eprintln!("Processing character: '{}'", byte as char);
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Char(byte as char),
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                _ => {
                    // For other control characters, we can add more handling as needed
                    // For now, ignore or handle as needed
                    eprintln!("Ignoring unknown byte: 0x{:02x}", byte);
                }
            }
        }
        eprintln!("TerminalHandle now has {} events queued", self.events.len());
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        let event = self.events.pop_front();
        if let Some(ref event) = event {
            eprintln!("TerminalHandle::pop_event returning: {:?}", event);
        } else {
            // eprintln!("TerminalHandle::pop_event returning None (queue empty)");
        }
        event
    }
}

// The crossterm backend writes to the terminal handle.
impl std::io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.sink.is_empty() {
            let result = self.sender.send(self.sink.clone());
            if result.is_err() {
                return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, result.unwrap_err()));
            }
            self.sink.clear();
        }
        Ok(())
    }
}
