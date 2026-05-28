import { useState, useRef } from 'react';
import { useTerminalStore } from '../stores/terminalStore';

export default function BrowserPane() {
  const { browserUrl, setBrowserUrl, toggleBrowser } = useTerminalStore();
  const [inputUrl, setInputUrl] = useState(browserUrl);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  const handleNavigate = (e: React.FormEvent) => {
    e.preventDefault();
    let url = inputUrl;
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'https://' + url;
    }
    setBrowserUrl(url);
    setInputUrl(url);
  };

  return (
    <div className="browser-pane">
      {/* Browser Header */}
      <div className="browser-header">
        <div className="browser-nav">
          <button className="browser-btn" onClick={() => iframeRef.current?.contentWindow?.history.back()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="19" y1="12" x2="5" y2="12" />
              <polyline points="12 19 5 12 12 5" />
            </svg>
          </button>
          <button className="browser-btn" onClick={() => iframeRef.current?.contentWindow?.history.forward()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </button>
          <button className="browser-btn" onClick={() => setBrowserUrl(browserUrl)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="23 4 23 10 17 10" />
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
          </button>
        </div>
        <form className="browser-url-form" onSubmit={handleNavigate}>
          <input
            type="text"
            className="browser-url-input"
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            placeholder="Enter URL..."
          />
        </form>
        <button className="browser-btn browser-close" onClick={toggleBrowser}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      {/* Browser Content */}
      <div className="browser-content">
        <iframe
          ref={iframeRef}
          src={browserUrl}
          className="browser-iframe"
          title="Built-in Browser"
          sandbox="allow-same-origin allow-scripts allow-forms"
        />
      </div>
    </div>
  );
}
