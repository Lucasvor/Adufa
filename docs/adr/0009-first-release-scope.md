# First release scope

- Status: Accepted
- Date: 2026-09-19

## Context

The product exploration includes quick routing, a mini mixer, profiles, automatic
rules, and operating-system shell integrations. Shipping all of them together would
couple the reliable routing core to features with different risk and maturity.

## Decision

The first release includes:

- the compact tray or menu-bar popup;
- the expanded mini mixer;
- application and output search;
- favorite outputs;
- persistent application-level routes;
- current volume and mute controls;
- optional launch at login; and
- configurable global shortcuts.

Launch at login is disabled by default and offered during initial setup and in
settings.

The default open shortcut is `Ctrl+Alt+A` on Windows and Linux and `Command+Option+A`
on macOS. Registration uses supported system APIs, remains configurable, and fails
visibly without preventing the application from running.

Profiles and event-driven automatic rules are planned for the following feature
release. Taskbar-adjacent and system-volume-panel integrations follow only after the
core is stable and are isolated platform modules.

## Consequences

- The first release provides the complete manual routing workflow.
- Startup behavior remains an explicit user choice.
- Shortcut conflicts degrade gracefully.
- Deferred features reuse the same routing commands instead of expanding the first
  release architecture prematurely.
