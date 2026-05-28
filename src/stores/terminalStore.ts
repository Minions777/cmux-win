import { create } from 'zustand';
import type {
  Tab,
  TerminalConfig,
  Theme,
  GitStatus,
  Notification,
  NotificationConfig,
  SshSession,
} from '../types/terminal';

// Default Catppuccin Mocha theme
const defaultTheme: Theme = {
  name: 'Catppuccin Mocha',
  background: '#1e1e2e',
  foreground: '#cdd6f4',
  cursor: '#f5e0dc',
  selection: '#45475a',
  black: '#45475a',
  red: '#f38ba8',
  green: '#a6e3a1',
  yellow: '#f9e2af',
  blue: '#89b4fa',
  magenta: '#f5c2e7',
  cyan: '#94e2d5',
  white: '#bac2de',
  brightBlack: '#585b70',
  brightRed: '#f38ba8',
  brightGreen: '#a6e3a1',
  brightYellow: '#f9e2af',
  brightBlue: '#89b4fa',
  brightMagenta: '#f5c2e7',
  brightCyan: '#94e2d5',
  brightWhite: '#a6adc8',
};

interface TerminalStore {
  // Tabs
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<Tab>) => void;

  // Config
  config: TerminalConfig;
  updateConfig: (updates: Partial<TerminalConfig>) => void;

  // Git status per terminal
  gitStatuses: Record<string, GitStatus>;
  updateGitStatus: (terminalId: string, status: GitStatus) => void;

  // Notifications
  notifications: Notification[];
  notificationConfig: NotificationConfig;
  addNotification: (notification: Notification) => void;
  clearNotifications: () => void;
  updateNotificationConfig: (config: Partial<NotificationConfig>) => void;

  // SSH Sessions
  sshSessions: Record<string, SshSession>;
  addSshSession: (session: SshSession) => void;
  removeSshSession: (id: string) => void;
  updateSshSession: (id: string, updates: Partial<SshSession>) => void;

  // Sidebar
  isSidebarOpen: boolean;
  toggleSidebar: () => void;

  // Notification ring
  activeNotificationTerminalId: string | null;
  setActiveNotification: (terminalId: string | null) => void;
}

export const useTerminalStore = create<TerminalStore>((set) => ({
  // Tabs
  tabs: [],
  activeTabId: null,
  addTab: (tab) =>
    set((state) => ({
      tabs: [...state.tabs, tab],
      activeTabId: tab.id,
    })),
  removeTab: (id) =>
    set((state) => ({
      tabs: state.tabs.filter((t) => t.id !== id),
      activeTabId:
        state.activeTabId === id
          ? state.tabs.find((t) => t.id !== id)?.id || null
          : state.activeTabId,
    })),
  setActiveTab: (id) => set({ activeTabId: id }),
  updateTab: (id, updates) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === id ? { ...t, ...updates } : t)),
    })),

  // Config
  config: {
    fontSize: 14,
    fontFamily: 'Cascadia Code, Consolas, monospace',
    theme: defaultTheme,
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 10000,
  },
  updateConfig: (updates) =>
    set((state) => ({
      config: { ...state.config, ...updates },
    })),

  // Git
  gitStatuses: {},
  updateGitStatus: (terminalId, status) =>
    set((state) => ({
      gitStatuses: { ...state.gitStatuses, [terminalId]: status },
    })),

  // Notifications
  notifications: [],
  notificationConfig: {
    enabled: true,
    soundEnabled: true,
    minDurationSecs: 5,
    watchPatterns: ['\\$\\s*$', '%\\s*$', '>\\s*$', 'PS [^>]*>\\s*$'],
  },
  addNotification: (notification) =>
    set((state) => ({
      notifications: [...state.notifications, notification],
    })),
  clearNotifications: () => set({ notifications: [] }),
  updateNotificationConfig: (config) =>
    set((state) => ({
      notificationConfig: { ...state.notificationConfig, ...config },
    })),

  // SSH
  sshSessions: {},
  addSshSession: (session) =>
    set((state) => ({
      sshSessions: { ...state.sshSessions, [session.id]: session },
    })),
  removeSshSession: (id) =>
    set((state) => {
      const { [id]: _, ...rest } = state.sshSessions;
      return { sshSessions: rest };
    }),
  updateSshSession: (id, updates) =>
    set((state) => ({
      sshSessions: {
        ...state.sshSessions,
        [id]: { ...state.sshSessions[id], ...updates },
      },
    })),

  // Sidebar
  isSidebarOpen: true,
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),

  // Notification ring
  activeNotificationTerminalId: null,
  setActiveNotification: (terminalId) =>
    set({ activeNotificationTerminalId: terminalId }),
}));
