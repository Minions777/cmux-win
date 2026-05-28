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

// ============================================================
// Git Types
// ============================================================

export interface GitStatus {
  branch: string;
  upstream: string | null;
  state: RepoState;
  ahead: number;
  behind: number;
  staged: number;
  modified: number;
  untracked: number;
  conflicted: number;
  lastCommit: CommitInfo | null;
  remoteUrl: string | null;
  tags: string[];
}

export type RepoState = 'clean' | 'dirty' | 'ahead' | 'behind' | 'aheadAndBehind' | 'conflicted' | 'unknown';

export interface CommitInfo {
  hash: string;
  shortHash: string;
  message: string;
  author: string;
  timestamp: number;
}

export interface BranchInfo {
  name: string;
  isCurrent: boolean;
  upstream: string | null;
}

// ============================================================
// SSH Types
// ============================================================

export interface SshConfig {
  host: string;
  port: number;
  username: string;
  authMethod: SshAuth;
}

export type SshAuth =
  | { Password: string }
  | { Key: { path: string; passphrase: string | null } }
  | 'Agent';

export interface SshSession {
  id: string;
  config: SshConfig;
  isConnected: boolean;
}

// ============================================================
// Notification Types
// ============================================================

export interface Notification {
  title: string;
  body: string;
  icon: string | null;
  urgency: 'low' | 'normal' | 'critical';
  terminalId: string;
  timestamp: number;
}

export interface NotificationConfig {
  enabled: boolean;
  soundEnabled: boolean;
  minDurationSecs: number;
  watchPatterns: string[];
}
