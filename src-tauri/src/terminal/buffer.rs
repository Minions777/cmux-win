use serde::{Deserialize, Serialize};

/// Represents a single cell in the terminal grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub char: String,
    pub fg: u8,
    pub bg: u8,
    pub flags: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: " ".to_string(),
            fg: 0,
            bg: 0,
            flags: 0,
        }
    }
}

/// Screen buffer with scrollback
pub struct ScreenBuffer {
    /// Active lines (visible area)
    pub lines: Vec<Vec<Cell>>,
    /// Scrollback buffer
    pub scrollback: Vec<Vec<Cell>>,
    /// Maximum scrollback lines
    pub max_scrollback: usize,
    /// Width and height
    pub cols: u16,
    pub rows: u16,
}

impl ScreenBuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        let lines = (0..rows)
            .map(|_| (0..cols).map(|_| Cell::default()).collect())
            .collect();

        Self {
            lines,
            scrollback: Vec::new(),
            max_scrollback: 10000,
            cols,
            rows,
        }
    }

    /// Resize the buffer
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;

        // Resize existing lines
        for line in &mut self.lines {
            line.resize(cols as usize, Cell::default());
        }
        self.lines.resize(rows as usize, (0..cols).map(|_| Cell::default()).collect());
    }

    /// Get a mutable reference to a cell
    pub fn cell_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        self.lines
            .get_mut(row as usize)
            .and_then(|line| line.get_mut(col as usize))
    }

    /// Get a reference to a cell
    pub fn cell(&self, row: u16, col: u16) -> Option<&Cell> {
        self.lines
            .get(row as usize)
            .and_then(|line| line.get(col as usize))
    }

    /// Scroll the buffer up by one line
    pub fn scroll_up(&mut self) {
        if let Some(line) = self.lines.first().cloned() {
            self.scrollback.push(line);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
        }
        self.lines.remove(0);
        self.lines.push((0..self.cols).map(|_| Cell::default()).collect());
    }

    /// Clear the entire buffer
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            for cell in line {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to end of line
    pub fn clear_to_eol(&mut self, row: u16, col: u16) {
        if let Some(line) = self.lines.get_mut(row as usize) {
            for cell in line.iter_mut().skip(col as usize) {
                *cell = Cell::default();
            }
        }
    }

    /// Convert to JSON for frontend
    pub fn to_json(&self) -> serde_json::Value {
        let lines: Vec<Vec<serde_json::Value>> = self.lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|cell| {
                        serde_json::json!({
                            "char": cell.char,
                            "fg": cell.fg,
                            "bg": cell.bg,
                            "flags": cell.flags,
                        })
                    })
                    .collect()
            })
            .collect();

        serde_json::json!({
            "lines": lines,
            "cols": self.cols,
            "rows": self.rows,
        })
    }
}
