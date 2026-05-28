import { useEffect, useRef } from 'react';
import { useTerminal } from '../hooks/useTerminal';
import type { Tab } from '../types/terminal';

interface TerminalProps {
  tab: Tab;
}

export default function Terminal({ tab }: TerminalProps) {
  const { canvasRef, containerRef, handleKeyDown, handlePaste } = useTerminal(tab.id);
  const terminalRef = useRef<HTMLDivElement>(null);

  // Auto-focus the terminal
  useEffect(() => {
    terminalRef.current?.focus();
  }, []);

  return (
    <div
      ref={terminalRef}
      className="terminal-container"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
      onClick={() => terminalRef.current?.focus()}
    >
      {/* Terminal Header */}
      <div className="terminal-header">
        <div className="terminal-title">
          <span className="terminal-icon">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
          </span>
          <span>{tab.title}</span>
          {tab.terminalState.gitBranch && (
            <span className="terminal-git">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="18" cy="18" r="3" />
                <circle cx="6" cy="6" r="3" />
                <path d="M6 21V9a9 9 0 0 0 9 9" />
              </svg>
              {tab.terminalState.gitBranch}
            </span>
          )}
        </div>
        <div className="terminal-actions">
          <span className="terminal-size">
            {tab.terminalState.size.cols}×{tab.terminalState.size.rows}
          </span>
        </div>
      </div>

      {/* Terminal Canvas */}
      <div ref={containerRef} className="terminal-canvas-container">
        <canvas ref={canvasRef} className="terminal-canvas" />
      </div>
    </div>
  );
}
