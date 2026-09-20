# Native platform UIs with a shared visual language

Each operating system will have a native UI shell while sharing the Rust domain core, application behavior, color system, spacing, iconography, and information architecture. Minimum idle resource use is more important than sharing UI implementation, so Windows will use Win32/Direct2D/DirectWrite/DWM and macOS and Linux will use their native platform facilities instead of Slint, Electron, or a WebView-based toolkit. Slint remains suitable for disposable visual experiments, but it is not a production dependency unless a future measured decision supersedes this ADR.

## Consequences

The three UIs may differ where platform conventions require it, and contributors must maintain a small design specification plus platform screenshots. Platform-native shells cost more implementation effort, but can release visual resources while the tray application is idle and avoid carrying a cross-platform renderer in every process.
