import { useEffect, useRef, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { Cell, TerminalState } from '../types/terminal';

interface UseWebGLTerminalOptions {
  terminalId: string;
  cols: number;
  rows: number;
  fontSize?: number;
  fontFamily?: string;
}

interface UseWebGLTerminalReturn {
  canvasRef: React.RefObject<HTMLCanvasElement>;
  isConnected: boolean;
  cursorPos: { row: number; col: number };
  write: (data: string) => Promise<void>;
  resize: (cols: number, rows: number) => Promise<void>;
}

export function useWebGLTerminal({
  terminalId,
  cols,
  rows,
  fontSize = 14,
  fontFamily = 'Cascadia Code, Consolas, monospace',
}: UseWebGLTerminalOptions): UseWebGLTerminalReturn {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const glRef = useRef<WebGL2RenderingContext | null>(null);
  const programRef = useRef<WebGLProgram | null>(null);
  const vaoRef = useRef<WebGLVertexArrayObject | null>(null);
  const cellsRef = useRef<Cell[]>(
    Array.from({ length: cols * rows }, () => ({
      char: ' ',
      fg: 0xcccccc,
      bg: 0x1e1e2e,
      flags: 0,
    })),
  );
  const [isConnected, setIsConnected] = useState(false);
  const [cursorPos, setCursorPos] = useState({ row: 0, col: 0 });
  const animationFrameRef = useRef<number>(0);

  // Initialize WebGL
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const gl = canvas.getContext('webgl2', {
      antialias: false,
      alpha: false,
    });

    if (!gl) {
      console.error('WebGL2 not supported');
      return;
    }

    glRef.current = gl;

    // Create shader program
    const vertShader = createShader(gl, gl.VERTEX_SHADER, VERTEX_SHADER);
    const fragShader = createShader(gl, gl.FRAGMENT_SHADER, FRAGMENT_SHADER);

    if (!vertShader || !fragShader) return;

    const program = createProgram(gl, vertShader, fragShader);
    if (!program) return;

    programRef.current = program;

    // Clean up shaders
    gl.deleteShader(vertShader);
    gl.deleteShader(fragShader);

    // Set up VAO
    const vao = setupVAO(gl, program);
    vaoRef.current = vao;

    // Set initial viewport
    const dpr = window.devicePixelRatio || 1;
    canvas.width = canvas.clientWidth * dpr;
    canvas.height = canvas.clientHeight * dpr;
    gl.viewport(0, 0, canvas.width, canvas.height);

    // Start render loop
    const render = () => {
      renderFrame(gl, program, vao, cellsRef.current, cols, rows, canvas.width, canvas.height);
      animationFrameRef.current = requestAnimationFrame(render);
    };
    animationFrameRef.current = requestAnimationFrame(render);

    return () => {
      cancelAnimationFrame(animationFrameRef.current);
      gl.deleteProgram(program);
      gl.deleteVertexArray(vao);
    };
  }, [cols, rows]);

  // Listen for terminal output
  useEffect(() => {
    const unlisten = listen<{ id: string; data: string }>(
      'terminal-output',
      (event) => {
        if (event.payload.id === terminalId) {
          // Parse VT output and update cells
          parseAndUpdateCells(event.payload.data, cellsRef.current, cols, rows, setCursorPos);
        }
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [terminalId, cols, rows]);

  // Write to terminal
  const write = useCallback(
    async (data: string) => {
      try {
        await invoke('write_to_terminal', {
          id: terminalId,
          data: Array.from(new TextEncoder().encode(data)),
        });
      } catch (err) {
        console.error('Failed to write:', err);
      }
    },
    [terminalId],
  );

  // Resize terminal
  const resize = useCallback(
    async (newCols: number, newRows: number) => {
      try {
        await invoke('resize_terminal', {
          id: terminalId,
          cols: newCols,
          rows: newRows,
        });
      } catch (err) {
        console.error('Failed to resize:', err);
      }
    },
    [terminalId],
  );

  return {
    canvasRef,
    isConnected,
    cursorPos,
    write,
    resize,
  };
}

// Shader sources
const VERTEX_SHADER = `#version 300 es
precision highp float;

in vec2 a_position;
in vec4 a_color;

uniform vec2 u_resolution;

out vec4 v_color;

void main() {
    vec2 clipSpace = (a_position / u_resolution) * 2.0 - 1.0;
    gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
    v_color = a_color;
}
`;

const FRAGMENT_SHADER = `#version 300 es
precision highp float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    fragColor = v_color;
}
`;

// Helper functions
function createShader(gl: WebGL2RenderingContext, type: number, source: string): WebGLShader | null {
  const shader = gl.createShader(type);
  if (!shader) return null;

  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error('Shader compile error:', gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

function createProgram(
  gl: WebGL2RenderingContext,
  vertShader: WebGLShader,
  fragShader: WebGLShader,
): WebGLProgram | null {
  const program = gl.createProgram();
  if (!program) return null;

  gl.attachShader(program, vertShader);
  gl.attachShader(program, fragShader);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error('Program link error:', gl.getProgramInfoLog(program));
    gl.deleteProgram(program);
    return null;
  }

  return program;
}

function setupVAO(gl: WebGL2RenderingContext, program: WebGLProgram): WebGLVertexArrayObject {
  const vao = gl.createVertexArray()!;
  gl.bindVertexArray(vao);

  // Create dynamic buffer for vertices + colors
  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

  // Position attribute (vec2)
  const posLoc = gl.getAttribLocation(program, 'a_position');
  gl.enableVertexAttribArray(posLoc);
  gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 24, 0);

  // Color attribute (vec4)
  const colorLoc = gl.getAttribLocation(program, 'a_color');
  gl.enableVertexAttribArray(colorLoc);
  gl.vertexAttribPointer(colorLoc, 4, gl.FLOAT, false, 24, 8);

  gl.bindVertexArray(null);

  return vao;
}

function renderFrame(
  gl: WebGL2RenderingContext,
  program: WebGLProgram,
  vao: WebGLVertexArrayObject,
  cells: Cell[],
  cols: number,
  rows: number,
  width: number,
  height: number,
) {
  gl.clearColor(0.12, 0.12, 0.15, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);

  gl.useProgram(program);
  gl.bindVertexArray(vao);

  // Update resolution uniform
  const resLoc = gl.getUniformLocation(program, 'u_resolution');
  gl.uniform2f(resLoc, width, height);

  // Build vertex data for all cells
  const cellWidth = width / cols;
  const cellHeight = height / rows;

  const vertices: number[] = [];

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const cell = cells[row * cols + col];
      const x = col * cellWidth;
      const y = row * cellHeight;

      // Convert colors from int to float
      const r = ((cell.bg >> 16) & 0xff) / 255;
      const g = ((cell.bg >> 8) & 0xff) / 255;
      const b = (cell.bg & 0xff) / 255;

      // Two triangles for quad
      // Triangle 1
      vertices.push(x, y, r, g, b, 1.0);
      vertices.push(x + cellWidth, y, r, g, b, 1.0);
      vertices.push(x, y + cellHeight, r, g, b, 1.0);
      // Triangle 2
      vertices.push(x + cellWidth, y, r, g, b, 1.0);
      vertices.push(x + cellWidth, y + cellHeight, r, g, b, 1.0);
      vertices.push(x, y + cellHeight, r, g, b, 1.0);
    }
  }

  // Upload vertex data
  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.DYNAMIC_DRAW);

  // Re-bind attributes to new buffer
  const posLoc = gl.getAttribLocation(program, 'a_position');
  gl.enableVertexAttribArray(posLoc);
  gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 24, 0);

  const colorLoc = gl.getAttribLocation(program, 'a_color');
  gl.enableVertexAttribArray(colorLoc);
  gl.vertexAttribPointer(colorLoc, 4, gl.FLOAT, false, 24, 8);

  // Draw
  gl.drawArrays(gl.TRIANGLES, 0, vertices.length / 6);

  gl.bindVertexArray(null);
}

function parseAndUpdateCells(
  data: string,
  cells: Cell[],
  cols: number,
  rows: number,
  setCursorPos: (pos: { row: number; col: number }) => void,
) {
  // Simple VT parsing - in production, use a proper VT parser
  let col = 0;
  let row = 0;

  for (const char of data) {
    if (char === '\n') {
      row++;
      col = 0;
    } else if (char === '\r') {
      col = 0;
    } else if (char === '\x1b') {
      // Skip escape sequences (simplified)
      continue;
    } else if (char >= ' ') {
      const idx = row * cols + col;
      if (idx < cells.length) {
        cells[idx] = { char, fg: 0xcccccc, bg: 0x1e1e2e, flags: 0 };
      }
      col++;
      if (col >= cols) {
        col = 0;
        row++;
      }
    }

    if (row >= rows) {
      // Scroll up
      cells.copyWithin(0, cols);
      for (let c = 0; c < cols; c++) {
        cells[(rows - 1) * cols + c] = { char: ' ', fg: 0xcccccc, bg: 0x1e1e2e, flags: 0 };
      }
      row = rows - 1;
    }
  }

  setCursorPos({ row, col });
}
