use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anathema_geometry::Size;
use anathema_widgets::components::events::Event;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::{QueueableCommand, cursor, terminal};
use rand_core::OsRng;
use russh::keys::ssh_key::{self, PublicKey};
use russh::{Channel, ChannelId, Pty};
use russh::{CryptoVec, server::*};

use crate::error::Error;
use crate::error::Result;
use crate::sshbackend::SSHBackend;
use crate::terminalhandle::TerminalHandle;

type RuntimeFactory = Arc<dyn Fn(&mut SSHBackend, Option<PublicKey>) -> anathema_runtime::Result<()> + Send + Sync>;

pub struct AnathemaSSHServerBuilder {
    runtime_factory: Option<RuntimeFactory>,
    mouse_enabled: bool,
    ssh_key_folder: Option<PathBuf>,
}

impl AnathemaSSHServerBuilder {
    /// Set the application runner factory.
    /// This factory is used to create new application instances for each client connection.
    /// The factory should return a closure that takes a mutable reference to `SSHBackend`.
    pub fn runtime_factory<F>(mut self, app_runner: F) -> Self
    where
        F: Fn(&mut SSHBackend, Option<PublicKey>) -> anathema_runtime::Result<()> + Send + Sync + 'static,
    {
        self.runtime_factory = Some(Arc::new(app_runner));
        self
    }

    /// Enable mouse support.
    pub fn enable_mouse(mut self) -> Self {
        self.mouse_enabled = true;
        self
    }

    /// Folder to store SSH keys
    /// Defaults to ".ssh_keys" in the current directory if not set
    pub fn ssh_key_folder<P: AsRef<Path>>(mut self, folder: P) -> Self {
        self.ssh_key_folder = Some(folder.as_ref().to_path_buf());
        self
    }

    /// Build the SSH server with the provided configuration.
    pub fn build(self) -> AnathemaSSHServer {
        if self.runtime_factory.is_none() {
            panic!("AnathemaSSHServerBuilder requires an app runner factory to be set");
        }
        AnathemaSSHServer {
            next_id: 0,
            runtime_factory: self.runtime_factory.unwrap(),
            mouse_enabled: self.mouse_enabled,
            ssh_key_folder: self.ssh_key_folder.unwrap_or_else(|| PathBuf::from(".ssh_keys")),
        }
    }
}

pub struct AnathemaSSHServer {
    /// Unique identifier for the next client
    next_id: usize,
    /// Factory for creating new application instances
    /// This allows the server to spawn new applications for each client connection
    runtime_factory: RuntimeFactory,
    /// Whether mouse support is enabled
    mouse_enabled: bool,
    /// Folder to store SSH keys
    /// Defaults to ".ssh_keys" in the current directory if not set
    ssh_key_folder: PathBuf,
}

impl AnathemaSSHServer {
    pub fn builder() -> AnathemaSSHServerBuilder {
        AnathemaSSHServerBuilder {
            mouse_enabled: false,
            runtime_factory: None,
            ssh_key_folder: None,
        }
    }

    /// Load or create a persistent SSH key
    fn load_or_create_key(&mut self) -> Result<russh::keys::PrivateKey> {
        let key_dir = Path::new(&self.ssh_key_folder);
        let key_file = key_dir.join("ssh_host_ed25519_key");

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

    pub async fn run(&mut self) -> Result<()> {
        let key = self.load_or_create_key()?;

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
    type Handler = ClientHandler;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self::Handler {
        self.next_id += 1;
        ClientHandler {
            public_key: None,
            id: self.next_id,
            runtime_factory: self.runtime_factory.clone(),
            terminal_handle: None,
            mouse_enabled: self.mouse_enabled,
        }
    }
}

pub struct ClientHandler {
    /// Terminal handle for the client, used to send events and input
    terminal_handle: Option<TerminalHandle>,
    /// Unique identifier for this client
    id: usize,
    /// Factory for creating new application instances
    runtime_factory: RuntimeFactory,
    /// Whether mouse support is enabled
    mouse_enabled: bool,
    /// Public key of the client, if authenticated
    public_key: Option<PublicKey>,
}

impl Handler for ClientHandler {
    type Error = Error;

    /// Handle a new SSH client connection
    async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool> {
        let terminal_handle = TerminalHandle::start(session.handle(), channel.id()).await;

        let backend = SSHBackend::new(terminal_handle.clone())?;

        let backend_arc = Arc::new(tokio::sync::Mutex::new(backend));

        let runtime_factory = self.runtime_factory.clone();

        let public_key = self.public_key.clone();
        tokio::spawn({
            let backend_arc = backend_arc.clone();
            let runtime_factory = runtime_factory.clone();
            async move {
                // Wait a bit to ensure the SSH session with pty is fully established
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                let backend_clone = backend_arc.clone();

                match tokio::task::spawn_blocking(move || {
                    let mut backend = backend_clone.blocking_lock();
                    (runtime_factory)(&mut backend, public_key)
                })
                .await
                {
                    Err(e) => eprintln!("App runner task failed: {}", e),
                    _ => {}
                }
            }
        });

        Ok(true)
    }

    async fn auth_publickey(&mut self, _: &str, public_key: &PublicKey) -> Result<Auth> {
        self.public_key = Some(public_key.clone());
        Ok(Auth::Accept)
    }

    /// Handle raw input data from the client.
    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        if data.len() == 1 {
            if data[0] == 3 {
                let mut buf = Vec::new();

                if self.mouse_enabled {
                    buf.queue(DisableMouseCapture)?;
                }

                buf.queue(cursor::Show)?;
                buf.queue(terminal::LeaveAlternateScreen)?;

                let data = CryptoVec::from(buf);
                session.data(channel, data)?;
                session.flush()?;
                session.close(channel)?;
            }
        }
        if let Some(terminal_handle) = self.terminal_handle.as_mut() {
            terminal_handle.push_input(data);
        } else {
            eprintln!("Backend not found for client {}, input lost", self.id);
        }

        Ok(())
    }

    async fn channel_close(&mut self, _channel: ChannelId, _session: &mut Session) -> Result<()> {
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
    ) -> Result<()> {
        let size = Size::new(col_width as u16, row_height as u16);

        if let Some(terminal_handle) = self.terminal_handle.as_mut() {
            terminal_handle.push_event(Event::Resize(size));
        }

        Ok(())
    }

    /// The client requests a pty (pseudo-terminal) session.
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
    ) -> Result<()> {
        let size = Size::new(col_width as u16, row_height as u16);

        if let Some(terminal_handle) = self.terminal_handle.as_mut() {
            terminal_handle.push_event(Event::Resize(size));
        }

        let mut buf = Vec::new();

        if self.mouse_enabled {
            buf.queue(EnableMouseCapture)?;
        }

        buf.queue(cursor::Hide)?;
        buf.queue(terminal::EnterAlternateScreen)?;

        let data = CryptoVec::from(buf);
        session.data(channel, data)?;

        session.channel_success(channel)?;
        Ok(())
    }
}
