import { create } from 'zustand';
import type { Tab, TerminalConfig, Theme } from '../types/terminal';

// ============================================================
// Default Theme (One Dark inspired, like cmux)
// ============================================================

const defaultTheme: Theme = {
  name: 'One Dark',
  background: '#282c34',
  foreground: '#abb2bf',
  cursor: '#528bff',
  selection: '#3e4451',
  black: '#282c34',
  red: '#e06c75',
  green: '#98c379',
  yellow: '#e5c07b',
  blue: '#61afef',
  magenta: '#c678dd',
  cyan: '#56b6c2',
  white: '#abb2bf',
  brightBlack: '#5c6370',
  brightRed: '#e06c75',
  brightGreen: '#98c379',
  brightYellow: '#e5c07b',
  brightBlue: '#61afef',
  brightMagenta: '#c678dd',
  brightCyan: '#56b6c2',
  brightWhite: '#ffffff',
};

const defaultConfig: TerminalConfig = {
  fontSize: 14,
  fontFamily: 'Cascadia Code, Consolas, monospace',
  theme: defaultTheme,
  cursorStyle: 'block',
  cursorBlink: true,
  scrollback: 10000,
};

// ============================================================
// Store State & Actions
// ============================================================

interface TerminalStore {
  // State
  tabs: Tab[];
  activeTabId: string | null;
  config: TerminalConfig;
  sidebarVisible: boolean;
  browserVisible: boolean;
  browserUrl: string;
  notificationMessage: string | null;

  // Actions
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<Tab>) => void;
  updateTerminalState: (id: string, updates: Partial<Tab['terminalState']>) => void;
  setConfig: (config: Partial<TerminalConfig>) => void;
  toggleSidebar: () => void;
  toggleBrowser: () => void;
  setBrowserUrl: (url: string) => void;
  showNotification: (message: string) => void;
  clearNotification: () => void;
}

// ============================================================
// Create Store
// ============================================================

export const useTerminalStore = create<TerminalStore>((set) => ({
  tabs: [],
  activeTabId: null,
  config: defaultConfig,
  sidebarVisible: true,
  browserVisible: false,
  browserUrl: 'about:blank',
  notificationMessage: null,

  addTab: (tab) =>
    set((state) => ({
      tabs: [...state.tabs, tab],
      activeTabId: tab.id,
    })),

  removeTab: (id) =>
    set((state) => {
      const newTabs = state.tabs.filter((t) => t.id !== id);
      const newActiveId =
        state.activeTabId === id
          ? newTabs.length > 0
            ? newTabs[newTabs.length - 1].id
            : null
          : state.activeTabId;
      return { tabs: newTabs, activeTabId: newActiveId };
    }),

  setActiveTab: (id) => set({ activeTabId: id }),

  updateTab: (id, updates) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === id ? { ...t, ...updates } : t)),
    })),

  updateTerminalState: (id, updates) =>
    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === id
          ? { ...t, terminalState: { ...t.terminalState, ...updates } }
          : t,
      ),
    })),

  setConfig: (config) =>
    set((state) => ({
      config: { ...state.config, ...config },
    })),

  toggleSidebar: () => set((state) => ({ sidebarVisible: !state.sidebarVisible })),

  toggleBrowser: () => set((state) => ({ browserVisible: !state.browserVisible })),

  setBrowserUrl: (url) => set({ browserUrl: url }),

  showNotification: (message) => set({ notificationMessage: message }),

  clearNotification: () => set({ notificationMessage: null }),
}));
