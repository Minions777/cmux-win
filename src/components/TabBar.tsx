import type { Tab } from '../types/terminal';

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onTabClick: (id: string) => void;
  onNewTab: () => void;
  onCloseTab: (id: string) => void;
}

export default function TabBar({ tabs, activeTabId, onTabClick, onNewTab, onCloseTab }: TabBarProps) {
  return (
    <div className="tab-bar">
      <div className="tab-list">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`tab-item ${tab.id === activeTabId ? 'active' : ''}`}
            onClick={() => onTabClick(tab.id)}
          >
            <span className="tab-icon">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="4 17 10 11 4 5" />
                <line x1="12" y1="19" x2="20" y2="19" />
              </svg>
            </span>
            <span className="tab-title">{tab.title}</span>
            {tab.terminalState.isRunning && (
              <span className="tab-running" title="Process running">
                <span className="running-dot"></span>
              </span>
            )}
            <button
              className="tab-close"
              onClick={(e) => {
                e.stopPropagation();
                onCloseTab(tab.id);
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
      <button className="tab-new" onClick={onNewTab} title="New Terminal (Ctrl+T)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>
  );
}
