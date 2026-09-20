# Language rollout and release architectures

- Status: Accepted
- Date: 2026-09-19

## Context

The project needs a stable source language, an extensible localization boundary, and
an achievable initial architecture matrix. Shipping translations before the English
interface stabilizes would create churn, while promising untested binaries would
reduce release quality.

## Decision

- English is the first interface language and the fallback locale.
- Users can select installed interface languages once translations are available.
- User-visible strings are externalized from the beginning; no UI text is embedded
  in domain or platform-backend logic.
- Code, code comments, technical logs, contributor documentation, and internal API
  names remain in English.
- The project name `Audio Router` is provisional. Final brand, package identifiers,
  store identities, and domains are selected before the first public release.

Initial release architectures are:

- Windows: x86_64 and ARM64;
- macOS: a universal application containing Apple Silicon and Intel slices;
- Linux: x86_64 supported, with ARM64 experimental until equivalent testing exists.

## Consequences

- Portuguese and other languages can be added without changing core code.
- Missing translations fall back to English instead of exposing localization keys.
- Release status describes tested support honestly for each architecture.
- Provisional names must not leak into permanent store or signing identifiers.
