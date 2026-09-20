# Application list and routing failures

- Status: Accepted
- Date: 2026-09-19

## Context

One user-recognizable application can own several processes and audio sessions.
Presenting every session as a separate application creates noise, while hiding all
session detail makes diagnosis difficult. Routes can also become temporarily
unavailable when a device disconnects or the operating system rejects an operation.

## Decision

- The primary list shows applications with active audio first.
- Configured applications without active audio appear in a collapsed inactive
  section.
- Processes and audio sessions are grouped under their Application identity.
- An Application row can expand to reveal its current child processes and sessions.
- A persistent Application Route applies to current and future sessions belonging to
  that Application.
- When the Desired Output is unavailable or routing fails, use the system default as
  the Effective Output without deleting the desired route.
- Show the unavailable or failed state in the interface and issue at most one quiet
  notification for each distinct failure episode.

## Consequences

- Browser and multi-process applications remain understandable in the main view.
- Advanced information remains available without becoming the default interface.
- Repeated callbacks for the same failure cannot create notification spam.
- Returning devices can restore their exact saved routes automatically.
