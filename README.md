# 🪟 cmux-win

**A Windows terminal emulator inspired by [cmux](https://github.com/manaflow-ai/cmux), built with Tauri 2.0 + Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri 2.0](https://img.shields.io/badge/Tauri-2.0-ffc131)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-dea584)](https://rust-lang.org)

> 🚧 **Status: Early Development** — This project is in active development. Core terminal emulation is working, with more cmux features coming soon.

## ✨ Features

### Current (v0.1.0)
- 🖥️ **Terminal Emulation** — Full VT100/ANSI support via ConPTY + vte parser
- 📂 **Vertical Tab Sidebar** — Project-aware sidebar with git branch, status info
- 🔔 **Notification Ring** — Visual alert when a long-running command completes
- 🌐 **Built-in Browser** — WebView2-powered browser pane
- ⌨️ **CLI Scripting** — Named Pipes API for automation
- 🎨 **Theme System** — Dark/light themes with customizable colors
- 📐 **Split Panes** — Horizontal and vertical splits

### Planned (v0.2.0+)
- 🔐 **SSH Remote Workspaces**
- 🤖 **AI Agent Integration** (Claude Code, etc.)
- 📦 **Plugin System**
- 🔍 **Fuzzy Search**
- 📊 **Performance Metrics**

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Frontend (WebView2)                   │
│  ┌──────────┬─────────────────────┬──────────────────┐  │
│  │ Sidebar  │  Terminal Canvas    │  Browser Pane    │  │
│  │ (React)  │  (Canvas 2D/WebGL)  │  (WebView2)      │  │
│  └──────────┴─────────────────────┴──────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                   Rust Backend (Tauri)                   │
│  ┌───────────────┐ ┌──────────────┐ ┌────────────────┐  │
│  │ Terminal Mgr  │ │ VT Parser    │ │ Git Integration│  │
│  │ (ConPTY)      │ │ (vte crate)  │ │ (git2-rs)      │  │
│  └───────────────┘ └──────────────┘ └────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                   Windows OS Layer                       │
│  ConPTY │ Win32 Console │ Named Pipes │ Toast API      │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Windows 10** version 1809+ (for ConPTY support)
- **Rust** 1.75+ — [Install](https://rustup.rs)
- **Node.js** 18+ — [Install](https://nodejs.org)
- **Visual Studio Build Tools** — [Install](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/Minions777/cmux-win.git
cd cmux-win

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## 📁 Project Structure

```
cmux-win/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # UI components
│   │   ├── Sidebar.tsx     # Vertical tab sidebar
│   │   ├── Terminal.tsx    # Terminal canvas renderer
│   │   ├── TabBar.tsx      # Tab management
│   │   ├── NotificationRing.tsx
│   │   └── BrowserPane.tsx # Built-in browser
│   ├── hooks/              # React hooks
│   ├── stores/             # State management
│   └── styles/             # CSS styles
├── src-tauri/              # Rust backend
│   └── src/
│       ├── terminal/       # Terminal emulation core
│       │   ├── pty.rs      # ConPTY integration
│       │   ├── parser.rs   # VT sequence parser
│       │   ├── state.rs    # Terminal state
│       │   └── buffer.rs   # Screen buffer
│       ├── commands/       # Tauri IPC commands
│       └── event.rs        # Event system
├── package.json
└── Cargo.toml
```

## ⌨️ Default Keybindings

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+Shift+D` | Split pane horizontally |
| `Ctrl+Shift+E` | Split pane vertically |
| `Ctrl+Shift+C` | Copy |
| `Ctrl+Shift+V` | Paste |
| `Ctrl+L` | Clear terminal |
| `Ctrl+,` | Open settings |

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [cmux](https://github.com/manaflow-ai/cmux) — The original macOS terminal that inspired this project
- [Ghostty](https://github.com/ghostty-org/ghostty) — Terminal emulation engine
- [Tauri](https://tauri.app) — Cross-platform app framework
- [Alacritty](https://github.com/alacritty/alacritty) — VT parser reference
- [WezTerm](https://github.com/wez/wezterm) — Terminal emulation patterns
