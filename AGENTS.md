# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Install dependencies
npm install

# Tauri dev mode (starts both Vite frontend + Rust backend)
npm run tauri:dev

# TypeScript type checking (no emit)
npm run lint

# Production Tauri build (auto-runs prebuild for mihomo download)
npm run tauri:build

# Download/update mihomo kernel binary to src-tauri/sidecar/
npm run prebuild

# Rust-only check (syntax and type errors only, no final binary)
cd src-tauri && cargo check

# Rust full build
cd src-tauri && cargo build
```

## Project Architecture

### Core Concept
Desktop GUI (Tauri 2 + Vue 3) that manages the **mihomo** proxy kernel as a child process. The frontend communicates with mihomo's REST API at `http://127.0.0.1:9090` — both directly via `fetch`/WebSocket and indirectly through Tauri `invoke` commands (which proxy to the same API via Rust's reqwest client).

### Tauri Backend (Rust) — `src-tauri/src/`

```
main.rs → lib.rs (setup, state, command registration)
  ├── proxy/          — Kernel lifecycle
  │   ├── process.rs  — AppState: proxy_process (Child), start_time, start_core/stop_core/uptime
  │   ├── config.rs   — ClashConfig::generate_file() creates configs/mihomo/config.yaml
  │   └── paths.rs    — Centralized project/config/sidecar path helpers
  └── commands/       — Tauri command handlers (exposed via invoke_handler)
      ├── proxy.rs   — start/stop/get_proxies/change_proxy/test/get_rules/get_connections/uptime/tun
      ├── config.rs   — add/update/remove proxy providers via serde_yaml config editing
      └── api_client.rs — reqwest-based HTTP client for mihomo REST API
```

**Key Rust state**: `AppState { proxy_process: Mutex<Option<Child>>, start_time: Mutex<Option<Instant>> }` — managed by Tauri and passed to commands via `State<AppState>`.

### Frontend (Vue 3 + TypeScript + Pinia) — `src/`

```
main.ts → App.vue (sidebar + header + tab container)
  ├── components/common/    — Sidebar (nav + kernel status), Header (traffic + connect toggle)
  ├── components/dashboard/ — Dashboard (connection status, current node, traffic totals, theme picker)
  ├── components/proxies/   — ProxyGroups, ProxyNodes (subscription CRUD), Connections, Rules, Logs
  ├── components/settings/  — Settings (TUN, port, kernel select, subscription management)
  ├── stores/
  │   ├── proxyStore.ts     — All proxy state, WebSocket connections, polling, toggle/switch/fetch actions
  │   └── themeStore.ts     — 4 color themes (mint/sakura/lavender/sky)
  ├── services/proxy.ts     — REST API client (fetch to 127.0.0.1:9090, fallback to invoke)
  └── types/index.ts        — Connection, Rule, Log, ProxyGroup, Subscription, Proxy, Theme, TrafficData
```

### Data Flow
1. **Startup**: `lib.rs` setup() generates `configs/mihomo/config.yaml` → spawns mihomo with `-d config_dir` → frontend `initialize()` detects running state via `is_proxy_running()` → connects WebSockets
2. **Proxy Management**: Frontend fetches proxies/rules/groups from mihomo REST API (`/proxies`, `/rules`), displays in UI, sends PUT to switch nodes
3. **Real-time**: Three WebSocket connections (`/traffic`, `/logs`, `/connections`) provide live data
4. **Subscription**: User adds URL → `add_proxy_provider` writes to `config.yaml` via serde_yaml → mihomo restarts to load new providers
5. **Config file**: `configs/mihomo/config.yaml` — edited by Rust commands (serde_yaml), read by mihomo on startup. Auto-generated if missing with minimal defaults (mixed-port 7890, "直连" direct proxy).

### mihomo Binary
- Downloaded by `scripts/prebuild.mjs` from GitHub releases (with mirror fallbacks)
- Stored at `src-tauri/sidecar/my-mihomo-{target-triple}.exe`
- Launched with `-d <config_dir>` flag so it reads `configs/mihomo/config.yaml` and creates runtime files in that directory
- Version: v1.19.x (mihomo meta)
- Tauri `externalBin` in `tauri.conf.json` registers `sidecar/my-mihomo`

### Key Design Decisions
- Frontend talks to mihomo API directly (fetch/WebSocket) for most operations; Tauri commands are a fallback layer
- Vite config ignores `configs/` and `src-tauri/sidecar/` from HMR to avoid restart on file changes
- mihomo runs as a standard child process (not Tauri sidecar API) to avoid Tauri's sidecar naming requirements
- All paths are centralized in `paths.rs` — config at `configs/mihomo/`, sidecar at `src-tauri/sidecar/`
- Theme system uses Tailwind CSS classes mapped through Pinia store (4 presets)
- Windows is the primary dev target (CREATE_NO_WINDOW for mihomo process)
