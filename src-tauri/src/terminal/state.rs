use super::buffer::{Cell, ScreenBuffer};
use serde::{Deserialize, Serialize};
use vte::{Parser, Perform};

/// Cursor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub row: u16,
    pub col: u16,
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            visible: true,
        }
    }
}

/// Text attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct TextAttrs {
    pub fg: u8,
    pub bg: u8,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub strikethrough: bool,
}

impl TextAttrs {
    pub fn to_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.bold { flags |= 1; }
        if self.italic { flags |= 2; }
        if self.underline { flags |= 4; }
        if self.inverse { flags |= 8; }
        if self.strikethrough { flags |= 16; }
        flags
    }
}

/// Terminal state
pub struct TerminalState {
    pub buffer: ScreenBuffer,
    pub cursor: Cursor,
    pub attrs: TextAttrs,
    pub parser: Parser,
    pub saved_cursor: Cursor,
    pub scroll_top: u16,
    pub scroll_bottom: u16,
}

impl TerminalState {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            buffer: ScreenBuffer::new(cols, rows),
            cursor: Cursor::default(),
            attrs: TextAttrs::default(),
            parser: Parser::new(),
            saved_cursor: Cursor::default(),
            scroll_top: 0,
            scroll_bottom: rows - 1,
        }
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.buffer.resize(cols, rows);
        self.scroll_bottom = rows - 1;
        if self.cursor.row >= rows {
            self.cursor.row = rows - 1;
        }
        if self.cursor.col >= cols {
            self.cursor.col = cols - 1;
        }
    }

    /// Process raw input data through VT parser
    pub fn process_input(&mut self, data: &[u8]) {
        // We need to use a temporary performer because of borrow checker
        // Split the state into parts to avoid borrow conflicts
        for &byte in data {
            self.parser.advance(self, byte);
        }
    }

    /// Write a character at the current cursor position
    fn put_char(&mut self, c: char) {
        let cell = Cell {
            char: c.to_string(),
            fg: self.attrs.fg,
            bg: self.attrs.bg,
            flags: self.attrs.to_flags(),
        };

        if self.cursor.col < self.buffer.cols {
            if let Some(cell_ref) = self.buffer.cell_mut(self.cursor.row, self.cursor.col) {
                *cell_ref = cell;
            }
        }

        self.cursor.col += 1;
        if self.cursor.col >= self.buffer.cols {
            self.cursor.col = 0;
            self.line_feed();
        }
    }

    fn line_feed(&mut self) {
        if self.cursor.row >= self.scroll_bottom {
            self.buffer.scroll_up();
        } else {
            self.cursor.row += 1;
        }
    }

    fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    fn tab(&mut self) {
        let next_tab = (self.cursor.col / 8 + 1) * 8;
        self.cursor.col = next_tab.min(self.buffer.cols - 1);
    }

    /// Convert state to JSON for frontend
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "cursor": {
                "row": self.cursor.row,
                "col": self.cursor.col,
                "visible": self.cursor.visible,
            },
            "grid": self.buffer.to_json(),
        })
    }
}

// ============================================================
// VT Parser Implementation (vte::Perform trait)
// ============================================================

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => { /* BEL - bell */ }
            0x08 => self.backspace(),
            0x09 => self.tab(),
            0x0A | 0x0B | 0x0C => self.line_feed(),
            0x0D => self.carriage_return(),
            0x7F => { /* DEL - ignored */ }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'A' => {
                // Cursor Up
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                self.cursor.row = self.cursor.row.saturating_sub(n);
            }
            'B' => {
                // Cursor Down
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                self.cursor.row = (self.cursor.row + n).min(self.buffer.rows - 1);
            }
            'C' => {
                // Cursor Forward
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                self.cursor.col = (self.cursor.col + n).min(self.buffer.cols - 1);
            }
            'D' => {
                // Cursor Backward
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                self.cursor.col = self.cursor.col.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor Position (CUP)
                let row = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                let col = params.iter().nth(1).and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                self.cursor.row = (row - 1).min(self.buffer.rows - 1);
                self.cursor.col = (col - 1).min(self.buffer.cols - 1);
            }
            'J' => {
                // Erase in Display
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0);
                match n {
                    0 => {
                        // Clear from cursor to end
                        self.buffer.clear_to_eol(self.cursor.row, self.cursor.col);
                        for row in (self.cursor.row + 1)..self.buffer.rows {
                            self.buffer.clear_to_eol(row, 0);
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.buffer.clear();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0);
                match n {
                    0 => self.buffer.clear_to_eol(self.cursor.row, self.cursor.col),
                    2 => self.buffer.clear_to_eol(self.cursor.row, 0),
                    _ => {}
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition (colors/styles)
                self.handle_sgr(params);
            }
            'r' => {
                // Set Scrolling Region
                let top = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as u16;
                let bottom = params.iter().nth(1).and_then(|p| p.first()).copied().unwrap_or(self.buffer.rows) as u16;
                self.scroll_top = top - 1;
                self.scroll_bottom = (bottom - 1).min(self.buffer.rows - 1);
                self.cursor.row = 0;
                self.cursor.col = 0;
            }
            's' => {
                // Save cursor position
                self.saved_cursor = self.cursor.clone();
            }
            'u' => {
                // Restore cursor position
                self.cursor = self.saved_cursor.clone();
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences (e.g., window title)
        // TODO: Handle OSC 0/2 (set window title)
    }
}

impl TerminalState {
    /// Handle SGR (Select Graphic Rendition) escape sequences
    fn handle_sgr(&mut self, params: &vte::Params) {
        let mut iter = params.iter();

        while let Some(param_group) = iter.next() {
            for &param in param_group {
                match param {
                    0 => {
                        // Reset
                        self.attrs = TextAttrs::default();
                    }
                    1 => self.attrs.bold = true,
                    3 => self.attrs.italic = true,
                    4 => self.attrs.underline = true,
                    7 => self.attrs.inverse = true,
                    9 => self.attrs.strikethrough = true,
                    22 => self.attrs.bold = false,
                    23 => self.attrs.italic = false,
                    24 => self.attrs.underline = false,
                    27 => self.attrs.inverse = false,
                    29 => self.attrs.strikethrough = false,
                    30..=37 => {
                        // Standard foreground colors
                        self.attrs.fg = (param - 30) as u8;
                    }
                    38 => {
                        // Extended foreground
                        if let Some(next) = iter.next() {
                            if next.first() == Some(&5) {
                                if let Some(color) = iter.next().and_then(|p| p.first()) {
                                    self.attrs.fg = *color as u8;
                                }
                            }
                        }
                    }
                    39 => {
                        // Default foreground
                        self.attrs.fg = 7;
                    }
                    40..=47 => {
                        // Standard background colors
                        self.attrs.bg = (param - 40) as u8;
                    }
                    48 => {
                        // Extended background
                        if let Some(next) = iter.next() {
                            if next.first() == Some(&5) {
                                if let Some(color) = iter.next().and_then(|p| p.first()) {
                                    self.attrs.bg = *color as u8;
                                }
                            }
                        }
                    }
                    49 => {
                        // Default background
                        self.attrs.bg = 0;
                    }
                    90..=97 => {
                        // Bright foreground colors
                        self.attrs.fg = (param - 90 + 8) as u8;
                    }
                    100..=107 => {
                        // Bright background colors
                        self.attrs.bg = (param - 100 + 8) as u8;
                    }
                    _ => {}
                }
            }
        }
    }
}
