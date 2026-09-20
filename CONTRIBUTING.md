# Contributing to Adufa

Thanks for helping make Adufa dependable. The project is a local-first audio
router: correctness, safety, accessibility, and small resource usage take
priority over adding surface area quickly.

## Before you start

1. Read [CONTEXT.md](CONTEXT.md) for the project's domain language.
2. Read [docs/architecture.md](docs/architecture.md) and relevant ADRs before
   proposing a structural change.
3. Search existing issues before opening a new one.
4. Open an issue first for new features, platform work, or changes to routing
   behavior. Small documentation and focused bug fixes can go directly to a
   pull request.

## Development workflow

Windows is the current beta platform. Use the MSVC Rust toolchain and run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release --locked -p router-windows
```

Do not commit `target/`, `artifacts/`, `.tmp/`, private logs, recordings, or
configuration files. Test changes involving live routing or volume only with
audio you are comfortable interrupting.

## Design and code expectations

- Keep the routing engine independent of platform UI and native handle types.
- Keep platform-specific unsafe code at the boundary and include a `SAFETY:`
  explanation for every unsafe block.
- Prefer event-driven native callbacks over polling.
- Preserve the distinction between a desired output and an effective output.
- Do not add telemetry, network dependencies, accounts, or background uploads.
- Write code, comments, API documentation, issues, and pull requests in English.
- Add or update focused tests for behavioral changes.
- Update an ADR when changing an accepted architectural decision.

## Pull requests

Keep pull requests small and describe the user-visible behavior, platform
coverage, validation performed, and any known limitation. Include screenshots
or a short capture for visual changes, with private information removed.

By submitting a pull request, you confirm that you have the right to submit the
work under GPL-3.0-or-later. A contributor license agreement is planned for
official dual-licensed distribution; do not assume this repository alone grants
rights to the Adufa name, logo, signing identity, or store listings.

## Reporting security issues

Do not file public issues for security-sensitive findings. Follow
[SECURITY.md](SECURITY.md) instead.
