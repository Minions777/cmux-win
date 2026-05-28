import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTerminalStore } from '../stores/terminalStore';
import type { GitStatus as GitStatusType } from '../types/terminal';

interface GitStatusProps {
  terminalId: string;
  cwd: string;
}

export function GitStatus({ terminalId, cwd }: GitStatusProps) {
  const { gitStatuses, updateGitStatus } = useTerminalStore();
  const status = gitStatuses[terminalId];

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const result = await invoke<GitStatusType | null>('git_get_status', {
          path: cwd,
        });
        if (result) {
          updateGitStatus(terminalId, result);
        }
      } catch (err) {
        // Not a git repo or error
      }
    };

    fetchStatus();

    // Listen for git status updates
    const unlisten = listen<{ terminalId: string; status: GitStatusType }>(
      'git-status-changed',
      (event) => {
        if (event.payload.terminalId === terminalId) {
          updateGitStatus(terminalId, event.payload.status);
        }
      },
    );

    // Poll for changes every 5 seconds
    const interval = setInterval(fetchStatus, 5000);

    return () => {
      unlisten.then((fn) => fn());
      clearInterval(interval);
    };
  }, [terminalId, cwd, updateGitStatus]);

  if (!status) return null;

  const stateColor = {
    clean: 'var(--color-green)',
    dirty: 'var(--color-yellow)',
    ahead: 'var(--color-blue)',
    behind: 'var(--color-red)',
    aheadAndBehind: 'var(--color-magenta)',
    conflicted: 'var(--color-red)',
    unknown: 'var(--color-subtext)',
  }[status.state];

  return (
    <div className="git-status">
      <span className="git-branch" style={{ color: stateColor }}>
        {'\ue0a0'} {status.branch}
      </span>

      {status.ahead > 0 && (
        <span className="git-ahead" title="Commits ahead">
          {'\u2191'}{status.ahead}
        </span>
      )}

      {status.behind > 0 && (
        <span className="git-behind" title="Commits behind">
          {'\u2193'}{status.behind}
        </span>
      )}

      {status.staged > 0 && (
        <span className="git-staged" title="Staged files">
          {'\u2714'}{status.staged}
        </span>
      )}

      {status.modified > 0 && (
        <span className="git-modified" title="Modified files">
          {'\u270e'}{status.modified}
        </span>
      )}

      {status.untracked > 0 && (
        <span className="git-untracked" title="Untracked files">
          ?{status.untracked}
        </span>
      )}

      {status.conflicted > 0 && (
        <span className="git-conflicted" title="Conflicted files">
          {'\u2716'}{status.conflicted}
        </span>
      )}

      {status.lastCommit && (
        <span className="git-last-commit" title={status.lastCommit.message}>
          {status.lastCommit.shortHash}
        </span>
      )}

      {status.tags.length > 0 && (
        <span className="git-tags" title={status.tags.join(', ')}>
          {'\uf02b'} {status.tags[0]}
        </span>
      )}
    </div>
  );
}
