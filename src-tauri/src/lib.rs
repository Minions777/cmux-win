mod commands;
mod event;
mod git;
mod notification;
mod ssh;
mod terminal;

use commands::terminal::{create_terminal, close_terminal, write_to_terminal, resize_terminal};
use commands::tab::{get_tabs, get_terminal_state};
use commands::git::{git_get_status, git_refresh_status, git_get_branches, git_clear_cache};
use commands::ssh::{ssh_connect, ssh_open_pty, ssh_write, ssh_resize, ssh_disconnect, ssh_list};
use commands::notification::{
    notification_get_config, notification_update_config,
    notification_register_terminal, notification_unregister_terminal,
    notification_test,
};
use git::GitService;
use notification::{NotificationService, NotificationConfig};
use ssh::SshManager;
use terminal::TerminalManager;

use parking_lot::Mutex;
use std::sync::Arc;
use tauri::Manager;

/// Application state shared across all commands
pub struct AppState {
    pub terminal_manager: Arc<Mutex<TerminalManager>>,
    pub git_service: Arc<GitService>,
    pub ssh_manager: Arc<Mutex<SshManager>>,
    pub notification_service: Arc<Mutex<NotificationService>>,
}

pub fn run() {
    env_logger::init();

    let terminal_manager = Arc::new(Mutex::new(TerminalManager::new()));
    let git_service = Arc::new(GitService::new());
    let ssh_manager = Arc::new(Mutex::new(SshManager::new()));
    let notification_service = Arc::new(Mutex::new(NotificationService::new(
        NotificationConfig::default(),
    )));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            terminal_manager: terminal_manager.clone(),
            git_service: git_service.clone(),
            ssh_manager: ssh_manager.clone(),
            notification_service: notification_service.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            // Terminal commands
            create_terminal,
            close_terminal,
            write_to_terminal,
            resize_terminal,
            get_tabs,
            get_terminal_state,
            // Git commands
            git_get_status,
            git_refresh_status,
            git_get_branches,
            git_clear_cache,
            // SSH commands
            ssh_connect,
            ssh_open_pty,
            ssh_write,
            ssh_resize,
            ssh_disconnect,
            ssh_list,
            // Notification commands
            notification_get_config,
            notification_update_config,
            notification_register_terminal,
            notification_unregister_terminal,
            notification_test,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let tm = terminal_manager.clone();
            let ns = notification_service.clone();

            // Spawn a background task to read terminal output
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60fps

                    let updates = {
                        let mut manager = tm.lock();
                        manager.drain_output()
                    };

                    for (id, data) in updates {
                        // Check for notifications
                        let notification = {
                            let service = ns.lock();
                            service.process_output(&id, &data)
                        };

                        // Send notification if triggered
                        if let Some(notif) = notification {
                            let _ = notification::send_notification(&notif);
                            let _ = handle.emit("notification", serde_json::json!({
                                "title": notif.title,
                                "body": notif.body,
                                "urgency": notif.urgency,
                                "terminal_id": notif.terminal_id,
                            }));
                        }

                        // Send terminal output
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
