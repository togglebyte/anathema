use anathema_widgets::components::events::Event;
use russh::{ChannelId, server::Handle};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::eventmapper;

/// TerminalHandle is used to send data to the SSH client and handle input events.
/// It wraps the UnboundedSender to send data to the client.
/// It collects events from the SSH client, which can be processed by the Anathema Runtime.
#[derive(Clone)]
pub struct TerminalHandle {
    /// The sender is used to send data to the SSH client.
    sender: UnboundedSender<Vec<u8>>,
    /// The sink collects the data from the application to be sent to the client.
    sink: Vec<u8>,
    /// Event queue for processing input from SSH client.
    /// The Anathema Runtime will read events from this queue through the SSHBackend.
    events: Arc<Mutex<std::collections::VecDeque<Event>>>,
}

impl TerminalHandle {
    /// Create a new TerminalHandle that can send data to the SSH client.
    pub async fn start(handle: Handle, channel_id: ChannelId) -> Self {
        let (sender, mut receiver) = unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
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
            events: Arc::new(Mutex::new(std::collections::VecDeque::new())),
        }
    }

    /// Push raw input bytes to the application from the client.
    /// This is used to handle terminal input events.
    /// The data is parsed into Anathema events and stored in the event queue.
    pub fn push_input(&mut self, data: &[u8]) {
        // Convert raw input bytes to Anathema events
        let mut events = self.events.lock().unwrap();

        if let Ok(Some(e)) = terminput::Event::parse_from(data) {
            if let Some(event) = eventmapper::from_event(e) {
                events.push_back(event);
            } else {
                eprintln!("Unsupported event type in mapper: {:02x?}", data);
            }
        } else {
            eprintln!("Failed to parse input data as terminal event: {:02x?}", data);
        }
    }

    /// Push a custom event to the event queue for processing by the application.
    pub fn push_event(&mut self, event: Event) {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
    }

    /// Pop an event from the event queue.
    /// This is used by the SSHBackend->Anathema Runtime to retrieve events for processing.
    pub fn pop_event(&mut self) -> Option<Event> {
        let mut events = self.events.lock().unwrap();
        let event = events.pop_front();
        event
    }
}

// The SSHBackend writes to the terminal handle.
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
