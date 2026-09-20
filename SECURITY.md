# Security policy

## Supported versions

The current `main` branch and the newest tagged beta release are supported.
Older portable builds are not patched automatically.

## Reporting a vulnerability

Do not open a public issue for a vulnerability involving privilege boundaries,
malicious audio metadata, code execution, data exposure, update integrity, or
unsafe interaction with Windows components.

Use GitHub's private vulnerability reporting feature for this repository. If it
is not enabled yet, contact the maintainers privately through the account that
publishes the official release and include a minimal reproduction, affected
version, impact, and any mitigation you know.

Expect an acknowledgement within seven days. The project will coordinate a fix
and release before publishing technical details where reasonably possible.

## Scope

The Windows beta uses native Windows audio and shell integration. Reports about
unsupported Explorer injection techniques are out of scope: Adufa does not load
code into Explorer, ShellExperienceHost, or other Windows processes.
