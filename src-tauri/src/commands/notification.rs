use crate::notification::{NotificationService, NotificationConfig, Notification};
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::State;

/// Get notification configuration
#[tauri::command]
pub async fn notification_get_config(
    notification_service: State<'_, Arc<Mutex<NotificationService>>>,
) -> Result<NotificationConfig, String> {
    let service = notification_service.lock();
    Ok(service.get_config().clone())
}

/// Update notification configuration
#[tauri::command]
pub async fn notification_update_config(
    config: NotificationConfig,
    notification_service: State<'_, Arc<Mutex<NotificationService>>>,
) -> Result<(), String> {
    let mut service = notification_service.lock();
    service.update_config(config);
    Ok(())
}

/// Register terminal for notification tracking
#[tauri::command]
pub async fn notification_register_terminal(
    terminal_id: String,
    notification_service: State<'_, Arc<Mutex<NotificationService>>>,
) -> Result<(), String> {
    let service = notification_service.lock();
    service.register_terminal(terminal_id);
    Ok(())
}

/// Unregister terminal from notification tracking
#[tauri::command]
pub async fn notification_unregister_terminal(
    terminal_id: String,
    notification_service: State<'_, Arc<Mutex<NotificationService>>>,
) -> Result<(), String> {
    let service = notification_service.lock();
    service.unregister_terminal(&terminal_id);
    Ok(())
}

/// Send a test notification
#[tauri::command]
pub async fn notification_test() -> Result<(), String> {
    let notification = Notification {
        title: "Test Notification".to_string(),
        body: "cmux-win notification system is working!".to_string(),
        icon: None,
        urgency: crate::notification::NotificationUrgency::Normal,
        terminal_id: "test".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    };
    crate::notification::send_notification(&notification)
}
