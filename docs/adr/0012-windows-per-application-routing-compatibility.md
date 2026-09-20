# Windows per-application routing compatibility boundary

- Status: Accepted
- Date: 2026-09-19

## Context

The public Windows Core Audio API can enumerate endpoints and sessions, but it does
not provide a documented operation for assigning another process to a render
endpoint. Windows exposes that behavior through the internal
`Windows.Media.Internal.AudioPolicyConfig` activation factory. Its interface ID
changed in Windows 10 21H2, demonstrating that this is a compatibility-sensitive
boundary rather than a stable public contract.

The application must never report a successful route that Windows did not accept.
It must also keep this Windows-specific mechanism out of the portable domain model.

## Decision

- The Windows adapter isolates the private ABI in `platforms/windows/src/routing.rs`.
- Factory activation probes the current interface ID first and the downlevel ID
  second instead of relying on an OS-version string.
- A route writes both the multimedia and console roles for every process grouped
  under the selected application.
- Selecting `System Default` sends a null device string, which removes the explicit
  endpoint selection.
- Every native failure is returned to the caller with the affected process, role,
  and HRESULT. The UI updates its selected state only after a successful call.
- Adufa persists accepted routes by stable Application Identity in the versioned
  per-user configuration. One `RouterEngine` instance remains alive across audio
  observations; a refresh never recreates and empties its route map.
- The adjacent private `GetPersistedDefaultAudioEndpoint` ABI imports an existing
  Windows choice when Adufa has no route for that Application. It never erases
  Adufa's saved intent because Windows can temporarily report no value.
- On startup and whenever a new PID appears for a configured Application, the
  Windows adapter reapplies the saved route to that PID. This repairs the known
  gap between persisted policy and newly created application processes.
- Builds and smoke tests cover all supported Windows release baselines. Failure to
  activate this interface disables per-application routing and surfaces an honest
  capability error; it does not fall back to simulated success.
- No other crate or platform adapter may depend on this interface or its device-path
  encoding.

## Consequences

- Windows can provide the required per-application output selection with a very
  small native implementation and no resident helper process.
- A future Windows update may require another interface projection or may remove
  the mechanism. Keeping it behind one narrow module limits that maintenance risk.
- Existing streams may not move immediately when an application controls its own
  audio lifecycle; the persisted route still governs subsequently created streams.
- `GetPersistedDefaultAudioEndpoint` reports desired policy, not proof of the
  endpoint receiving a current stream. Effective Output remains a separate piece
  of observed session state.
- Automated ABI tests cannot replace real-machine smoke tests across the supported
  Windows matrix.
