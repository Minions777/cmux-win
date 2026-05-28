import { useEffect, useState } from 'react';

interface NotificationRingProps {
  message: string;
  onDismiss: () => void;
}

export default function NotificationRing({ message, onDismiss }: NotificationRingProps) {
  const [visible, setVisible] = useState(true);
  const [animating, setAnimating] = useState(false);

  useEffect(() => {
    // Start entrance animation
    setAnimating(true);

    // Auto-dismiss after 5 seconds
    const timer = setTimeout(() => {
      handleDismiss();
    }, 5000);

    return () => clearTimeout(timer);
  }, []);

  const handleDismiss = () => {
    setVisible(false);
    setTimeout(onDismiss, 300); // Wait for exit animation
  };

  if (!visible) return null;

  return (
    <div className={`notification-ring ${animating ? 'animate' : ''}`} onClick={handleDismiss}>
      <div className="notification-glow" />
      <div className="notification-content">
        <div className="notification-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
            <path d="M13.73 21a2 2 0 0 1-3.46 0" />
          </svg>
        </div>
        <div className="notification-text">
          <div className="notification-title">Task Complete</div>
          <div className="notification-message">{message}</div>
        </div>
        <button className="notification-dismiss" onClick={handleDismiss}>×</button>
      </div>
    </div>
  );
}
