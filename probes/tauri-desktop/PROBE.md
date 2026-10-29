# Tauri desktop capability probe

Isolated feasibility experiment, not the LittleTodo product. Generated with the official create-tauri-app CLI (reported binary 4.8.0; npm registry package latest resolved 4.7.5); exact application dependencies are pinned by package-lock.json and src-tauri/Cargo.lock.

Scope: control window, synthetic task panel, foreground editable detail window, background alpha/color, own-window desktop parenting. No database, notification, autostart, telemetry, remote content or OS input hooks. Rust desktop adapter is Windows-only; no macOS success claim.

## Build on Windows

Requires installed Rust MSVC toolchain, C++ BuildTools, WebView2 and Node. Rust was installed with official hash-verified rustup minimal profile, without modifying PATH.

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
# First time only: npm ci (network may be needed).
$env:CARGO_NET_OFFLINE = 'true'
npm run tauri build -- --debug --no-bundle
```

Run from this probe directory after the build. For event evidence, set an absolute log destination before launch:

```powershell
$env:LITTLETODO_PROBE_LOG = Join-Path (Get-Location) 'tauri-probe.log'
Start-Process -FilePath '.\src-tauri\target\debug\tauri-desktop.exe' -WindowStyle Normal
```

Debug exe size/memory are not release performance results; no installer is generated. VERSION remains V0.1.0 for this isolated initial experiment; probe manifest copies match its numeric version.

## Checks

Before attaching, verify WebView renders, checkbox works, detail accepts edits, and background alpha/color changes. Then click 挂到桌面; success must be corroborated by desktop visibility and interaction, not just a parent handle.
Show Desktop: task panel should remain visible, ordinary applications should cover it, drag header and edit button should work; repeat alpha/color checks after attaching. Events show requested and applied appearance separately. Detail text itself is never logged.

Exit via 退出探针 closes the entire process. No permanent startup/service registration. Panel may initially appear in taskbar to allow automation selection; after attachment it is hidden from taskbar. The current probe positions to fixed coordinates (1500,80), not production multi-monitor/DPI recovery.

Limits: uses existing undocumented desktop host topology, despite documented SetParent; Explorer restart/DPI/IME are separate checks. Drag coordinates mix browser and native coordinates and require scaling validation. macOS adapter is intentionally absent.
