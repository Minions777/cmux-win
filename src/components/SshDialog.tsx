import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTerminalStore } from '../stores/terminalStore';
import type { SshConfig } from '../types/terminal';

interface SshDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SshDialog({ isOpen, onClose }: SshDialogProps) {
  const [host, setHost] = useState('');
  const [port, setPort] = useState('22');
  const [username, setUsername] = useState('');
  const [authMethod, setAuthMethod] = useState<'password' | 'key'>('password');
  const [password, setPassword] = useState('');
  const [keyPath, setKeyPath] = useState('');
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { addSshSession } = useTerminalStore();

  if (!isOpen) return null;

  const handleConnect = async () => {
    setIsConnecting(true);
    setError(null);

    try {
      const id = `ssh-${host}-${Date.now()}`;
      const config: SshConfig = {
        host,
        port: parseInt(port, 10),
        username,
        authMethod:
          authMethod === 'password'
            ? { Password: password }
            : { Key: { path: keyPath, passphrase: null } },
      };

      await invoke('ssh_connect', { id, config });

      addSshSession({
        id,
        config,
        isConnected: true,
      });

      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="ssh-dialog-overlay" onClick={onClose}>
      <div className="ssh-dialog" onClick={(e) => e.stopPropagation()}>
        <h2>Connect to SSH Server</h2>

        <div className="ssh-form">
          <label>
            Host
            <input
              type="text"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              placeholder="192.168.1.100 or hostname"
            />
          </label>

          <label>
            Port
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(e.target.value)}
              placeholder="22"
            />
          </label>

          <label>
            Username
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="root"
            />
          </label>

          <label>
            Authentication
            <select
              value={authMethod}
              onChange={(e) => setAuthMethod(e.target.value as 'password' | 'key')}
            >
              <option value="password">Password</option>
              <option value="key">SSH Key</option>
            </select>
          </label>

          {authMethod === 'password' ? (
            <label>
              Password
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </label>
          ) : (
            <label>
              Key Path
              <input
                type="text"
                value={keyPath}
                onChange={(e) => setKeyPath(e.target.value)}
                placeholder="~/.ssh/id_rsa"
              />
            </label>
          )}

          {error && <div className="ssh-error">{error}</div>}

          <div className="ssh-actions">
            <button onClick={onClose} className="btn-secondary">
              Cancel
            </button>
            <button
              onClick={handleConnect}
              className="btn-primary"
              disabled={isConnecting || !host || !username}
            >
              {isConnecting ? 'Connecting...' : 'Connect'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
