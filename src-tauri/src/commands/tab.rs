use crate::AppState;
tauri::command;

/// Get all terminal tab IDs
#[command]
pub fn get_tabs(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state.terminal_manager.lock();
    Ok(manager.get_ids())
}

/// Get terminal state as JSON
#[command]
pub fn get_terminal_state(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, String> {
    let manager = state.terminal_manager.lock();
    manager.get_state_json(&id)
}
