## Summary

Describe the user-visible change and why it belongs in Adufa.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] I tested the affected native UI or audio behavior where applicable.

## Checklist

- [ ] Tests and documentation cover the changed behavior.
- [ ] No private logs, binaries, recordings, or generated build output are included.
- [ ] Platform capabilities and fallback behavior remain explicit.
- [ ] New UI strings include all supported translations.
- [ ] Unsafe code, if any, has a `SAFETY:` explanation.