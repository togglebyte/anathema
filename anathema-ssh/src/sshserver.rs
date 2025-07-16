use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_widgets::GlyphMap;
use anathema_widgets::components::events::Event;
use rand_core::OsRng;
use russh::keys::ssh_key::{self, PublicKey};
use russh::{Channel, ChannelId, Pty};
use russh::{CryptoVec, server::*};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::sshbackend::SSHBackend;

#[derive(Clone)]
pub struct TerminalHandle {
    sender: UnboundedSender<Vec<u8>>,
    // The sink collects the data which is finally sent to sender.
    sink: Vec<u8>,
    // Event queue for processing input from SSH client
    events: std::collections::VecDeque<Event>,
}

impl TerminalHandle {
    async fn start(handle: Handle, channel_id: ChannelId) -> Self {
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

    fn push_input(&mut self, data: &[u8]) {
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
                    eprintln!("Processing character: '{}'", byte as char);
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

#[derive(Clone)]
pub struct AnathemaSSHServer {
    clients: HashMap<usize, (Arc<Mutex<SSHBackend>>, TerminalHandle)>,
    id: usize,
    app_runner: Arc<Mutex<dyn FnMut(&mut SSHBackend) -> anyhow::Result<()> + Send + Sync>>,
}

impl AnathemaSSHServer {
    pub fn new(app_runner: impl FnMut(&mut SSHBackend) -> anyhow::Result<()> + Send + Sync + 'static) -> Self {
        Self {
            clients: HashMap::new(),
            id: 0,
            app_runner: Arc::new(Mutex::new(app_runner)),
        }
    }

    /// Load or create a persistent SSH key
    fn load_or_create_key() -> Result<russh::keys::PrivateKey, anyhow::Error> {
        let key_dir = Path::new(".ssh_keys");
        let key_file = key_dir.join("server_key");

        // Create the directory if it doesn't exist
        if !key_dir.exists() {
            fs::create_dir_all(key_dir)?;
        }

        // Try to load existing key
        if key_file.exists() {
            match fs::read_to_string(&key_file) {
                Ok(key_data) => match russh::keys::PrivateKey::from_openssh(&key_data) {
                    Ok(key) => {
                        println!("Loaded existing SSH key from {}", key_file.display());
                        return Ok(key);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse existing key: {}, generating new one", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read key file: {}, generating new one", e);
                }
            }
        }

        // Generate new key
        let key = russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519)?;

        // Save the key
        let key_data = key.to_openssh(ssh_key::LineEnding::LF)?;
        match fs::write(&key_file, &key_data) {
            Ok(()) => {
                println!("Generated and saved new SSH key to {}", key_file.display());
            }
            Err(e) => {
                eprintln!("Failed to save key file: {}", e);
            }
        }

        Ok(key)
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let key = Self::load_or_create_key()?;

        let config = Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![key],
            nodelay: true,
            ..Default::default()
        };

        self.run_on_address(Arc::new(config), ("0.0.0.0", 2222)).await?;
        Ok(())
    }
}

impl Server for AnathemaSSHServer {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl Handler for AnathemaSSHServer {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let terminal_handle = TerminalHandle::start(session.handle(), channel.id()).await;

        let backend = SSHBackend::new(terminal_handle.clone())?;

        self.clients
            .insert(self.id, (Arc::new(Mutex::new(backend)), terminal_handle));
        println!("New SSH client connected with ID: {}", self.id);

        let app_runner = self.app_runner.clone();
        let client_id = self.id;
        let backend_arc = self.clients.get(&client_id).unwrap().0.clone();

        tokio::spawn(async move {
            // Wait a bit to ensure the SSH session is fully established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let backend_clone = backend_arc.clone();

            // Run the app in a blocking task to not block the async runtime
            match tokio::task::spawn_blocking(move || {
                let mut backend = backend_clone.blocking_lock();
                eprintln!("Backend clone LOCK acquired");
                let mut app_runner = app_runner.blocking_lock();
                eprintln!("App Runner clone LOCK acquired");
                let r = (app_runner)(&mut backend);
                eprintln!("App runner task completed for client {}", client_id);
                r
            })
            .await
            {
                Ok(Ok(())) => { /* Success */ }
                Ok(Err(e)) => eprintln!("App runner failed: {}", e),
                Err(e) => eprintln!("App runner task failed: {}", e),
            }
        });
        Ok(true)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        eprintln!(
            "SSH data method called with {} bytes from client {}",
            data.len(),
            self.id
        );
        if let Some((_, terminal_handle)) = self.clients.get_mut(&self.id) {
            eprintln!("terminal_handle found");
            //let mut backend = backend_arc.lock().await;
            terminal_handle.push_input(data);
            eprintln!("Received {} bytes from client {}: {:?}", data.len(), self.id, data);
            //backend.output_mut().push_input(data);
        } else {
            eprintln!("Backend not found for client {}, input lost", self.id);
        }

        let data = CryptoVec::from(format!("Got data: {}\r\n", String::from_utf8_lossy(data)));
        session.data(channel, data)?;
        Ok(())
    }

    /// The client's window size has changed.
    async fn window_change_request(
        &mut self,
        _: ChannelId,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        let size = Size::new(col_width as u16, row_height as u16);

        if let Some((backend_arc, _)) = self.clients.get_mut(&self.id) {
            eprintln!("Client ID: {} requested window resize to: {:?}", self.id, size);
            let mut backend = backend_arc.lock().await;

            eprintln!("Backend clone LOCK acquired for client {}", self.id);
            backend.resize(size, &mut GlyphMap::empty());
        }

        Ok(())
    }

    /// The client requests a pseudo-terminal with the given
    /// specifications.
    ///
    /// **Note:** Success or failure should be communicated to the client by calling
    /// `session.channel_success(channel)` or `session.channel_failure(channel)` respectively.
    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let size = Size::new(col_width as u16, row_height as u16);

        if let Some((backend_arc, _)) = self.clients.get_mut(&self.id) {
            println!("Client ID: {} requested PTY with size: {:?}", self.id, size);
            let mut backend = backend_arc.lock().await;
            eprintln!("Backend clone LOCK acquired for client {}", self.id);
            backend.resize(size, &mut GlyphMap::empty());
        }

        session.channel_success(channel)?;

        Ok(())
    }
}

impl Drop for AnathemaSSHServer {
    fn drop(&mut self) {
        let id = self.id;
        self.clients.remove(&id);
    }
}
