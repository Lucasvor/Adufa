# Local configuration, updates, and diagnostics

- Status: Accepted
- Date: 2026-09-19

## Context

The application needs durable settings, maintainable distribution, and useful
support diagnostics without weakening its local-only privacy boundary.

## Decision

- Store settings as versioned JSON in the conventional per-user configuration
  directory for each operating system.
- Write settings atomically and retain the immediately previous valid copy for
  recovery.
- Manual editing is supported while the application is not running. The schema and
  migration rules are documented.
- Store and package-manager releases rely on their distribution channel for updates.
- Direct-download releases perform no background update requests. Users may invoke
  an explicit update check.
- Do not upload crashes, analytics, logs, or diagnostic data automatically.
- Generate a diagnostic report only when the user explicitly exports one.
- Redact application names, executable paths, device names, and other identifying
  values by default. The user may deliberately include them when needed for support.

## Consequences

- Settings survive interrupted writes and schema evolution.
- The routing core never requires network access.
- Direct-download users must choose when to contact the release service.
- Support reports require explicit user action and have a privacy-preserving default.
