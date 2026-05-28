// ============================================================
// Terminal Type Definitions
// ============================================================

export interface Position {
  row: number;
  col: number;
}

export interface Cell {
  char: string;
  fg: number;      // Foreground color (ANSI index or RGB)
  bg: number;      // Background color
  flags: number;   // Bold, italic, underline, etc.
}

export const CellFlags = {
  BOLD: 1,
  ITALIC: 2,
  UNDERLINE: 4,
  BLINK: 8,
  INVERSE: 16,
  STRIKETHROUGH: 32,
} as const;

export interface TerminalSize {
  cols: number;
  rows: number;
}

export interface TerminalState {
  id: string;
  title: string;
  cwd: string;
  gitBranch?: string;
  gitStatus?: 'clean' | 'dirty' | 'ahead' | 'behind';
  isRunning: boolean;
  lastActivity: number;
  size: TerminalSize;
}

export interface TerminalOutput {
  id: string;
  data: string;  // Raw VT output
  timestamp: number;
}

export interface Tab {
  id: string;
  title: string;
  icon?: string;
  isActive: boolean;
  terminalState: TerminalState;
}

export interface SidebarItem {
  id: string;
  label: string;
  icon: string;
  type: 'terminal' | 'browser' | 'settings';
  badge?: string | number;
}

export interface Theme {
  name: string;
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}

export interface TerminalConfig {
  fontSize: number;
  fontFamily: string;
  theme: Theme;
  cursorStyle: 'block' | 'underline' | 'bar';
  cursorBlink: boolean;
  scrollback: number;
}
