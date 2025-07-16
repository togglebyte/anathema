use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anathema_backend::Backend;
use anathema_backend::tui::TuiBackend;
use anathema_geometry::Size;
use anathema_widgets::GlyphMap;
use rand_core::OsRng;
use russh::keys::ssh_key::{self, PublicKey};
use russh::server::*;
use russh::{Channel, ChannelId, Pty};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub struct TerminalHandle {
    sender: UnboundedSender<Vec<u8>>,
    // The sink collects the data which is finally sent to sender.
    sink: Vec<u8>,
}

impl TerminalHandle {
    async fn start(handle: Handle, channel_id: ChannelId) -> Self {
        let (sender, mut receiver) = unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                let result = handle.data(channel_id, data.into()).await;
                if result.is_err() {
                    eprintln!("Failed to send data: {:?}", result);
                }
            }
        });
        Self {
            sender,
            sink: Vec::new(),
        }
    }
}

// The crossterm backend writes to the terminal handle.
impl std::io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result = self.sender.send(self.sink.clone());
        if result.is_err() {
            return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, result.unwrap_err()));
        }

        self.sink.clear();
        Ok(())
    }
}

#[derive(Clone)]
pub struct AnathemaSSHServer {
    clients: Arc<Mutex<HashMap<usize, Arc<Mutex<TuiBackend<TerminalHandle>>>>>>,
    id: usize,
    app_runner: Arc<dyn Fn(TuiBackend<TerminalHandle>) -> anyhow::Result<()> + Send + Sync>,
}

impl AnathemaSSHServer {
    pub fn new(app_runner: impl Fn(TuiBackend<TerminalHandle>) -> anyhow::Result<()> + Send + Sync + 'static) -> Self {
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

        let mut backend = TuiBackend::builder_with_output(terminal_handle)
            .enable_alt_screen()
            .enable_raw_mode()
            .hide_cursor()
            .finish()?;

        backend.finalize();

        let mut clients = self.clients.lock().await;
        clients.insert(self.id, Arc::new(Mutex::new(backend)));
        println!("New SSH client connected with ID: {}", self.id);
        drop(clients); // Release lock early

        // Run the app function in a separate thread
        let app_runner = self.app_runner.clone();
        let clients_arc = self.clients.clone();
        let client_id = self.id;

        tokio::spawn(async move {
            // Wait a bit to ensure the backend is set up
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Get the backend from the clients map and remove it for exclusive access
            let mut clients = clients_arc.lock().await;
            if let Some(backend_arc) = clients.remove(&client_id) {
                drop(clients); // Release lock before running app

                // Extract the backend from the Arc<Mutex<>>
                let backend = Arc::try_unwrap(backend_arc)
                    .map_err(|_| "Failed to unwrap Arc")
                    .unwrap()
                    .into_inner();

                if let Err(e) = (app_runner)(backend) {
                    eprintln!("App runner failed: {}", e);
                }
            }
        });

        Ok(true)
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        let mut clients = self.clients.lock().await;
        let backend_arc = clients.get_mut(&self.id).unwrap();
        let mut backend = backend_arc.lock().await;

        backend.output().sink.extend_from_slice(data);
        let _ = backend.output().flush();

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
