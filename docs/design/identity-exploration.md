<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->

# Product identity exploration

Status: exploratory, not a naming or trademark decision  
Date: 2026-09-19

## Design position

This product is a quiet piece of desktop infrastructure. It should feel precise,
fast, trustworthy, and native—not like a music player, streaming service, or
miniature DAW.

The visual idea is a **cut channel**: one signal enters a deliberate groove and
leaves through the output the user selected. The groove is a physical metaphor
for routing, but it avoids the usual headphones, speakers, waveforms, equalizer
bars, neon gradients, and letter-in-a-rounded-square marks.

Audience: mainstream and power users who need to redirect an application's audio
without opening system settings.  
Central job: find an application and change its output in seconds.  
Tone: discreet technical utility; refined, trustworthy, distinctive, and calm.

The visual system must preserve the interaction surfaces accepted in
`docs/adr/0005-interaction-surfaces-and-rollout.md`:

1. Compact tray or menu-bar popup as the primary surface.
2. Expanded mini mixer for per-application output and volume.
3. Taskbar-adjacent selector after the core is stable.
4. System volume-panel integration as an optional platform module.

The native implementations may differ in platform mechanics, but application
rows, output labels, route status, search, and the selected-output treatment keep
the same hierarchy and recognizable identity.

## Naming exploration

### Provisional recommendation: Sulco

**Sulco** means a groove or furrow in Portuguese: a path cut into a surface. It
connects naturally to a signal path, a vinyl groove, and the product's promise of
putting audio into the intended channel. It is short, concrete, and not assembled
from startup-name fragments.

Pronunciation:

- Brazilian Portuguese: `SOOL-koh`.
- English guidance: `SOOL-koh`; the spelling is learnable after one exposure.

Working descriptor: **Sulco — application audio routing**. The descriptor should
appear in store listings and first-run material until the name is established.

### Ranked candidates

| Rank | Candidate | Rationale | Preliminary collision signal |
| ---: | --- | --- | --- |
| 1 | **Sulco** | A carved route; subtle audio association through “groove”; strong icon territory. | No direct desktop audio utility surfaced in the preliminary queries. The word and company name are used in unrelated fields, so it still needs formal clearance. |
| 2 | **Desvio** | A deliberate reroute; memorable in Portuguese and pronounceable after guidance. | No direct audio utility surfaced, but the Portuguese word can also imply deviation, detour, or misappropriation. That semantic risk prevents it from ranking first. |
| 3 | **Runnel** | A small channel that carries a stream; the metaphor is accurate and quiet. | An active app already uses **Runnel** on the [Apple App Store](https://apps.apple.com/us/app/runnel/id6758957231). High store-confusion risk. |
| 4 | **Rivus** | Latin-root channel or stream; compact and professional. | Multiple active apps use the name, including [Rivus: Fly Fishing](https://apps.apple.com/us/app/rivus-fly-fishing/id6763570083). High store-confusion risk. |
| 5 | **Trilho** | Track or path in Portuguese; direct routing metaphor and good visual language. | An active software product already operates as [Trilho AI](https://www.trilho.ai/auth/signup). Medium-to-high collision risk. |
| 6 | **Viarum** | “Of roads” in Latin; systematic and international in tone. | An established technology company uses [VIARUM](https://www.viarum.mx/index.html), and a current registration also surfaced in the preliminary search. High legal-review risk. |
| 7 | **Patchfield** | An audio patching field; immediately understood by technical users. | Disqualified: Google's [Patchfield](https://github.com/google/patchfield) is specifically an audio-routing infrastructure project. |

Other obvious terms—Audio Router, Pulse, Nexus, Flow, Switchboard, Patchbay,
Relay, and “-ly” switch names—were intentionally excluded because they are
generic, crowded, or carry the exact startup clichés this identity should avoid.

### Name-check boundary

This is a preliminary web and app-ecosystem screen, performed on 2026-09-19. It
is **not trademark clearance** and does not establish availability in any Nice
class, country, app store, package registry, domain registry, or social handle.
Before public commitment:

1. Search INPI Brazil, USPTO, EUIPO, WIPO Global Brand Database, and the relevant
   registries for launch countries, including phonetic and similar marks.
2. Search Microsoft Store, Mac App Store, Flathub, Snap Store, Homebrew, winget,
   GitHub, crates.io, and common package namespaces.
3. Check domains and social handles without buying them through a public bulk
   lookup that could expose intent.
4. Ask qualified trademark counsel to assess software and audio-related classes.

Until then, use `Sulco` only as a provisional design label and keep package IDs
neutral.

## Icon system

### Concept

The mark is an **engraved switch path**. A broad dark cut establishes the durable
channel; a narrow copper lane marks the effective route. It is neither an audio
level display nor a network-node diagram. The asymmetry is functional: the mark
must communicate that one source can be sent to a chosen destination.

Three hand-authored explorations are included:

- [`icons/sulco-switch-cut.svg`](icons/sulco-switch-cut.svg) — preferred. One
  source, two possible grooves, one selected output. Strongest at tray size.
- [`icons/sulco-dual-groove.svg`](icons/sulco-dual-groove.svg) — more compact and
  architectural. Better as a monochrome system-tray glyph.
- [`icons/sulco-gate.svg`](icons/sulco-gate.svg) — heavier and more emblematic.
  Better for store artwork, but less immediately directional.

### Icon rules

- Build on a 1024 × 1024 master grid with a 128-unit optical safe area.
- Keep the silhouette readable at 16 px. Remove inner detail before shrinking
  stroke widths below one physical pixel.
- Use rounded terminals only where a routed channel physically ends. Do not put
  circles on every junction.
- The accent marks **the selected route**, never decoration.
- Provide a one-colour template for Windows tray, macOS menu bar, Linux panels,
  high contrast, and forced-colour modes.
- Let each operating system supply its own app-icon container, mask, corner
  treatment, and shadow. Do not bake a rounded-square tile into the core glyph.
- Never animate the installed application icon. Inside the UI, a route change may
  crossfade the accent lane once; it must not pulse or loop.

### Recommended production path

Start from `sulco-switch-cut.svg`. Test it at 16, 20, 24, 32, 48, 128, 256, and
1024 px on light and dark backgrounds. Then redraw—not merely auto-trace—the
surviving geometry into platform assets. Optical corrections at 16–24 px are
expected and should live in dedicated small-size masters.

## Color identity

The anchor is **cut copper**, a warm orange associated with physical controls and
conductive material rather than entertainment neon. The surrounding neutrals are
warm graphite and mineral paper. Accent coverage stays below roughly 5% of a
surface; it identifies the current route, keyboard focus, or a single primary
action.

All named palette values are opaque OKLCH colors. Transparency, blur, and noise
are material modifiers, not colors.

### Shared semantic colors

| Token | Value | Role |
| --- | --- | --- |
| `color-route` | `oklch(58% 0.16 45)` | Selected route on light or solid surfaces |
| `color-route-dark` | `oklch(72% 0.14 48)` | Selected route on dark/translucent surfaces |
| `color-route-ink` | `oklch(18% 0.012 55)` | Text or glyph over a route-colored fill |
| `color-focus` | `oklch(64% 0.20 45)` | Immediate 2–3 px keyboard focus ring |
| `color-error` | `oklch(56% 0.20 25)` | Failure plus icon and explanatory text |
| `color-success` | `oklch(52% 0.12 150)` | Confirmed availability plus icon/text |

Success and error colors never carry meaning alone. A route uses the copper
accent, a check or warning glyph, and a textual status where ambiguity matters.

## Appearance modes

All four appearances keep the same cut-copper route signal, information
hierarchy, spacing, iconography, and component geometry. They change surface
material, not product identity.

### Light

| Token | Value |
| --- | --- |
| `surface-canvas` | `oklch(97% 0.008 80)` |
| `surface-panel` | `oklch(94% 0.012 80)` |
| `surface-raised` | `oklch(99% 0.006 80)` |
| `text-primary` | `oklch(20% 0.012 55)` |
| `text-secondary` | `oklch(43% 0.012 55)` |
| `border-default` | `oklch(80% 0.018 70)` |
| `route-active` | `oklch(58% 0.16 45)` |
| `route-active-hover` | `oklch(53% 0.17 45)` |

Use mineral paper rather than pure white. Elevation comes from a small lightness
step and one hairline, not stacked shadows.

### Dark

| Token | Value |
| --- | --- |
| `surface-canvas` | `oklch(14% 0.012 55)` |
| `surface-panel` | `oklch(18% 0.014 55)` |
| `surface-raised` | `oklch(22% 0.014 55)` |
| `text-primary` | `oklch(94% 0.008 80)` |
| `text-secondary` | `oklch(72% 0.010 70)` |
| `border-default` | `oklch(30% 0.015 60)` |
| `route-active` | `oklch(72% 0.14 48)` |
| `route-active-hover` | `oklch(77% 0.13 48)` |

Raised surfaces become lighter, never shadow-glowing. Light text uses the native
platform's regular or optical-light adjustment rather than a heavy weight.

### Glass

Glass is a restrained, adaptive desktop material—not decorative glassmorphism.
It is permitted only on the outer popup/mixer surface where the desktop context
is genuinely behind it.

| Token | Value |
| --- | --- |
| `glass-tint-light` | `oklch(98% 0.008 80)` |
| `glass-tint-dark` | `oklch(18% 0.012 55)` |
| `glass-border-light` | `oklch(82% 0.014 75)` |
| `glass-border-dark` | `oklch(36% 0.014 60)` |
| `glass-route` | `oklch(72% 0.14 48)` |

Material modifiers: tint opacity 72–82%, backdrop blur 14–20 px, one 1 px edge,
and no decorative gradient. Application rows receive a solid or near-solid local
scrim so text contrast does not depend on the wallpaper. When blur, composition,
power mode, or accessibility settings make the material unreliable, fall back to
the corresponding Light or Dark solid tokens.

### Acrylic

Acrylic is denser than Glass and carries a fine material grain. On Windows, use
the supported native transient-window material. On macOS and Linux, reproduce the
**density and hierarchy**, not Microsoft's private implementation details.

| Token | Value |
| --- | --- |
| `acrylic-tint-light` | `oklch(94% 0.014 75)` |
| `acrylic-tint-dark` | `oklch(20% 0.016 55)` |
| `acrylic-scrim-light` | `oklch(99% 0.006 80)` |
| `acrylic-scrim-dark` | `oklch(24% 0.014 55)` |
| `acrylic-route` | `oklch(70% 0.14 48)` |

Material modifiers: tint opacity 84–90%, backdrop blur 20–28 px, monochrome noise
at 1–2% opacity, and a single edge highlight. Noise is static. Nested acrylic
cards, glowing edges, colored shadows, and translucent text fields are forbidden.
The solid fallback is mandatory and must preserve the same selected-route color.

## Typography

The installed application uses native system typography to minimize memory,
startup cost, and visual friction:

- Windows: Segoe UI Variable where available, then Segoe UI.
- macOS: San Francisco through the system font APIs.
- Linux: the desktop environment's configured UI sans, with a metrics-safe
  fallback chosen by the native toolkit layer.

Product hierarchy—not a bundled display font—carries the brand inside the app.
The icon, copper route signal, spacing, and row composition provide recognition.

For the GitHub page, release artwork, and wordmark construction, use **IBM Plex
Sans** with **IBM Plex Mono** only for version/build notation. Both are open-source
and have a technical but human voice. If the wordmark is finalized, distribute it
as reviewed vector outlines rather than bundling a font solely for four letters.

Type rules:

- Base UI text: platform default, typically 13–15 px according to platform norms.
- Application name: semibold; output and status: regular.
- Numeric volume: tabular figures.
- No all-caps paragraphs, italic headings, gradient text, or low-contrast metadata.
- Truncate long application/device names only after preserving the unique portion;
  expose the full value to accessibility APIs and a keyboard-accessible tooltip.

## Spacing and geometry

Use a 4 px base with semantic steps: 4, 8, 12, 16, 20, 24, and 32 px. The compact
popup should feel dense but never cramped.

- Popup edge: 12 px compact / 16 px comfortable.
- Application row: 48 px minimum visual height; 44 px minimum hit target.
- Icon-to-label gap: 12 px.
- Primary-to-secondary label gap: 2–4 px.
- Section gap: 16–24 px, chosen by content—not repeated mechanically.
- Corners: 4 px controls, 8 px menus/popovers, 12 px outer floating surface.
- Pills are reserved for short statuses and segmented choices; they are not the
  default shape for every button.
- One containment layer. Never put cards inside a bordered outer card.

## Component voice

### Application row

The row leads with the application icon and name. The current output sits beneath
or on the trailing edge depending on available width. The selected row uses a
quiet surface change; the **route itself** uses copper. Expanding an application
reveals sessions/processes as indented informative rows in V1, matching the
accepted routing semantics.

### Output picker

Device names are the primary labels. Availability is explicit. The desired but
unavailable output remains visible with `Unavailable`; the effective fallback is
shown as `Using system default`. Never hide the distinction.

### Volume

The native slider keeps the platform's input behavior and accessibility mapping.
The thumb receives the focus state, not the entire track. Volume values use
tabular numbers. The route color must not turn the whole slider into an equalizer
visual.

### Search and commands

Search opens ready for typing. Results update after a short debounce and announce
their count through platform accessibility APIs. Labels use specific verbs:
`Route to`, `Use system default`, `Open mixer`, and `Export diagnostics`—never
`OK`, `Submit`, or `Continue` without context.

### States

Every interactive control defines default, hover, focus, pressed, disabled,
loading, error, and success. Focus is immediate, 2–3 px, and at least 3:1 against
both the control and surrounding surface. Hover never owns functionality. Errors
state what failed and what the user can do.

## Motion

Motion is almost absent. The product should feel faster because it does less.

- Popup open: platform-native transition, or 180–220 ms opacity plus at most 4 px
  translation when the platform provides no suitable transition.
- Output selection: 120 ms crossfade of the route marker and label; no toast when
  the changed output is already visible.
- Expanded sessions: 180–220 ms opacity/transform reveal; no animated height.
- Button press: 80–120 ms, 1 px physical depression.
- Error rollback: 200 ms color restoration plus concise retry guidance.
- Focus rings appear instantly and never animate.
- Only transform and opacity animate. No bounce, parallax, cursor follower,
  looping ambient motion, or celebratory success animation.
- Reduced-motion mode makes all spatial changes instantaneous or uses an opacity
  crossfade of at most 150 ms.

## Accessibility and platform adaptation

- Full keyboard navigation and visible focus are release requirements.
- Minimum touch target: 44 × 44 CSS-equivalent pixels where touch is possible.
- Respect high contrast, forced colors, reduced transparency, reduced motion,
  increased contrast, and platform text scaling.
- Never rely on blur for separation or color alone for status.
- Application and device icons supplement text; they never replace accessible
  names.
- Keep route failure, desired output, and effective output distinct in both text
  and accessibility descriptions.
- Use platform-provided menus, semantics, and input behavior where they improve
  accessibility. Visual sameness never outranks native behavior.

## Anti-slop prohibitions

- No headphones, speakers, waveforms, equalizer bars, music notes, neon glows, or
  purple-to-cyan gradients in the brand mark.
- No rounded-square monogram as the core identity.
- No decorative glass cards. Translucency only communicates the real desktop
  layer behind a transient surface.
- No three-column icon-card vocabulary in project pages.
- No startup superlatives or invented performance claims.
- No mixed icon libraries. Platform symbols may be used for conventional actions;
  product-specific actions use one reviewed custom set.
- No animation whose removal would lose no information.

## Hallmark pre-emit critique

| Axis | Score | Review |
| --- | ---: | --- |
| Philosophy | 5 | Identity comes from the product's routing behavior and low-footprint constraints, not a fashionable skin. |
| Hierarchy | 5 | Primary popup, mini mixer, selected route, and fallback states retain an explicit order. |
| Execution | 4 | Tokens and SVG masters are implementation-ready explorations; small-size optical masters and formal contrast tests remain production work. |
| Specificity | 5 | The cut-channel metaphor, candidate checks, route semantics, and platform material rules are product-specific. |
| Restraint | 5 | One accent, native type, limited motion, one translucent layer, and no decorative gradients. |
| Variety | 4 | Three structurally different marks and four materially distinct appearances share one recognizable system without becoming palette swaps. |

No axis is below 3. The next review should test the three glyphs at tray size with
real application rows before promoting any mark from exploration to identity.
