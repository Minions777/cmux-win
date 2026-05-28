use serde::{Deserialize, Serialize};n
/// Terminal event types for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalEvent {
    /// Terminal output data
    Output {
        id: String,
        data: Vec<u8>,
    },
    /// Terminal state changed
    StateChanged {
        id: String,
        state: serde_json::Value,
    },
    /// Terminal closed
    Closed {
        id: String,
    },
    /// Notification (e.g., bell, long-running command finished)
    Notification {
        id: String,
        message: String,
    },
}

impl TerminalEvent {
    /// Get the event name for Tauri
    pub fn event_name(&self) -> &'static str {
        match self {
            TerminalEvent::Output { .. } => "terminal-output",
            TerminalEvent::StateChanged { .. } => "terminal-state-changed",
            TerminalEvent::Closed { .. } => "terminal-closed",
            TerminalEvent::Notification { .. } => "terminal-notification",
        }
    }
}
