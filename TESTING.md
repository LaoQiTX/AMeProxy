# Core regression checks

Run frontend checks from the repository root:

```powershell
npm run lint
npm test
node node_modules/vite/bin/vite.js build
```

The direct Vite command verifies the frontend without triggering the existing npm
`prebuild` hook, which downloads a kernel. TypeScript lint currently checks `.ts`
files; Vite also compiles the Vue components.

Run backend checks from `src-tauri`:

```powershell
cargo check --locked --offline
cargo test --locked --offline --lib
```

With the local mihomo binary available in `src-tauri/sidecar`, also run:

```powershell
cargo test --locked --offline --lib -- --include-ignored
```

The optional integration test uses a temporary configuration directory, independent
loopback ports, a local HTTP origin, and a test-only controller secret. It checks
port conflicts, concurrent startup, subscription reference updates, persisted rule
reloads, invalid configuration preservation, authenticated API calls, proxy
forwarding, latency requests, connection deletion, and stop/restart behavior. It
does not read real subscriptions, modify the system proxy, or enable TUN.

For manual verification, start the desktop application with `npm run tauri:dev`.
Browser-only preview intentionally reports that native commands require Tauri.
Import a supported Clash YAML subscription, wait for its loaded status, and select
a node in the default group. Click **开启代理**: the backend starts the kernel,
waits for its authenticated controller and HTTP/mixed listener, then applies the
current user's Windows LAN HTTP/HTTPS proxy settings via WinINet. The dashboard
reports kernel and system proxy states independently. Importing subscriptions and
initial app startup only load the kernel; they do not enable the system proxy.

Click **关闭代理** to restore the previous proxy/PAC/auto-detect/bypass settings
before stopping the kernel (including any TUN session). Normal window close does
the same; a restoration error keeps the window and kernel open for retry. A Rust
background monitor restores proxy settings if the owned kernel dies, even when
the UI is hidden. A durable recovery journal is kept next to the kernel config.
After a forced app termination, reopening the app restores that journal before
starting the kernel; it cannot restore immediately while the app is not running.
Another app's subsequent proxy changes are preserved. Only one desktop instance
is allowed per Windows logon session to prevent competing recovery operations.

Automated system-proxy tests use a fake OS backend and temporary recovery files;
the native WinINet smoke test only reads settings. They verify idempotent enable,
PAC/bypass/auto-detect restoration, recovery after manager recreation, external
changes, write failures, retained recovery records, and corrupt records. Frontend
tests cover enabling with an already running kernel, disable, failed restoration,
external changes, and browser preview. No automated test changes the user's proxy.

For an opt-in live desktop smoke test, record the existing Windows proxy settings,
enable from the app, open an HTTP/HTTPS page in a browser honoring system settings,
check live connections, and disable. Confirm Windows settings are restored. Repeat
with an existing PAC configuration, normal window close, and a stopped kernel.
System proxy applies to applications honoring Windows LAN proxy settings; it does
not configure WinHTTP services or separate dial-up/VPN profiles. TUN remains an
experimental feature requiring an appropriate configuration and permissions.

UI checks: desktop and narrow layouts, navigation, import/edit/rule dialogs and
Escape dismissal, disabled operations in browser preview, and theme persistence.
Themes use shared CSS variables and are saved in local storage.

Multiple subscriptions are saved in `config.yaml`, with the active name persisted
in `ameproxy-active-subscription`. Only the selected provider is emitted into the
generated `runtime.yaml` used at kernel startup, or into the live reload payload.
Provider-backed groups use the active provider; include-all groups can only see
that provider. The first import becomes active, later imports preserve the active
selection and do not reload the kernel when the runtime config is unchanged.
The dashboard selector and subscription cards switch the same persisted selection.
Inactive subscriptions show as saved, with no background loading/health checks.

Switching validates and reloads the selected runtime config, then closes existing
connections so old sessions cannot keep using the previous subscription. Failure
restores the old config/runtime. Renaming preserves the selection, deleting an
inactive subscription preserves the active one, and deleting the active one selects
a remaining provider (or falls back to DIRECT when none remain). Legacy merged
configs select the first provider referenced by the default/manual group.
Do not edit `runtime.yaml`; it is generated from the saved catalog at startup.

The real-kernel integration test now saves two local providers, confirms that the
inactive provider is absent from the API, switches with an open TCP request,
checks the old request closes and only the new provider's nodes remain, restarts
the kernel, and verifies an invalid switch preserves the working configuration.

Subscription and rule edits now validate a candidate with `mihomo -t`, back up the
previous YAML, replace it atomically, and reload a running kernel. Validation errors
leave the current file and running configuration untouched. A failed reload attempts
to restore both; restoration failures are reported with the backup location.
