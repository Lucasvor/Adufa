<div align="center">
  <img src="docs/assets/adufa-icon.svg" width="112" alt="Adufa icon: a sound channel redirected toward a selected output">
  <h1>Adufa</h1>
  <p><strong>Route every app to the right audio device.</strong></p>
  <p>A fast, local-first per-application audio output switcher and volume controller.</p>
</div>

<p align="center">
  <strong>English</strong> ·
  <a href="README.pt-BR.md">Português (Brasil)</a> ·
  <a href="docs/readme/README.es.md">Español</a> ·
  <a href="docs/readme/README.fr.md">Français</a> ·
  <a href="docs/readme/README.de.md">Deutsch</a> ·
  <a href="docs/readme/README.it.md">Italiano</a> ·
  <a href="docs/readme/README.ja.md">日本語</a> ·
  <a href="docs/readme/README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Windows beta</code> · <code>macOS planned</code> · <code>Linux planned</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — route every app to the right audio device](docs/assets/adufa-hero.svg)

## Why Adufa?

Video calls should use the headset. Music should use the speakers. A browser may
need a monitor or virtual cable. Adufa lets you see audible applications and send
each one to the output it belongs on—without digging through the Windows volume
mixer every time.

Adufa is deliberately small: it lives in the notification area, opens close to
where you are working, remembers application routes, and gets out of the way.

## See it in action

![A short Adufa walkthrough showing the tray popup, taskbar companion, volume control, and output selection](docs/assets/adufa-demo.gif)

The animation is a deterministic documentation rendering of the native interface;
it contains no captured desktop or personal data.

## What works today

The current public beta runs on **Windows 10 22H2 and Windows 11**.

- Automatically discovers applications with active audio sessions.
- Changes the output device for one application without changing the system default.
- Remembers application routes across Adufa and application restarts.
- Returns an application to `System default` with one selection.
- Changes current per-application volume and mute state.
- On supported Windows 11 builds, right-click a running taskbar app to open
  experimental output, volume, and mute controls beside the native menu, even
  before it has an active audio session.
- Includes **Find sound**, a temporary live view that highlights the loudest application.
- Opens a quick selector near the cursor with `Ctrl + Alt + A`.
- Can start when you sign in; this remains off until you enable it.
- Supports English, Brazilian Portuguese, Spanish, French, German, Italian,
  Japanese, and Simplified Chinese.
- Detects the Windows display language and allows a manual language override.
- Works locally, without accounts, analytics, telemetry, or audio uploads.

### Experimental Windows integration

On supported Windows 11 builds, right-clicking the taskbar icon of a running
application can open Adufa's compact companion beside the native taskbar menu,
even before that application starts producing audio. The native menu remains
available; Adufa complements it with volume and output controls.

This integration uses the taskbar button's AppID when available, with the
accessible name and executable path as fallbacks. It remains beta functionality
and can fall back to the global shortcut or tray popup when Windows does not
expose a reliable match.

## Install the beta

### Download a portable build

Tagged versions are built by GitHub Actions. Open the repository's
[Releases page](../../releases), download `Adufa-Windows-x64.exe`, and run it.

The initial beta executable is portable and unsigned. Windows may show a
SmartScreen warning until signed packages are available. Verify that the file came
from this repository's release before allowing it to run.

### Build from source

Requirements:

- Windows 10 22H2 or Windows 11, x64;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 or newer with the MSVC toolchain;
- Visual Studio Build Tools with **Desktop development with C++** and a Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Run:

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Use a release build for normal use. Release builds are Windows GUI applications
and do not open a terminal window. Debug builds intentionally keep a console for
diagnostics.

## How to use it

### Route an application from the tray

1. Start audio in the application you want to route.
2. Open the hidden notification-area icons and select Adufa.
3. Select the application row.
4. Choose an output, or choose `System default` to remove its saved route.
5. Click outside the popup or press `Esc` to dismiss it.

The route is stored against a stable application identity rather than a temporary
process ID. When an application restarts, Adufa restores the saved choice when the
platform can identify it safely.

### Find which application is making sound

1. Open Adufa.
2. Select **Find sound**.
3. Watch the live level indicators; the strongest audible source is emphasized.
4. Select that application to change its output.

Find sound does not record audio. It reads session peak levels that Windows already
provides and stops when the compact popup closes.

### Use the taskbar companion

1. Keep the target application running and right-click its taskbar icon. Audio
  does not need to be playing yet.
2. Use Adufa's adjacent panel to mute, adjust volume, or select an output.
3. Selecting an output closes both the companion and the native menu. Windows
  applies the selected route when the application's audio session is created.

If a companion does not appear, use `Ctrl + Alt + A` while pointing at the target
application, or open Adufa from the notification area.

### Change the language or startup behavior

Open **Settings** to:

- follow the Windows language or choose any supported language;
- enable or disable opening Adufa at sign-in;
- open the Windows volume mixer for system-level controls.

## Keyboard and mouse reference

| Input | Action |
| --- | --- |
| `Ctrl + Alt + A` | Open the quick selector near the cursor for the audible app under the pointer |
| Right-click a running taskbar app | Open the experimental Adufa companion beside the native menu |
| `Tab` or `↓` | Move to the next item |
| `↑` | Move to the previous item |
| `Enter` or `Space` | Activate the focused item |
| `Esc` | Close the current selector or return from Settings |
| Click outside | Dismiss transient Adufa windows |

The global shortcut can be unavailable when another application already owns it;
Adufa continues running and the tray workflow remains usable.

## Platform status

| Platform | Status | Planned backend |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Beta available** | Windows Core Audio / WASAPI and native Win32 UI |
| macOS 14.2+ | Planned | Core Audio with a native menu-bar interface |
| Linux, Wayland and X11 | Planned | PipeWire with a native desktop integration |

Cross-platform is the product direction, not a claim of feature parity today.
Each backend reports its capabilities explicitly so Adufa never pretends an
unsupported routing operation succeeded. The icon and core interaction language
are shared; platform behavior and materials remain native.

## Privacy

Adufa is local-only by design.

- No telemetry or analytics.
- No user account.
- No audio recording or upload.
- No network service required for routing.
- Routes and preferences are stored on your computer.

On Windows, the configuration is stored under the current user's local application
data directory. Removing the portable executable does not automatically delete
that preferences file.

## Roadmap

No dates are promised until the relevant platform implementation is proven.

### In development

- Harden the Windows beta across supported Windows 10 and 11 builds.
- Signed installer and portable release checksums.
- Accessible labels, high-contrast validation, and improved keyboard workflows.
- Translation review by native speakers.
- Reliable update checks that remain optional and privacy-preserving.

### Next

- Application and output search.
- Favorite outputs.
- Configurable global shortcuts.
- Expanded mini mixer.
- Profiles and automatic per-application rules.
- Better diagnostics and recoverable route reassociation.

### Planned platforms

- macOS 14.2+ using Core Audio, distributed for Apple Silicon and Intel.
- Linux using PipeWire on Wayland and X11; a PulseAudio-only backend is not part
  of the initial Linux release.

### Exploring

- Deeper native menu integrations where the operating system provides a safe API.
- Optional preference synchronization without uploading audio or activity data.
- Multiple device sets and reusable work, gaming, call, and streaming profiles.

## Troubleshooting

### An application is missing

Start playback and reopen Adufa. Some applications do not create an audio session
until they produce sound. Protected or system-owned sessions can expose less
identity information and may appear under a grouped application.

### A saved output is unavailable

Adufa keeps the desired route instead of silently replacing it with a similarly
named device. Reconnect the exact device, choose another output, or select
`System default`.

### The taskbar companion did not open

The integration is experimental and requires a unique match between the taskbar
icon and an audible application. Try the global shortcut or tray popup. Windows 10
uses the fallback workflow for integrations that only behave reliably on Windows 11.

### `Ctrl + Alt + A` does nothing

Another program may already own that global shortcut. Open Adufa from the tray;
configurable shortcuts are on the roadmap.

### A terminal window appears

You are probably running a debug build or launching through `cargo run`. Build and
start the release executable shown in [Build from source](#build-from-source).

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

To regenerate the committed icon package and documentation GIF, see
[scripts/README.md](scripts/README.md).

One live Windows audio-session round-trip test is ignored by default because it
changes a real session temporarily. Run it only on a development machine with an
active disposable audio session.

The repository is split into:

- `crates/router-engine`: platform-independent identities, commands, and state;
- `platforms/windows`: Windows audio, persistence, taskbar integration, and native UI;
- `docs/adr`: accepted architectural decisions;
- `docs/design`: interaction and identity research;
- `docs/assets`: generated documentation and brand assets.

The CI workflow validates formatting, Clippy, and tests, then produces the portable
Windows x64 executable. Pushing a tag such as `v0.1.0-beta.1` creates a GitHub
Release and attaches `Adufa-Windows-x64.exe`.

## Contributing

Bug reports should include the Windows version, the affected application, the
expected output, the observed output, and whether the tray, shortcut, or taskbar
workflow was used. Never attach recordings, configuration files, or logs containing
private paths without reviewing them first.

Contributions should preserve the core rules: stable application identity,
event-driven observation, honest capability reporting, native platform surfaces,
and no telemetry.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request,
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community expectations, and
[SECURITY.md](SECURITY.md) for private vulnerability reporting. See
[CHANGELOG.md](CHANGELOG.md) for beta changes and known limitations.

## License and name

Source code is licensed under **GPL-3.0-or-later**; see [LICENSE](LICENSE).
“Adufa” and the project artwork identify the official project builds; the
open-source license does not imply endorsement of modified distributions.

The name has passed only a preliminary web and repository collision screen. That
is not legal trademark clearance; official distribution should complete a formal
search in the intended launch territories.

---

<p align="center"><strong>Adufa</strong> — one small control surface for the audio paths your desktop hides.</p>
