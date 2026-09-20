# Routing and volume semantics

- Status: Accepted
- Date: 2026-09-19

## Context

Application-level routes need stable semantics across operating systems even though
their audio-session models differ. Per-session controls and application-managed
volume persistence would add complexity and could fight native operating-system
behavior.

## Decision

- Expanding an Application shows its child processes and audio sessions for
  inspection only in the first release.
- Persistent routes are created and changed at the Application level.
- Selecting `System Default` removes the saved Application Route. The application
  then follows subsequent changes to the operating system's default output.
- The mini mixer changes current application volume and mute state through the
  platform backend.
- The product does not add its own volume persistence in the first release. It
  accepts whatever persistence the operating system provides.

## Consequences

- Child-session routing can be evaluated later as a capability-specific temporary
  override without complicating the initial model.
- `System Default` has one clear meaning and cannot leave a hidden desired route.
- The application does not repeatedly overwrite volume chosen elsewhere.
- Volume persistence may differ by platform and must be described honestly in the
  capability report and user documentation.
