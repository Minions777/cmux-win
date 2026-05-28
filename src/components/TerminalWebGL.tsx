import { useEffect, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useTerminalStore } from '../stores/terminalStore';
import type { TerminalState, Cell } from '../types/terminal';

interface TerminalWebGLProps {
  terminalId: string;
  width: number;
  height: number;
}

// Vertex shader for character rendering
const VERTEX_SHADER = `#version 300 es
precision highp float;

// Per-vertex attributes
in vec2 a_position;    // Quad vertex position
in vec2 a_texCoord;    // Texture coordinate

// Per-instance attributes
in vec2 a_cellPos;     // Cell position in grid
in vec4 a_fgColor;     // Foreground color
in vec4 a_bgColor;     // Background color
in vec2 a_charIndex;   // Character index in atlas

// Uniforms
uniform vec2 u_resolution;
uniform vec2 u_cellSize;
uniform vec2 u_atlasSize;

// Outputs
out vec2 v_texCoord;
out vec4 v_fgColor;
out vec4 v_bgColor;
out vec2 v_charIndex;

void main() {
    // Calculate cell position in pixels
    vec2 cellPixel = a_cellPos * u_cellSize;
    
    // Scale quad to cell size
    vec2 scaledPos = a_position * u_cellSize;
    
    // Final position
    vec2 position = cellPixel + scaledPos;
    
    // Convert to clip space
    vec2 clipSpace = (position / u_resolution) * 2.0 - 1.0;
    gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
    
    // Pass to fragment shader
    v_texCoord = a_texCoord;
    v_fgColor = a_fgColor;
    v_bgColor = a_bgColor;
    v_charIndex = a_charIndex;
}
`;

// Fragment shader for character rendering
const FRAGMENT_SHADER = `#version 300 es
precision highp float;

in vec2 v_texCoord;
in vec4 v_fgColor;
in vec4 v_bgColor;
in vec2 v_charIndex;

uniform sampler2D u_atlas;
uniform vec2 u_atlasSize;

out vec4 fragColor;

void main() {
    // Calculate texture coordinate in atlas
    vec2 atlasCoord = (v_charIndex + v_texCoord) / u_atlasSize;
    
    // Sample font atlas
    float alpha = texture(u_atlas, atlasCoord).r;
    
    // Mix foreground and background based on alpha
    vec4 color = mix(v_bgColor, v_fgColor, alpha);
    
    fragColor = color;
}
`;

class WebGLTerminalRenderer {
  private gl: WebGL2RenderingContext;
  private program: WebGLProgram;
  private vao: WebGLVertexArrayObject;
  private atlasTexture: WebGLTexture;
  private cells: Cell[][] = [];
  private cols: number;
  private rows: number;
  private cellWidth: number = 8;
  private cellHeight: number = 16;
  private atlasSize: [number, number] = [16, 16]; // 16x16 character grid in atlas

  constructor(
    canvas: HTMLCanvasElement,
    cols: number,
    rows: number,
  ) {
    const gl = canvas.getContext('webgl2', {
      antialias: false,
      alpha: false,
      premultipliedAlpha: false,
    });

    if (!gl) {
      throw new Error('WebGL2 not supported');
    }

    this.gl = gl;
    this.cols = cols;
    this.rows = rows;

    // Initialize shader program
    this.program = this.createProgram(VERTEX_SHADER, FRAGMENT_SHADER);

    // Create font atlas texture
    this.atlasTexture = this.createFontAtlas();

    // Create buffers and VAO
    this.vao = this.setupVAO();

    // Initialize cell grid
    this.initCells();

    // Set up viewport
    this.resize(canvas.width, canvas.height);
  }

  private createShader(type: number, source: string): WebGLShader {
    const gl = this.gl;
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      const info = gl.getShaderInfoLog(shader);
      gl.deleteShader(shader);
      throw new Error(`Shader compile error: ${info}`);
    }

    return shader;
  }

  private createProgram(vertexSrc: string, fragmentSrc: string): WebGLProgram {
    const gl = this.gl;
    const vertexShader = this.createShader(gl.VERTEX_SHADER, vertexSrc);
    const fragmentShader = this.createShader(gl.FRAGMENT_SHADER, fragmentSrc);

    const program = gl.createProgram()!;
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      const info = gl.getProgramInfoLog(program);
      gl.deleteProgram(program);
      throw new Error(`Program link error: ${info}`);
    }

    gl.deleteShader(vertexShader);
    gl.deleteShader(fragmentShader);

    return program;
  }

  private createFontAtlas(): WebGLTexture {
    const gl = this.gl;
    const texture = gl.createTexture()!;

    // Create a simple ASCII font atlas (placeholder)
    // In production, render actual font glyphs to a canvas and upload
    const atlasData = new Uint8Array(256 * 16 * 16);

    // Fill with simple patterns for ASCII 32-127
    for (let i = 32; i < 128; i++) {
      const x = (i % 16) * 16;
      const y = Math.floor(i / 16) * 16;

      // Create a simple block pattern for each character
      for (let py = 0; py < 16; py++) {
        for (let px = 0; px < 8; px++) {
          const idx = (y + py) * 256 + (x + px);
          // Simple pattern: some pixels on for visible chars
          atlasData[idx] = (px + py) % 3 === 0 ? 255 : 0;
        }
      }
    }

    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.R8,
      256,
      256,
      0,
      gl.RED,
      gl.UNSIGNED_BYTE,
      atlasData,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    return texture;
  }

  private setupVAO(): WebGLVertexArrayObject {
    const gl = this.gl;
    const vao = gl.createVertexArray()!;
    gl.bindVertexArray(vao);

    // Quad vertices (2 triangles)
    const quadVertices = new Float32Array([
      0, 0, 0, 0,
      1, 0, 1, 0,
      0, 1, 0, 1,
      1, 0, 1, 0,
      1, 1, 1, 1,
      0, 1, 0, 1,
    ]);

    const vertexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, quadVertices, gl.STATIC_DRAW);

    // Position attribute
    const posLoc = gl.getAttribLocation(this.program, 'a_position');
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 16, 0);

    // TexCoord attribute
    const texLoc = gl.getAttribLocation(this.program, 'a_texCoord');
    gl.enableVertexAttribArray(texLoc);
    gl.vertexAttribPointer(texLoc, 2, gl.FLOAT, false, 16, 8);

    // Instance buffers will be updated each frame
    gl.bindVertexArray(null);

    return vao;
  }

  private initCells() {
    this.cells = Array.from({ length: this.rows }, () =>
      Array.from({ length: this.cols }, () => ({
        char: ' ',
        fg: 0xffffff,
        bg: 0x000000,
        flags: 0,
      })),
    );
  }

  resize(width: number, height: number) {
    const gl = this.gl;
    gl.viewport(0, 0, width, height);

    // Update resolution uniform
    gl.useProgram(this.program);
    const resLoc = gl.getUniformLocation(this.program, 'u_resolution');
    gl.uniform2f(resLoc, width, height);

    const cellSizeLoc = gl.getUniformLocation(this.program, 'u_cellSize');
    gl.uniform2f(cellSizeLoc, this.cellWidth, this.cellHeight);
  }

  updateCell(row: number, col: number, cell: Cell) {
    if (row >= 0 && row < this.rows && col >= 0 && col < this.cols) {
      this.cells[row][col] = cell;
    }
  }

  updateFromState(state: TerminalState) {
    // Update cells from terminal state
    // This would parse the state grid and update cells
  }

  render() {
    const gl = this.gl;

    // Clear
    gl.clearColor(0.12, 0.12, 0.15, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);

    // Bind atlas texture
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.atlasTexture);
    const atlasLoc = gl.getUniformLocation(this.program, 'u_atlas');
    gl.uniform1i(atlasLoc, 0);

    // Update instance data and draw
    // In production, use instanced rendering with proper buffers
    // For now, draw each cell individually (simplified)

    gl.bindVertexArray(null);
  }

  destroy() {
    const gl = this.gl;
    gl.deleteProgram(this.program);
    gl.deleteVertexArray(this.vao);
    gl.deleteTexture(this.atlasTexture);
  }
}

export function TerminalWebGL({ terminalId, width, height }: TerminalWebGLProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<WebGLTerminalRenderer | null>(null);
  const { config } = useTerminalStore();

  const cols = Math.floor(width / 8);
  const rows = Math.floor(height / 16);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    canvas.width = width * window.devicePixelRatio;
    canvas.height = height * window.devicePixelRatio;

    try {
      rendererRef.current = new WebGLTerminalRenderer(canvas, cols, rows);
    } catch (err) {
      console.error('Failed to initialize WebGL:', err);
    }

    return () => {
      rendererRef.current?.destroy();
    };
  }, [width, height, cols, rows]);

  // Listen for terminal output
  useEffect(() => {
    const unlisten = listen<{ id: string; data: string }>(
      'terminal-output',
      (event) => {
        if (event.payload.id === terminalId) {
          // Parse VT output and update cells
          // rendererRef.current?.updateFromState(parsedState);
        }
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [terminalId]);

  // Animation loop
  useEffect(() => {
    let animationId: number;

    const animate = () => {
      rendererRef.current?.render();
      animationId = requestAnimationFrame(animate);
    };

    animationId = requestAnimationFrame(animate);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, []);

  // Handle keyboard input
  const handleKeyDown = useCallback(
    async (e: React.KeyboardEvent) => {
      e.preventDefault();

      let input = '';

      if (e.key === 'Enter') {
        input = '\r';
      } else if (e.key === 'Backspace') {
        input = '\x7f';
      } else if (e.key === 'Tab') {
        input = '\t';
      } else if (e.key === 'Escape') {
        input = '\x1b';
      } else if (e.key.startsWith('Arrow')) {
        const arrows: Record<string, string> = {
          ArrowUp: '\x1b[A',
          ArrowDown: '\x1b[B',
          ArrowRight: '\x1b[C',
          ArrowLeft: '\x1b[D',
        };
        input = arrows[e.key] || '';
      } else if (e.key.length === 1) {
        input = e.key;
      }

      if (input) {
        try {
          await invoke('write_to_terminal', {
            id: terminalId,
            data: Array.from(new TextEncoder().encode(input)),
          });
        } catch (err) {
          console.error('Failed to write to terminal:', err);
        }
      }
    },
    [terminalId],
  );

  return (
    <canvas
      ref={canvasRef}
      style={{ width, height, outline: 'none' }}
      tabIndex={0}
      onKeyDown={handleKeyDown}
    />
  );
}
