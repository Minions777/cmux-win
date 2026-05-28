use crate::ssh::{SshConfig, SshManager};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

/// Connect to SSH server
#[tauri::command]
pub async fn ssh_connect(
    id: String,
    config: SshConfig,
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<(), String> {
    let mut manager = ssh_manager.lock();
    manager.connect(id, config).await
}

/// Open PTY on SSH connection
#[tauri::command]
pub async fn ssh_open_pty(
    id: String,
    cols: u32,
    rows: u32,
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<(), String> {
    let mut manager = ssh_manager.lock();
    let session = manager.get_mut(&id).ok_or("Session not found")?;
    session.open_pty(cols, rows).await
}

/// Write to SSH terminal
#[tauri::command]
pub async fn ssh_write(
    id: String,
    data: Vec<u8>,
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<(), String> {
    let mut manager = ssh_manager.lock();
    let session = manager.get_mut(&id).ok_or("Session not found")?;
    session.write(&data).await
}

/// Resize SSH terminal
#[tauri::command]
pub async fn ssh_resize(
    id: String,
    cols: u32,
    rows: u32,
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<(), String> {
    let mut manager = ssh_manager.lock();
    let session = manager.get_mut(&id).ok_or("Session not found")?;
    session.resize(cols, rows).await
}

/// Disconnect SSH session
#[tauri::command]
pub async fn ssh_disconnect(
    id: String,
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<(), String> {
    let mut manager = ssh_manager.lock();
    manager.disconnect(&id).await;
    Ok(())
}

/// List SSH sessions
#[tauri::command]
pub async fn ssh_list(
    ssh_manager: State<'_, Arc<Mutex<SshManager>>>,
) -> Result<Vec<String>, String> {
    let manager = ssh_manager.lock();
    Ok(manager.list())
}
