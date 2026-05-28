mod commands;
mod event;
mod terminal;

use commands::terminal::{create_terminal, close_terminal, write_to_terminal, resize_terminal};
use commands::tab::{get_tabs, get_terminal_state};
use terminal::TerminalManager;

use parking_lot::Mutex;
use std::sync::Arc;
use tauri::Manager;

/// Application state shared across all commands
pub struct AppState {
    pub terminal_manager: Arc<Mutex<TerminalManager>>,
}

pub fn run() {
    env_logger::init();

    let terminal_manager = Arc::new(Mutex::new(TerminalManager::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            terminal_manager: terminal_manager.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            create_terminal,
            close_terminal,
            write_to_terminal,
            resize_terminal,
            get_tabs,
            get_terminal_state,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let tm = terminal_manager.clone();

            // Spawn a background task to read terminal output
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60fps

                    let updates = {
                        let mut manager = tm.lock();
                        manager.drain_output()
                    };

                    for (id, data) in updates {
                        let _ = handle.emit("terminal-output", serde_json::json!({
                            "id": id,
                            "data": data,
                        }));
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running cmux-win");
}
