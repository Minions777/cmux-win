use russh::client::{self, Handler, Session};
use russh::{Channel, ChannelMsg, Disconnect};
use russh_keys::{key, load_secret_key};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;

/// SSH connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SshAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshAuth {
    Password(String),
    Key { path: String, passphrase: Option<String> },
    Agent,
}

/// SSH connection handle
pub struct SshSession {
    id: String,
    config: SshConfig,
    session: Option<Arc<Mutex<client::Handle<SshClient>>>>,
    channel: Option<Channel<client::Msg>>,
    output_tx: mpsc::UnboundedSender<Vec<u8>>,
    output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

struct SshClient {
    id: String,
}

#[async_trait::async_trait]
impl Handler for SshClient {
    type Error = russh::Error;

    async fn check_server_key(
        &self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: Implement known_hosts checking
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Forward data to output channel
        // This would need a reference to the output sender
        Ok(())
    }
}

impl SshSession {
    /// Create a new SSH session
    pub async fn new(id: String, config: SshConfig) -> Result<Self, String> {
        let (output_tx, output_rx) = mpsc::unbounded_channel();

        Ok(Self {
            id,
            config,
            session: None,
            channel: None,
            output_tx,
            output_rx,
        })
    }

    /// Connect to SSH server
    pub async fn connect(&mut self) -> Result<(), String> {
        let ssh_config = client::Config::default();
        let config = Arc::new(ssh_config);

        let mut session = client::connect(
            config,
            (self.config.host.as_str(), self.config.port),
            SshClient { id: self.id.clone() },
        )
        .await
        .map_err(|e| format!("SSH connect failed: {}", e))?;

        // Authenticate
        match &self.config.auth_method {
            SshAuth::Password(password) => {
                let auth = session
                    .authenticate_password(&self.config.username, password)
                    .await
                    .map_err(|e| format!("Auth failed: {}", e))?;
                if !auth {
                    return Err("Password authentication failed".to_string());
                }
            }
            SshAuth::Key { path, passphrase } => {
                let key_path = PathBuf::from(path);
                let key_pair = load_secret_key(&key_path, passphrase.as_deref())
                    .map_err(|e| format!("Failed to load key: {}", e))?;
                let auth = session
                    .authenticate_publickey(&self.config.username, Arc::new(key_pair))
                    .await
                    .map_err(|e| format!("Auth failed: {}", e))?;
                if !auth {
                    return Err("Key authentication failed".to_string());
                }
            }
            SshAuth::Agent => {
                // TODO: Implement SSH agent authentication
                return Err("SSH agent auth not yet implemented".to_string());
            }
        }

        self.session = Some(Arc::new(Mutex::new(session)));
        Ok(())
    }

    /// Open a PTY channel
    pub async fn open_pty(&mut self, cols: u32, rows: u32) -> Result<(), String> {
        let session = self.session.as_ref()
            .ok_or("Not connected")?;

        let mut session_guard = session.lock();
        let mut channel = session_guard
            .channel_open_session()
            .await
            .map_err(|e| format!("Failed to open channel: {}", e))?;

        channel
            .request_pty(
                true,
                "xterm-256color",
                cols,
                rows,
                0,
                0,
                &[],
            )
            .await
            .map_err(|e| format!("PTY request failed: {}", e))?;

        channel
            .request_shell(true)
            .await
            .map_err(|e| format!("Shell request failed: {}", e))?;

        self.channel = Some(channel);
        Ok(())
    }

    /// Write data to SSH channel
    pub async fn write(&mut self, data: &[u8]) -> Result<(), String> {
        let channel = self.channel.as_mut()
            .ok_or("No channel open")?;

        channel
            .data(data)
            .await
            .map_err(|e| format!("Write failed: {}", e))
    }

    /// Read data from SSH channel (non-blocking)
    pub async fn read(&mut self) -> Option<Vec<u8>> {
        self.output_rx.try_recv().ok()
    }

    /// Resize the PTY
    pub async fn resize(&mut self, cols: u32, rows: u32) -> Result<(), String> {
        let channel = self.channel.as_mut()
            .ok_or("No channel open")?;

        channel
            .window_change(cols as u32, rows as u32, 0, 0)
            .await
            .map_err(|e| format!("Resize failed: {}", e))
    }

    /// Disconnect
    pub async fn disconnect(&mut self) {
        if let Some(channel) = self.channel.take() {
            let _ = channel.close().await;
        }
        if let Some(session) = self.session.take() {
            let mut session_guard = session.lock();
            let _ = session_guard
                .disconnect(Disconnect::ByApplication, "", "")
                .await;
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }
}

/// SSH session manager
pub struct SshManager {
    sessions: HashMap<String, SshSession>,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create and connect a new SSH session
    pub async fn connect(\mut self, id: String, config: SshConfig) -> Result<(), String> {
        let mut session = SshSession::new(id.clone(), config).await?;
        session.connect().await?;
        self.sessions.insert(id, session);
        Ok(())
    }

    /// Get a session by ID
    pub fn get(&self, id: &str) -> Option<&SshSession> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SshSession> {
        self.sessions.get_mut(id)
    }

    /// Disconnect and remove a session
    pub async fn disconnect(&mut self, id: &str) {
        if let Some(mut session) = self.sessions.remove(id) {
            session.disconnect().await;
        }
    }

    /// List all session IDs
    pub fn list(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }
}
