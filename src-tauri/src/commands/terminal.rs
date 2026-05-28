use crate::AppState;
tauri::command;

/// Create a new terminal instance
#[command]
pub fn create_terminal(
    state: tauri::State<'_, AppState>,
    shell: Option<String>,
    cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut manager = state.terminal_manager.lock();
    let (id, title, cwd) = manager.create(shell.as_deref(), cwd.as_deref())?;

    Ok(serde_json::json!({
        "id": id,
        "title": title,
        "cwd": cwd,
    }))
}

/// Close a terminal instance
#[command]
pub fn close_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut manager = state.terminal_manager.lock();
    manager.close(&id)
}

/// Write data to a terminal's stdin
#[command]
pub fn write_to_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
    data: String,
) -> Result<(), String> {
    let manager = state.terminal_manager.lock();
    manager.write(&id, data.as_bytes())
}

/// Resize a terminal
#[command]
pub fn resize_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let manager = state.terminal_manager.lock();
    manager.resize(&id, cols, rows)
}
