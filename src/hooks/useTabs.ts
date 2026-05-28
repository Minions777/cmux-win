import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTerminalStore } from '../stores/terminalStore';
import type { Tab, TerminalState } from '../types/terminal';

export function useTabs() {
  const { tabs, activeTabId, addTab, removeTab, setActiveTab } = useTerminalStore();

  const createTab = useCallback(async () => {
    try {
      const result = await invoke<{ id: string; title: string; cwd: string }>('create_terminal', {
        shell: null,
        cwd: null,
      });

      const terminalState: TerminalState = {
        id: result.id,
        title: result.title,
        cwd: result.cwd,
        isRunning: false,
        lastActivity: Date.now(),
        size: { cols: 80, rows: 24 },
      };

      const newTab: Tab = {
        id: result.id,
        title: result.title,
        isActive: true,
        terminalState,
      };

      addTab(newTab);
      return result.id;
    } catch (err) {
      console.error('Failed to create terminal:', err);
      return null;
    }
  }, [addTab]);

  const closeTab = useCallback(
    async (id: string) => {
      try {
        await invoke('close_terminal', { id });
      } catch (err) {
        console.error('Failed to close terminal:', err);
      }
      removeTab(id);
    },
    [removeTab],
  );

  const switchTab = useCallback(
    (id: string) => {
      setActiveTab(id);
    },
    [setActiveTab],
  );

  const getActiveTab = useCallback(() => {
    return tabs.find((t) => t.id === activeTabId) || null;
  }, [tabs, activeTabId]);

  return {
    tabs,
    activeTabId,
    createTab,
    closeTab,
    switchTab,
    getActiveTab,
  };
}
