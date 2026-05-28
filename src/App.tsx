import { useEffect } from 'react';
import { useTabs } from './hooks/useTabs';
import { useTerminalStore } from './stores/terminalStore';
import Sidebar from './components/Sidebar';
import Terminal from './components/Terminal';
import TabBar from './components/TabBar';
import NotificationRing from './components/NotificationRing';
import BrowserPane from './components/BrowserPane';
import './styles/sidebar.css';
import './styles/terminal.css';
import './styles/notification.css';

function App() {
  const { tabs, activeTabId, createTab, closeTab, switchTab } = useTabs();
  const { sidebarVisible, browserVisible, notificationMessage, clearNotification } = useTerminalStore();

  // Create initial tab on mount
  useEffect(() => {
    if (tabs.length === 0) {
      createTab();
    }
  }, []);

  const activeTab = tabs.find((t) => t.id === activeTabId);

  return (
    <div className="app-container">
      {/* Notification Ring */}
      {notificationMessage && (
        <NotificationRing message={notificationMessage} onDismiss={clearNotification} />
      )}

      {/* Sidebar */}
      {sidebarVisible && (
        <Sidebar
          tabs={tabs}
          activeTabId={activeTabId}
          onTabClick={switchTab}
          onNewTab={createTab}
          onCloseTab={closeTab}
        />
      )}

      {/* Main Content Area */}
      <div className="main-content">
        {/* Tab Bar */}
        <TabBar
          tabs={tabs}
          activeTabId={activeTabId}
          onTabClick={switchTab}
          onNewTab={createTab}
          onCloseTab={closeTab}
        />

        {/* Terminal + Browser Split */}
        <div className="content-split">
          {/* Terminal */}
          <div className="terminal-area" style={{ flex: browserVisible ? 1 : 1 }}>
            {activeTab ? (
              <Terminal key={activeTab.id} tab={activeTab} />
            ) : (
              <div className="empty-state">
                <div className="empty-icon">⚡</div>
                <div className="empty-title">cmux-win</div>
                <div className="empty-subtitle">Press Ctrl+T to open a new terminal</div>
              </div>
            )}
          </div>

          {/* Browser Pane */}
          {browserVisible && (
            <div className="browser-area">
              <BrowserPane />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
