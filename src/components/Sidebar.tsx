import type { Tab } from '../types/terminal';

interface SidebarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onTabClick: (id: string) => void;
  onNewTab: () => void;
  onCloseTab: (id: string) => void;
}

export default function Sidebar({ tabs, activeTabId, onTabClick, onNewTab, onCloseTab }: SidebarProps) {
  return (
    <div className="sidebar">
      {/* Logo */}
      <div className="sidebar-logo">
        <div className="logo-icon">⚡</div>
        <span className="logo-text">cmux</span>
      </div>

      {/* Navigation Icons */}
      <div className="sidebar-nav">
        <button className="nav-btn active" title="Terminal" onClick={onNewTab}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
        </button>
        <button className="nav-btn" title="Explorer">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </button>
        <button className="nav-btn" title="Search">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
        </button>
        <button className="nav-btn" title="Git">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="18" cy="18" r="3" />
            <circle cx="6" cy="6" r="3" />
            <path d="M6 21V9a9 9 0 0 0 9 9" />
          </svg>
        </button>
      </div>

      {/* Terminal Tabs List */}
      <div className="sidebar-section">
        <div className="section-header">
          <span>TERMINALS</span>
          <button className="section-btn" onClick={onNewTab} title="New Terminal">
            +
          </button>
        </div>
        <div className="terminal-list">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              className={`terminal-item ${tab.id === activeTabId ? 'active' : ''}`}
              onClick={() => onTabClick(tab.id)}
            >
              <div className="terminal-item-icon">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polyline points="4 17 10 11 4 5" />
                  <line x1="12" y1="19" x2="20" y2="19" />
                </svg>
              </div>
              <div className="terminal-item-info">
                <div className="terminal-item-title">{tab.title}</div>
                <div className="terminal-item-meta">
                  {tab.terminalState.gitBranch && (
                    <span className="git-branch">
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <circle cx="18" cy="18" r="3" />
                        <circle cx="6" cy="6" r="3" />
                        <path d="M6 21V9a9 9 0 0 0 9 9" />
                      </svg>
                      {tab.terminalState.gitBranch}
                    </span>
                  )}
                  {tab.terminalState.cwd && (
                    <span className="cwd">{tab.terminalState.cwd.split('\\').pop()}</span>
                  )}
                </div>
              </div>
              <button
                className="terminal-item-close"
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
      </div>

      {/* Bottom Actions */}
      <div className="sidebar-bottom">
        <button className="nav-btn" title="Settings">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
    </div>
  );
}
