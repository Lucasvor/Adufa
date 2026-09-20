# Supported platform baselines

- Status: Accepted
- Date: 2026-09-19

## Context

The product targets Windows, macOS, and Linux while relying on different native
audio and desktop capabilities. A single lowest-common-denominator implementation
would either lose important features or require unnecessary compatibility layers.

## Decision

Support these initial platform baselines:

- Windows 10 22H2 and Windows 11. Core routing and the primary popup support both;
  advanced taskbar integrations may require Windows 11.
- macOS 14.2 or later, distributed as a universal application for Apple Silicon and
  Intel. This baseline provides the official Core Audio process-tap API needed by
  the macOS routing design.
- Linux desktops using PipeWire, on both Wayland and X11. A legacy PulseAudio
  backend is outside the first release.

Each platform reports explicit capabilities. The shared core must not pretend that
an unavailable platform operation succeeded.

## Consequences

- Platform-specific features can degrade independently without breaking core use.
- Windows 10 receives a native fallback where Windows 11 shell integration is not
  available.
- macOS packaging produces both ARM64 and x86_64 slices.
- Linux testing covers PipeWire under at least one Wayland and one X11 desktop.
- Legacy macOS versions and PulseAudio-only Linux systems are not release blockers.
