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
use russh::server::*;
use russh::{Channel, ChannelId, Pty};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::sshbackend::SSHBackend;

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
        // Convert raw input bytes to Anathema events
        for &byte in data {
            match byte {
                b'\x1b' => {
                    // Escape sequence - for now, treat as escape key
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Esc,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\r' | b'\n' => {
                    // Enter key
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Enter,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\x7f' => {
                    // Backspace
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Backspace,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b'\t' => {
                    // Tab
                    self.events
                        .push_back(Event::Key(anathema_widgets::components::events::KeyEvent {
                            code: anathema_widgets::components::events::KeyCode::Tab,
                            ctrl: false,
                            state: anathema_widgets::components::events::KeyState::Press,
                        }));
                }
                b' '..=b'~' => {
                    // Printable ASCII characters
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
                }
            }
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop_front()
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
    clients: Arc<Mutex<HashMap<usize, Arc<Mutex<SSHBackend>>>>>,
    id: usize,
    app_runner: Arc<dyn Fn(SSHBackend) -> anyhow::Result<()> + Send + Sync>,
}

impl AnathemaSSHServer {
    pub fn new(app_runner: impl Fn(SSHBackend) -> anyhow::Result<()> + Send + Sync + 'static) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
            app_runner: Arc::new(app_runner),
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

        let backend = SSHBackend::new(terminal_handle)?;

        let mut clients = self.clients.lock().await;
        clients.insert(self.id, Arc::new(Mutex::new(backend)));
        println!("New SSH client connected with ID: {}", self.id);
        drop(clients); // Release lock early

        // Don't spawn the app task here - instead, spawn it after a small delay
        // to ensure the SSH session is fully established
        let app_runner = self.app_runner.clone();
        let clients_arc = self.clients.clone();
        let client_id = self.id;

        tokio::spawn(async move {
            // Wait a bit to ensure the SSH session is fully established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Get the backend from the clients map and remove it for exclusive access
            let mut clients = clients_arc.lock().await;
            if let Some(backend_arc) = clients.remove(&client_id) {
                drop(clients); // Release lock before running app

                // Extract the backend from the Arc<Mutex<>>
                let backend = Arc::try_unwrap(backend_arc)
                    .map_err(|_| "Failed to unwrap Arc")
                    .unwrap()
                    .into_inner();

                // Run the app in a blocking task to not block the async runtime
                match tokio::task::spawn_blocking(move || (app_runner)(backend)).await {
                    Ok(Ok(())) => { /* Success */ }
                    Ok(Err(e)) => eprintln!("App runner failed: {}", e),
                    Err(e) => eprintln!("App runner task failed: {}", e),
                }
            }
        });

        Ok(true)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn data(&mut self, _channel: ChannelId, data: &[u8], _session: &mut Session) -> Result<(), Self::Error> {
        let mut clients = self.clients.lock().await;
        if let Some(backend_arc) = clients.get_mut(&self.id) {
            let mut backend = backend_arc.lock().await;
            backend.output_mut().push_input(data);
        } else {
            eprintln!("Backend not found for client {}, input lost", self.id);
        }
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

        let mut clients = self.clients.lock().await;
        let backend_arc = clients.get_mut(&self.id).unwrap();
        let mut backend = backend_arc.lock().await;
        backend.resize(size, &mut GlyphMap::empty());

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

        let mut clients = self.clients.lock().await;
        println!("Client ID: {} requested PTY with size: {:?}", self.id, size);
        let backend_arc = clients.get_mut(&self.id).unwrap();
        let mut backend = backend_arc.lock().await;
        backend.resize(size, &mut GlyphMap::empty());

        session.channel_success(channel)?;

        Ok(())
    }
}

impl Drop for AnathemaSSHServer {
    fn drop(&mut self) {
        let id = self.id;
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut clients = clients.lock().await;
            clients.remove(&id);
        });
    }
}
