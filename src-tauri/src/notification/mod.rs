use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use regex::Regex;

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub min_duration_secs: u64,
    pub watch_patterns: Vec<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            min_duration_secs: 5,
            watch_patterns: vec![
                r"\$\s*$".to_string(),       // Bash prompt
                r"%\s*$".to_string(),        // Zsh prompt
                r">\s*$".to_string(),        // CMD prompt
                r"PS [^>]*>\s*$".to_string(), // PowerShell prompt
            ],
        }
    }
}

/// Notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub urgency: NotificationUrgency,
    pub terminal_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

/// Command tracking state per terminal
struct TerminalTracking {
    command_start: Option<Instant>,
    last_output: Instant,
    is_running_command: bool,
    prompt_regex: Regex,
}

/// Notification service
pub struct NotificationService {
    config: NotificationConfig,
    terminals: Mutex<HashMap<String, TerminalTracking>>,
    bell_pattern: Regex,
    osc94_pattern: Regex,
}

impl NotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        let bell_pattern = Regex::new(r"\x07").unwrap();
        let osc94_pattern = Regex::new(r"\x1b\]9;4;.*?\x07").unwrap();

        Self {
            config,
            terminals: Mutex::new(HashMap::new()),
            bell_pattern,
            osc94_pattern,
        }
    }

    /// Register a terminal for tracking
    pub fn register_terminal(&self, terminal_id: String) {
        let prompt_regex = Regex::new(&self.config.watch_patterns.join("|")).unwrap();
        let tracking = TerminalTracking {
            command_start: None,
            last_output: Instant::now(),
            is_running_command: false,
            prompt_regex,
        };
        self.terminals.lock().insert(terminal_id, tracking);
    }

    /// Unregister a terminal
    pub fn unregister_terminal(&self, terminal_id: &str) {
        self.terminals.lock().remove(terminal_id);
    }

    /// Process terminal output and check for notifications
    pub fn process_output(&self, terminal_id: &str, data: &str) -> Option<Notification> {
        if !self.config.enabled {
            return None;
        }

        let mut terminals = self.terminals.lock();
        let tracking = terminals.get_mut(terminal_id)?;

        // Check for BEL character (\x07) - explicit notification request
        if self.bell_pattern.is_match(data) {
            return Some(Notification {
                title: "Terminal Alert".to_string(),
                body: format!("Terminal {} sent a bell signal", terminal_id),
                icon: None,
                urgency: NotificationUrgency::Normal,
                terminal_id: terminal_id.to_string(),
                timestamp: chrono_now(),
            });
        }

        // Check for OSC 9;4 sequence (ConEmu-style progress/completion)
        if self.osc94_pattern.is_match(data) {
            return Some(Notification {
                title: "Task Complete".to_string(),
                body: format!("Terminal {} completed a task", terminal_id),
                icon: None,
                urgency: NotificationUrgency::Low,
                terminal_id: terminal_id.to_string(),
                timestamp: chrono_now(),
            });
        }

        // Track command execution
        tracking.last_output = Instant::now();

        // Check if a command is starting (non-empty line after prompt)
        if !tracking.is_running_command {
            // If we see input that's not a prompt, a command is starting
            let lines: Vec<&str> = data.lines().collect();
            for line in &lines {
                if !line.trim().is_empty() && !tracking.prompt_regex.is_match(line) {
                    tracking.is_running_command = true;
                    tracking.command_start = Some(Instant::now());
                    break;
                }
            }
        }

        // Check if command completed (prompt reappeared)
        if tracking.is_running_command {
            let lines: Vec<&str> = data.lines().collect();
            for line in lines.iter().rev() {
                if tracking.prompt_regex.is_match(line) {
                    let duration = tracking.command_start
                        .map(|start| start.elapsed())
                        .unwrap_or(Duration::ZERO);

                    tracking.is_running_command = false;
                    tracking.command_start = None;

                    // Only notify if command ran long enough
                    if duration >= Duration::from_secs(self.config.min_duration_secs) {
                        let duration_str = format_duration(duration);
                        return Some(Notification {
                            title: "Command Complete".to_string(),
                            body: format!("Finished in {}", duration_str),
                            icon: None,
                            urgency: NotificationUrgency::Normal,
                            terminal_id: terminal_id.to_string(),
                            timestamp: chrono_now(),
                        });
                    }
                    break;
                }
            }
        }

        None
    }

    /// Update configuration
    pub fn update_config(&mut self, config: NotificationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &NotificationConfig {
        &self.config
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Send a Windows Toast notification
#[cfg(windows)]
pub fn send_notification(notification: &Notification) -> Result<(), String> {
    // Use Windows Toast notification API
    // This is a simplified version - in production, use a proper toast library
    log::info!(
        "Notification: {} - {}",
        notification.title,
        notification.body
    );

    // For now, we'll use a simple approach
    // In production, use: windows-rs ToastNotificationManager
    // or the tauri-plugin-notification
    Ok(())
}

#[cfg(not(windows))]
pub fn send_notification(notification: &Notification) -> Result<(), String> {
    log::info!(
        "Notification: {} - {}",
        notification.title,
        notification.body
    );
    Ok(())
}
