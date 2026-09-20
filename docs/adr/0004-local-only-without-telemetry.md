# Local-only operation without telemetry

- Status: Accepted
- Date: 2026-09-19

## Context

Audio routing observes applications, processes, audio sessions, and output devices.
This is sensitive local activity, and the product does not require an online service
to perform its core function.

## Decision

The application works locally without an account, cloud service, analytics, or
telemetry. Settings, application identities, device identities, routes, and profiles
remain on the user's machine.

Network access is not part of the routing core. Any future networked feature must be
optional, disabled by default, isolated from the core, and approved by a new ADR.

## Consequences

- Core routing remains usable offline.
- Builds must not silently introduce analytics or crash-reporting SDKs.
- Diagnostics are exported explicitly by the user when support data is needed.
- Store packaging and update mechanisms must not change this privacy boundary.
