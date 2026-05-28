import { useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTerminalStore } from '../stores/terminalStore';
import type { Cell, Position, TerminalSize } from '../types/terminal';

// ============================================================
// Terminal Hook — Manages a single terminal instance
// ============================================================

interface TerminalGrid {
  lines: Cell[][];
  cursor: Position;
  cursorVisible: boolean;
  scrollTop: number;
  scrollBottom: number;
}

export function useTerminal(tabId: string) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<TerminalGrid | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const { config, updateTerminalState } = useTerminalStore();

  // ---- Canvas Rendering ----

  const renderGrid = useCallback(() => {
    const canvas = canvasRef.current;
    const grid = gridRef.current;
    if (!canvas || !grid) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const { theme } = config;
    const cellWidth = config.fontSize * 0.6;
    const cellHeight = config.fontSize * 1.4;

    // Clear canvas
    ctx.fillStyle = theme.background;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Render cells
    for (let row = 0; row < grid.lines.length; row++) {
      const line = grid.lines[row];
      if (!line) continue;

      for (let col = 0; col < line.length; col++) {
        const cell = line[col];
        if (!cell || cell.char === ' ') continue;

        const x = col * cellWidth;
        const y = row * cellHeight;

        // Background
        if (cell.bg !== 0) {
          ctx.fillStyle = ansiToColor(cell.bg, theme);
          ctx.fillRect(x, y, cellWidth, cellHeight);
        }

        // Foreground text
        ctx.fillStyle = ansiToColor(cell.fg, theme);
        ctx.font = `${cell.flags & 1 ? 'bold ' : ''}${config.fontSize}px ${config.fontFamily}`;
        ctx.textBaseline = 'top';
        ctx.fillText(cell.char, x, y);
      }
    }

    // Render cursor
    if (grid.cursorVisible) {
      const cx = grid.cursor.col * cellWidth;
      const cy = grid.cursor.row * cellHeight;

      ctx.fillStyle = theme.cursor;
      if (config.cursorStyle === 'block') {
        ctx.globalAlpha = 0.7;
        ctx.fillRect(cx, cy, cellWidth, cellHeight);
        ctx.globalAlpha = 1;
      } else if (config.cursorStyle === 'underline') {
        ctx.fillRect(cx, cy + cellHeight - 2, cellWidth, 2);
      } else {
        ctx.fillRect(cx, cy, 2, cellHeight);
      }
    }
  }, [config]);

  // ---- Event Listeners ----

  useEffect(() => {
    // Listen for terminal output from Rust backend
    const setupListener = async () => {
      const unlisten = await listen<{ id: string; grid: TerminalGrid }>(
        'terminal-update',
        (event) => {
          if (event.payload.id === tabId) {
            gridRef.current = event.payload.grid;
            renderGrid();
          }
        },
      );
      unlistenRef.current = unlisten;
    };

    setupListener();

    return () => {
      unlistenRef.current?.();
    };
  }, [tabId, renderGrid]);

  // ---- Resize Handler ----

  useEffect(() => {
    const handleResize = () => {
      const canvas = canvasRef.current;
      const container = containerRef.current;
      if (!canvas || !container) return;

      const rect = container.getBoundingClientRect();
      canvas.width = rect.width;
      canvas.height = rect.height;

      const cellWidth = config.fontSize * 0.6;
      const cellHeight = config.fontSize * 1.4;
      const cols = Math.floor(rect.width / cellWidth);
      const rows = Math.floor(rect.height / cellHeight);

      invoke('resize_terminal', {
        id: tabId,
        cols,
        rows,
      }).catch(console.error);

      updateTerminalState(tabId, { size: { cols, rows } });
      renderGrid();
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [tabId, config.fontSize, renderGrid, updateTerminalState]);

  // ---- Input Handling ----

  const writeToTerminal = useCallback(
    (data: string) => {
      invoke('write_to_terminal', { id: tabId, data }).catch(console.error);
    },
    [tabId],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      e.preventDefault();

      let data = '';

      if (e.key === 'Enter') data = '\r';
      else if (e.key === 'Backspace') data = '\x7f';
      else if (e.key === 'Tab') data = '\t';
      else if (e.key === 'Escape') data = '\x1b';
      else if (e.key === 'ArrowUp') data = '\x1b[A';
      else if (e.key === 'ArrowDown') data = '\x1b[B';
      else if (e.key === 'ArrowRight') data = '\x1b[C';
      else if (e.key === 'ArrowLeft') data = '\x1b[D';
      else if (e.key === 'Home') data = '\x1b[H';
      else if (e.key === 'End') data = '\x1b[F';
      else if (e.key === 'PageUp') data = '\x1b[5~';
      else if (e.key === 'PageDown') data = '\x1b[6~';
      else if (e.key === 'Delete') data = '\x1b[3~';
      else if (e.key === 'Insert') data = '\x1b[2~';
      else if (e.ctrlKey && e.key.length === 1) {
        data = String.fromCharCode(e.key.toLowerCase().charCodeAt(0) - 96);
      } else if (e.key.length === 1) {
        data = e.key;
      }

      if (data) writeToTerminal(data);
    },
    [writeToTerminal],
  );

  // ---- Clipboard ----

  const handlePaste = useCallback(
    (e: React.ClipboardEvent) => {
      const text = e.clipboardData.getData('text');
      if (text) writeToTerminal(text);
    },
    [writeToTerminal],
  );

  return {
    canvasRef,
    containerRef,
    handleKeyDown,
    handlePaste,
    renderGrid,
  };
}

// ============================================================
// ANSI Color Helper
// ============================================================

function ansiToColor(code: number, theme: import('../types/terminal').Theme): string {
  const colors = [
    theme.black, theme.red, theme.green, theme.yellow,
    theme.blue, theme.magenta, theme.cyan, theme.white,
    theme.brightBlack, theme.brightRed, theme.brightGreen, theme.brightYellow,
    theme.brightBlue, theme.brightMagenta, theme.brightCyan, theme.brightWhite,
  ];
  if (code >= 0 && code < colors.length) return colors[code];
  if (code === 0) return theme.foreground;
  return theme.foreground;
}
