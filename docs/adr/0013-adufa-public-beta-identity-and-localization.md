# Adufa public beta identity and localization

- Status: Accepted
- Date: 2026-09-19

## Context

The prototype documentation still described a Windows-only benchmark, while the
product direction targets native Windows, macOS, and Linux implementations. The
public beta also needs a stable name, a portable visual metaphor, searchable
documentation, and a language policy that does not make English a prerequisite
for a small desktop utility.

## Decision

- Use **Adufa** as the public beta product name.
- Use the English tagline “Route every app to the right audio device” and the
  Brazilian Portuguese tagline “Direcione cada aplicativo para o dispositivo de
  áudio certo.”
- Base the visual mark on one sound channel reaching a gate and being redirected
  toward a selected output. The mark uses simple geometry, soft corners, graphite
  neutrals, and one electric-blue route accent. It contains no letters, operating-
  system marks, headphones, or music notes.
- Keep the same core mark across platforms while exporting and testing native
  asset variants for each platform's packaging and small-size requirements.
- Ship interface and documentation translations for `en`, `pt-BR`, `es`, `fr`,
  `de`, `it`, `ja`, and `zh-CN`.
- Follow the operating-system display language by default, allow a persisted
  manual override, and fall back to English for unsupported locales or missing
  strings.
- Keep the root README in English, place a complete Brazilian Portuguese README
  beside it, and link complete translations for the other supported languages.
- Describe Windows as the available beta, with macOS and Linux as planned
  platforms. Do not imply that planned backends already exist.

## Consequences

- Product copy, settings, release artifacts, and documentation use one public name.
- New interface strings must be added to all locale tables in the same change;
  English remains the executable fallback.
- Translations require native-speaker review before being considered final.
- The icon must be checked at 16, 24, 32, and large application-icon sizes instead
  of being judged only as README artwork.
- Public releases still require formal trademark review; the preliminary naming
  screen is not legal clearance.
