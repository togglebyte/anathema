use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_widgets::GlyphMap;
use crossterm::QueueableCommand;
use crossterm::event::EnableMouseCapture;
use rand_core::OsRng;
use russh::keys::ssh_key::{self, PublicKey};
use russh::{Channel, ChannelId, Pty};
use russh::{CryptoVec, server::*};
use tokio::sync::Mutex;

use crate::sshbackend::SSHBackend;
use crate::terminalhandle::TerminalHandle;

pub struct AnathemaSSHServerBuilder {
    app_runner_factory:
        Arc<dyn Fn() -> Box<dyn FnMut(&mut SSHBackend) -> anyhow::Result<()> + Send + Sync> + Send + Sync>,
    mouse_enabled: bool,
}

impl AnathemaSSHServerBuilder {
    pub fn runtime_factory<F>(mut self, app_runner: F) -> Self
    where
        F: Fn() -> Box<dyn FnMut(&mut SSHBackend) -> anyhow::Result<()> + Send + Sync> + Send + Sync + 'static,
    {
        self.app_runner_factory = Arc::new(app_runner);
        self
    }

    pub fn enable_mouse(mut self) -> Self {
        self.mouse_enabled = true;
        self
    }

    pub fn build(self) -> AnathemaSSHServer {
        AnathemaSSHServer {
            clients: HashMap::new(),
            id: 0,
            app_runner_factory: self.app_runner_factory,
            mouse_enabled: self.mouse_enabled,
        }
    }
}

#[derive(Clone)]
pub struct AnathemaSSHServer {
    clients: HashMap<usize, (Arc<Mutex<SSHBackend>>, TerminalHandle)>,
    id: usize,
    app_runner_factory:
        Arc<dyn Fn() -> Box<dyn FnMut(&mut SSHBackend) -> anyhow::Result<()> + Send + Sync> + Send + Sync>,

    mouse_enabled: bool,
}

impl AnathemaSSHServer {
    pub fn builder() -> AnathemaSSHServerBuilder {
        AnathemaSSHServerBuilder {
            mouse_enabled: false,
            app_runner_factory: Arc::new(|| Box::new(|_| Ok(()))),
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

        let app_runner_factory = self.app_runner_factory.clone();
        let client_id = self.id;
        let backend_arc = self.clients.get(&client_id).unwrap().0.clone();

        tokio::spawn(async move {
            // Wait a bit to ensure the SSH session is fully established
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let backend_clone = backend_arc.clone();

            match tokio::task::spawn_blocking(move || {
                let mut backend = backend_clone.blocking_lock();
                let mut app_runner = (app_runner_factory)();
                let r = (app_runner)(&mut backend);
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

    async fn data(&mut self, _channel: ChannelId, data: &[u8], _session: &mut Session) -> Result<(), Self::Error> {
        eprintln!("Received {} bytes from client {}", data.len(), self.id);
        if let Some((_, terminal_handle)) = self.clients.get_mut(&self.id) {
            terminal_handle.push_input(data);
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

        if let Some((backend_arc, _)) = self.clients.get_mut(&self.id) {
            let mut backend = backend_arc.lock().await;
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
            let mut backend = backend_arc.lock().await;
            backend.resize(size, &mut GlyphMap::empty());
        }

        session.channel_success(channel)?;

        if self.mouse_enabled {
            let mut mouse_enable_buffer = Vec::new();
            mouse_enable_buffer.queue(EnableMouseCapture)?;

            let data = CryptoVec::from(mouse_enable_buffer);
            session.data(channel, data)?;
        }

        Ok(())
    }
}

impl Drop for AnathemaSSHServer {
    fn drop(&mut self) {
        let id = self.id;
        self.clients.remove(&id);
    }
}
