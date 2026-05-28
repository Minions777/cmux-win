pub mod buffer;
pub mod parser;
pub mod pty;
pub mod state;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use self::pty::PtyHandle;
use self::state::TerminalState;

/// A single terminal instance
pub struct TerminalInstance {
    pub id: String,
    pub state: TerminalState,
    pub pty: PtyHandle,
    pub title: String,
    pub cwd: String,
    pub output_buffer: Vec<u8>,
}

/// Manages all terminal instances
pub struct TerminalManager {
    terminals: HashMap<String, Arc<Mutex<TerminalInstance>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: HashMap::new(),
        }
    }

    /// Create a new terminal instance
    pub fn create(&mut self, shell: Option<&str>, cwd: Option<&str>) -> Result<(String, String, String), String> {
        let id = uuid::Uuid::new_v4().to_string();

        // Determine shell
        let shell_cmd = shell.unwrap_or({
            if cfg!(windows) {
                "powershell.exe"
            } else {
                "bash"
            }
        });

        // Determine working directory
        let working_dir = cwd.unwrap_or({
            if cfg!(windows) {
                "C:\\Users"
            } else {
                "/home"
            }
        });

        // Create PTY
        let pty = PtyHandle::new(shell_cmd, working_dir)
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        let title = format!("{}", shell_cmd);

        let state = TerminalState::new(80, 24);

        let instance = TerminalInstance {
            id: id.clone(),
            state,
            pty,
            title: title.clone(),
            cwd: working_dir.to_string(),
            output_buffer: Vec::new(),
        };

        self.terminals.insert(id.clone(), Arc::new(Mutex::new(instance)));

        log::info!("Created terminal {} with shell '{}'", id, shell_cmd);

        Ok((id, title, working_dir.to_string()))
    }

    /// Close a terminal instance
    pub fn close(&mut self, id: &str) -> Result<(), String> {
        if let Some(instance) = self.terminals.remove(id) {
            let mut inst = instance.lock();
            inst.pty.kill().ok();
            log::info!("Closed terminal {}", id);
            Ok(())
        } else {
            Err(format!("Terminal {} not found", id))
        }
    }

    /// Write data to a terminal's stdin
    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), String> {
        if let Some(instance) = self.terminals.get(id) {
            let mut inst = instance.lock();
            inst.pty.write(data).map_err(|e| e.to_string())
        } else {
            Err(format!("Terminal {} not found", id))
        }
    }

    /// Resize a terminal
    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        if let Some(instance) = self.terminals.get(id) {
            let mut inst = instance.lock();
            inst.pty.resize(cols, rows).map_err(|e| e.to_string())?;
            inst.state.resize(cols, rows);
            Ok(())
        } else {
            Err(format!("Terminal {} not found", id))
        }
    }

    /// Drain all pending output from all terminals
    pub fn drain_output(&mut self) -> Vec<(String, Vec<u8>)> {
        let mut results = Vec::new();

        for (id, instance) in &self.terminals {
            let mut inst = instance.lock();
            let data = inst.pty.read_available();
            if !data.is_empty() {
                // Parse VT sequences and update state
                inst.state.process_input(&data);
                results.push((id.clone(), data));
            }
        }

        results
n    }

    /// Get all terminal IDs
    pub fn get_ids(&self) -> Vec<String> {
        self.terminals.keys().cloned().collect()
    }

    /// Get terminal state as JSON
    pub fn get_state_json(&self, id: &str) -> Result<serde_json::Value, String> {
        if let Some(instance) = self.terminals.get(id) {
            let inst = instance.lock();
            Ok(inst.state.to_json())
        } else {
            Err(format!("Terminal {} not found", id))
        }
    }
}
