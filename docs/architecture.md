# Production architecture

This document is the authoritative implementation map for the production
application. The repository contains the product source, release automation, and
documentation only; obsolete implementation experiments are not shipped.

## Goals

- Route a user-recognizable Application to an exact Desired Output and restore
  that route when the output returns.
- Share routing semantics, configuration, and state transitions in Rust while
  using native desktop shells.
- Remain event-driven and effectively idle when nothing changes.
- Keep platform differences visible through capabilities instead of pretending
  that all operating systems behave alike.
- Be small enough for contributors to understand without a framework tour.
- Preserve local-only operation, accessibility, and safe failure as release
  requirements.

## Non-goals

- A common cross-platform UI implementation.
- A daemon, Windows service, privileged helper, account, cloud API, database, or
  telemetry pipeline.
- A plugin system, dependency-injection framework, internal message bus, or
  generic workflow engine.
- Bit-for-bit visual identity across operating systems at the expense of native
  behavior.
- Hidden fallbacks that report an unsupported routing operation as successful.

## Modules and seams

The architecture has three production modules. Each is deliberately deep: the
interface is small and the platform or product complexity stays behind it.

```text
                         user input
                             |
                +------------v-------------+
                | native desktop shell     |
                | tray/menu, popup, mixer  |
                | theme, text, a11y, icons |
                +------------+-------------+
                             | Command
                             v
                +--------------------------+
                | router engine (Rust)     |
                | identity/grouping        |
                | desired/effective routes |
                | failure deduplication    |
                | config and snapshots     |
                +------------+-------------+
                             | AudioRequest
                             v
                +--------------------------+
                | platform audio adapter   |
                | Win Core Audio / macOS   |
                | Core Audio / PipeWire    |
                +------------+-------------+
                             |
                         operating system

OS callbacks --------> PlatformEvent --------> router engine
router engine --------> immutable Snapshot --> native desktop shell
```

The first seam is the engine's command/snapshot interface. A shell knows no
audio handles, process handles, COM objects, PipeWire objects, or persistence
rules. The second seam is the platform-audio interface. The engine knows stable
identities and capabilities, not native handles or thread models.

Windows, macOS, and Linux are three real adapters at the audio seam. Native UI
automation may use a small in-memory adapter in tests; no public abstraction is
added solely to make mocking convenient.

## Proposed repository tree

Create production directories only when their phase begins. The intended final
shape is shown here so names and dependency direction remain stable.

```text
/
|-- Cargo.toml                    # Rust workspace, once production starts
|-- CONTEXT.md                    # domain language
|-- README.md
|-- docs/
|   |-- architecture.md
|   `-- adr/
|-- crates/
|   `-- router-engine/            # shared Rust model, runtime, config
|       |-- Cargo.toml
|       `-- src/
|-- platforms/
|   |-- windows/                  # Rust executable and native Win32 shell
|   |-- macos/
|   |   |-- rust/                 # Rust static library and Core Audio code
|   |   `-- App/                  # thin Swift/AppKit application target
|   `-- linux/                    # Rust executable, PipeWire, desktop shell
|-- design/
|   |-- tokens.json               # colors, spacing, type roles, motion rules
|   `-- references/               # approved screenshots and interaction specs
|-- locales/
|   `-- en-US.json                # initial and fallback catalog
`-- tests/                        # cross-adapter fixtures only when needed
```

Do not split `router-engine` into domain/application/config crates until an
independent consumer or a build constraint makes that split pay for itself.
Platform source can use internal files freely; directories are not automatically
new crates or new interfaces.

## Dependency direction

```text
platform executable or macOS bridge
    -> router-engine
    -> Rust standard library and the minimum serialization support

platform shell
    -> native OS UI and accessibility facilities

platform audio adapter
    -> native OS audio facilities
```

`router-engine` never depends on a platform crate, UI toolkit, localization
catalog, renderer, or OS handle type. Platform code may translate both ways at
the seam. No platform adapter may call another platform adapter.

The visual tokens and locale catalog are build inputs, not runtime dependencies
of the routing model. Generated platform resources are acceptable when native
packaging requires them.

## Engine interface

Names may change during implementation, but the interface should remain this
small in spirit:

```rust
pub struct Router { /* owns the worker and latest snapshot */ }

impl Router {
    pub fn start(options: StartOptions, audio: Box<dyn PlatformAudio>)
        -> Result<Self, StartError>;
    pub fn dispatch(&self, command: Command) -> Result<(), DispatchError>;
    pub fn snapshot(&self) -> Snapshot;
    pub fn shutdown(self);
}

pub trait PlatformAudio: Send {
    fn capabilities(&self) -> Capabilities;
    fn start(&mut self, events: PlatformEventSender) -> Result<(), AudioError>;
    fn observe(&mut self) -> Result<Observation, AudioError>;
    fn execute(&mut self, request: AudioRequest) -> Result<(), AudioError>;
}
```

Each adapter is constructed by its platform host because initialization and
permissions are platform-specific. `start` registers native callbacks with the
engine's event notifier. `observe` is called after startup or a callback, never on
a timer. It returns value types with no borrowed native handles. `execute` covers
only operations needed by accepted product commands. Teardown is owned by the
adapter's `Drop` implementation, so partial startup also cleans up.

The trait is intentionally not an exhaustive mirror of Core Audio or PipeWire.
Platform-only operations remain private until product behavior needs them.

## Commands, events, and ownership

Initial commands are explicit product intentions:

```rust
pub enum Command {
    SetRoute { application: ApplicationId, output: OutputId },
    FollowSystemDefault { application: ApplicationId },
    SetVolume { application: ApplicationId, level: UnitInterval },
    SetMuted { application: ApplicationId, muted: bool },
    SetFavorite { output: OutputId, favorite: bool },
    ReassociateApplication { old: ApplicationId, new: ApplicationId },
    ReassociateOutput { old: OutputId, new: OutputId },
    SavePreferences { preferences: ShellPreferences },
}
```

Settings that do not affect routing, such as theme, locale, shortcuts, and
launch-at-login, are interpreted and applied by the shell. `SavePreferences`
only gives the engine one atomic writer for their durable values; it causes no
audio operation.

`PlatformEvent` is a wake-up fact such as devices changed, default changed,
streams changed, or permission changed. It does not carry an OS object. After an
event, the engine asks the adapter for a fresh observation and reconciles it.
Bursts may be coalesced before observation, but correctness may not depend on
receiving every duplicate callback.

The engine owns:

- Application grouping and identity association;
- Desired Output and Effective Output state;
- favorites and persistent routes;
- fallback and exact-device restoration;
- failure-episode deduplication;
- configuration schema, migration, atomic save, and previous-copy recovery; and
- the immutable Snapshot consumed by every shell.

The audio adapter owns native audio handles, callback registration, native error
codes, and conversion to/from platform identities. The shell owns popup lifetime,
selection, search text, expanded rows, theme resolution, localized text, icons,
notifications, shortcuts, launch-at-login, and accessibility objects.

Only the engine worker mutates routing state. Shells read immutable snapshots;
they never patch shared state in place. An accepted command means queued, not
successfully applied. Completion or failure appears in a later snapshot and, when
useful, a structured notice.

## Event and thread flow

### Windows

Use two long-lived threads:

1. The main STA thread owns the Win32 message loop, tray icon, hotkeys, popup,
   Direct2D/DirectWrite drawing, DWM integration, and accessibility providers.
2. One MTA worker owns the router engine and all Core Audio COM objects.

Core Audio callbacks can arrive on system threads. A callback copies only the
minimum stable identifier, enqueues a `PlatformEvent`, and wakes the worker. It
does not block, draw, persist, enumerate, or call back into Core Audio. The worker
reconciles state and posts one private window message when a new snapshot is
available. UI resources for the popup and mixer are created lazily and released
when closed; the tray and message loop remain.

### macOS

The AppKit main thread owns the menu-bar item, windows, accessibility, localized
resources, permission presentation, and appearance. A Rust worker owns the router
engine. Commands cross the C ABI into the worker queue; an ABI callback only
signals that a snapshot or notice changed, then Swift dispatches UI work onto the
main queue.

Core Audio C interfaces should stay in the Rust macOS adapter where that is clear
and safe. A tiny Swift or Objective-C shim is allowed for an operating-system API
that is materially safer or only practical through Apple language bindings. Such
a shim translates native callbacks and operations; it must not contain route
policy, persistence, grouping, or fallback behavior.

No real-time audio callback may allocate, lock, write a file, invoke Swift UI, or
enter the router model. If the chosen macOS routing technique handles audio data,
its real-time handoff requires a separately reviewed fixed-capacity mechanism.
Until that feasibility work is complete, the capability must report unavailable
rather than ship an unsafe path.

### Linux

The desktop main thread owns the tray/status item, popup, mixer, accessibility,
localization, and Wayland/X11 presentation. PipeWire objects live on their required
loop thread. PipeWire callbacks enqueue facts for the Rust engine; commands return
to the PipeWire loop through its supported invocation mechanism. Snapshot changes
wake the desktop main context. There is no sampling timer.

PipeWire is the only initial audio adapter. Wayland and X11 are presentation modes
of one shell, not separate routing backends. The exact native shell library is
chosen by a measured prototype covering memory, accessibility, packaging, status
items, Wayland, and X11; that choice does not change the engine interface.

## Rust and Swift C ABI policy

The macOS bridge exports a narrow C ABI around an opaque router handle:

- create/start and destroy;
- dispatch one of the supported commands;
- copy the current versioned snapshot; and
- register one non-reentrant change notification callback.

Use fixed-width integers, UTF-8 byte slices with explicit lengths, and caller-owned
input memory. Rust allocates output only through a paired export that also frees
it. Every pointer and length is validated before use. No Rust reference, `String`,
enum layout, panic, exception, Swift object, or native handle crosses the ABI.

The callback has no borrowed payload and remains valid only for the duration of
the registration. It may set a flag or schedule main-thread work; it must not call
back into the Rust handle. Destruction first disables callbacks, then joins the
worker, then frees the handle. Exported Rust functions catch panics and return a
stable status code; no panic may unwind into Swift.

Prefer generated C headers from one checked-in declaration source. Any ABI shape
change increments its version and retains compatibility only when a released
macOS build actually needs it. This is an in-process bridge, not an IPC protocol.

## Capability and error model

Capabilities describe operations, not operating-system names. The initial set
includes application routing, current volume, current mute, persistent future-
stream routing, device notifications, and stream detail. Each is
`Available`, `Unavailable(reason)`, or `PermissionRequired`; availability may
change at runtime.

The engine rejects unsupported commands before mutation. Platform failures map to
a compact stable category:

```text
Unsupported | PermissionDenied | ApplicationGone | OutputUnavailable
BackendUnavailable | InvalidInput | ConfigCorrupt | Internal
```

An error also records the attempted operation, whether retry can make sense, and
an optional native numeric code for local diagnostics. It does not contain
localized prose. The shell maps the category and structured arguments to text.
Sensitive names and paths are excluded from ordinary logs and redacted from
diagnostic exports by default.

Expected races are ordinary outcomes: an application can exit, a stream can end,
or a device can disappear between snapshot and command. They do not crash the
worker or erase the Desired Output. A failed route uses the system default as the
Effective Output and produces at most one quiet notice per failure episode.

## Configuration and localization

One versioned JSON document lives in the conventional per-user configuration
directory. The engine owns the routing portion and migration. Platform shells own
appearance, locale, shortcut, launch-at-login, and window preferences. A single
process coordinates writes: write a new file in the same directory, flush it,
atomically replace the current file, and keep the immediately previous valid copy.

Unknown fields survive only if a demonstrated forward-compatibility need appears;
do not build a generic extension map preemptively. Invalid fields fall back
individually where safe. An invalid routing identity never falls back by matching
display names.

`locales/en-US.json` is the initial and fallback catalog. UI snapshots and errors
carry stable keys plus typed arguments, never English sentences. Shells perform
formatting and use native locale rules. Backend and domain code contain no
user-visible strings. Missing keys are a test failure in release builds and fall
back to English at runtime.

The design source defines semantic tokens rather than platform effects: surface,
text roles, accent, spacing, radius, elevation, focus, and motion. Light, dark,
glass, and platform acrylic materials resolve inside each shell. High contrast,
reduced transparency, reduced motion, remote sessions, or unsupported composition
must fall back to an opaque accessible surface without changing layout or meaning.

## Testing strategy

- Test the engine through commands and snapshots. Cover grouping, exact identity,
  route persistence, disconnected-output fallback, restoration, reassociation,
  failure deduplication, capability rejection, and config migration.
- Use a small scripted in-memory audio adapter for engine tests. It represents the
  already-real platform seam; do not expose additional internals for tests.
- Run contract tests against every platform adapter for supported capabilities,
  including callbacks, disappearing applications/devices, and permission changes.
- Keep hardware-dependent tests opt-in and label their device requirements. CI
  smoke tests must still exercise enumeration and clean shutdown on each OS.
- Test the C ABI from Swift for invalid pointers/lengths, repeated startup/shutdown,
  callback shutdown races, Unicode identifiers, and version mismatch.
- Test config recovery by interrupting writes and loading previous schema versions.
- Add UI smoke coverage for keyboard-only use, screen readers, high contrast,
  scaling, light/dark/glass fallbacks, and popup focus/closure behavior.
- Continue the existing benchmark method for release builds: repeated warm runs,
  medians and P95 where meaningful, binary size, startup, scan latency, private
  memory, working set, idle CPU, threads, and handles. A dependency that changes
  idle cost needs measurements, not intuition.

The interface is the test surface. Tests should survive internal refactors and
must not assert COM vtable positions, PipeWire object layout, or private state.

## Comments and unsafe code

Code, identifiers, comments, logs, and contributor documentation are English.
Public interfaces document invariants, ordering, error behavior, thread affinity,
and ownership. Comments explain why, platform quirks, and non-obvious invariants;
they do not narrate syntax.

Every `unsafe` block has an adjacent `// SAFETY:` explanation covering pointer
validity, lifetime, aliasing, thread rules, ownership, and the native contract being
relied on. Keep unsafe code inside platform adapters and the C ABI bridge. Safe
wrappers should own COM reference counts, callback unregistration, native strings,
and handles so application logic cannot forget cleanup.

Review any unsafe callback for teardown races and any audio-thread code for real-
time safety. Warnings are denied in CI. Formatting, linting, and tests are required
for changed Rust code; native shell linters apply to Swift or other shell code.

## Dependency policy

Start with the standard library and operating-system facilities. Dependencies are
accepted only when they remove more correctness or maintenance risk than they add.
Use narrowly selected features, commit lockfiles, check licenses, and record binary
and idle-memory impact for runtime dependencies.

Likely justified dependencies are maintained platform bindings and JSON
serialization. Do not add an async runtime, logging framework, UI framework shared
across platforms, reactive state framework, general event bus, DI container,
database, HTTP client, updater, crash SDK, or plugin loader for the first release.
Native callbacks plus the standard channels and OS wake mechanisms are sufficient.

The Windows production adapter uses maintained Rust Windows bindings with only
the required features. Linux uses PipeWire's supported C interface through
maintained bindings. macOS uses Apple frameworks with the smallest maintained
binding surface that satisfies the implementation. The Linux shell-library
decision requires its own measured ADR because there is no single desktop-native
UI stack.

## Implementation sequence

1. Keep the Rust workspace limited to `router-engine` and the platform hosts.
2. Evolve the Windows beta through focused native integration tests, with routing,
  volume, and device-change failures reported honestly.
3. Keep drawing resources lazy and measure closed-popup idle cost after each
  substantial Windows feature.
4. Build a macOS routing feasibility spike, including permissions and real-time
   constraints. Choose direct Rust bindings or one tiny Swift shim per the ABI
   policy, then add the AppKit shell and universal packaging.
5. Build a measured Linux shell spike, implement PipeWire routing, and validate one
   Wayland and one X11 desktop before choosing the shell library.
6. Move only shared fixtures that have two consumers into root `tests/`. Keep each
   platform's native integration tests beside that platform.

Each step must leave one working vertical path and updated measurements. Directory
creation and abstraction extraction happen when the step needs them, not in an
up-front scaffold commit.

## Explicitly deferred

- Profiles and event-driven automatic rules (post-V1).
- Taskbar-adjacent selection, system volume-panel integration, and any Explorer or
  shell hook (after the routing core is stable and independently removable).
- Per-process or per-session route overrides and application-managed volume
  persistence.
- PulseAudio-only Linux support and official Linux ARM64 support.
- Automatic direct-download update checks, cloud features, accounts, telemetry,
  uploaded crash reports, and background network traffic.
- A public extension/plugin interface, headless daemon, remote control interface,
  IPC protocol, and configuration extension map.
- Final product name, icon, and visual artwork; those may change tokens and assets
  but not routing interfaces.
- A final Linux shell library and any optional macOS language shim until their
  focused prototypes are measured and reviewed.

Deferred work does not receive placeholder traits, empty crates, feature flags, or
configuration keys. It begins with a requirement and evidence when its phase
starts.
