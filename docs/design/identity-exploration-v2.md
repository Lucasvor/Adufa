<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->

# Product identity exploration — V2

Status: **concept work for bilingual testing; not an approved product identity, trademark clearance, or production asset package**  
Date: 2026-09-19  
Responds to: `docs/design/identity-review.md`  
Preserves: `docs/design/identity-exploration.md` as the V1 record

## What changed

V2 keeps only the useful core of V1: a quiet native utility, warm neutral surfaces,
one copper route signal, native UI typography, and a cut channel that shows one
chosen output. It does not continue the dual-groove or gate marks.

The identity remains deliberately subordinate to the job:

> Find an application, see its current output, and change that output in seconds.

The accepted surfaces remain the compact tray/menu popup first, expanded mini
mixer second, taskbar-adjacent selector later, and system-panel integration only
where a platform supports it cleanly. The information architecture does not
change between Windows, macOS, and Linux.

## Naming — reopened, not decided

### `Sulco` is a tested working candidate only

`Sulco Audio Router` may remain on internal boards while research continues. It
must not be used for package IDs, store listings, domains, social handles, or a
custom wordmark yet.

- Brazilian Portuguese pronunciation: `/ˈsuw.ku/`; the practical guide is
  **SOOL-koo**, not “SOOL-koh.”
- Useful meaning: a groove, furrow, incision, or channel.
- Risks: ordinary Portuguese vocabulary; medical and agricultural associations;
  unrelated exact-name uses including [Sulco on Audiomack](https://audiomack.com/sulco)
  and [SULCO Automotive Group](https://sulco-group.com/en/about-us/).
- Research label: always write **Sulco Audio Router**, never imply that `Sulco`
  has been cleared or selected.

### New candidate screen

This is a fresh discovery screen, not legal clearance. It checks obvious current
web, app, software, package, and adjacent-audio conflicts. Absence from a search
result is not evidence of registrability. Each surviving candidate still needs
formal word, phonetic, class, territory, domain, package, store, and handle
clearance by qualified counsel.

| Rank | Candidate | Concrete basis and pronunciation | Preliminary collision signal | Disposition |
| ---: | --- | --- | --- | --- |
| 1 | **Rillpath** | Coined from English *rill* — a small channel — plus *path*. EN `RILL-path`; likely PT-BR reading `RIL-péti`, which must be tested rather than prescribed. Eight letters, no diacritic, and the route metaphor is visible without saying “audio.” [Merriam-Webster](https://www.merriam-webster.com/wordplay/glossary-of-moon-terms-and-definitions/rill) supports the channel meaning. | No exact active software or audio product surfaced in the quoted-name screen. Re-run exact checks in [GitHub repositories](https://github.com/search?q=rillpath&type=repositories), [crates.io](https://crates.io/search?q=rillpath), npm, app stores, and trademark databases before adoption. Being coined increases the burden of spelling-recall testing. | **Research leader, not approved.** |
| 2 | **Calha** | A Portuguese channel that carries material from one point to another; PT-BR `KAL-yah`. [Michaelis](https://michaelis.uol.com.br/moderno-portugues/busca/portugues-brasileiro/calha) gives exactly that transport meaning. | No exact desktop audio utility surfaced. The word is common in construction, and English speakers may not infer the `lh` sound or spelling. Searchability is cleaner than meaning recall in English. | **Bilingual test only.** |
| 3 | **Rillway** | Coined channel + way; EN `RILL-way`; PT-BR `RIL-uei`. Concrete and easy to say after one exposure. | Exact music use exists as [Rillway on Spotify](https://open.spotify.com/artist/4naEeqLxfkfDF97ImeohBk), creating an adjacent audio-search conflict; it also resembles “railway.” | **Hold; meaningful collision.** |
| 4 | **Leito** | Portuguese riverbed/channel; PT-BR `LAY-too`; EN guide `LAY-too`. [Michaelis](https://michaelis.uol.com.br/moderno-portugues/busca/portugues-brasileiro/leito) records both river channel and track-bed meanings. | Exact long-running software company [Leito Ltd](https://leito.org/about-us/) ships desktop and mobile management software. Medical “bed” is the dominant Portuguese association and adds noise. | **Reject unless clearance overturns the risk.** |
| 5 | **Canalis** | Latin-derived channel; EN/PT guide `kah-NAH-lis`. Concrete and pronounceable, though spelling recall requires testing. | Exact media software exists as [IPTV Player: Canalis](https://apps.apple.com/at/app/iptv-player-canalis/id6761079381), with additional Canalis app and industrial product uses. | **Reject.** |
| 6 | **Duto** | Portuguese *duct*; PT-BR `DOO-too`; EN guide `DOO-too`. Short, concrete, and easy to spell. | [DUTO](https://docs.duto.ai/) is an active software platform with audio-generation features, a direct software/audio adjacency. | **Reject.** |

`Rillpath` leads only because this limited screen found the fewest direct conflicts.
It is not stronger than `Sulco` until English and Brazilian users can hear it
once, spell it, say it, and infer a channel or routing job. The naming test must
use all of these unaided prompts:

1. “Say this name aloud.”
2. “Type the name you just heard.”
3. “What do you think this product does?”
4. “Name it again after ten minutes.”
5. “Would you search for this name with or without ‘Audio Router’?”

## Mark V2 — the cut channel

### One idea, four optical drawings

The mark is a source channel that reaches a switch point. The chosen route is
continuous, wider, and exits the boundary. The alternative route is narrower,
shorter, and ends against a perpendicular stop. Copper reinforces selection in
the full-size version, but selection remains readable from geometry alone.

This deliberately avoids headphones, speakers, waveforms, equalizer bars,
arrows, a letter in a rounded square, and the rejected crossover and doorway
silhouettes.

Assets in this concept package:

| Asset | Intended evaluation |
| --- | --- |
| `cut-channel-master.svg` | Full-size product mark; copper is optional reinforcement. |
| `cut-channel-mono-16.svg` | Dedicated 16 px symbolic/tray optical master. |
| `cut-channel-mono-20.svg` | Dedicated 20 px optical master. |
| `cut-channel-mono-24.svg` | Dedicated 24 px optical master. |
| `cut-channel-contact-sheet.svg` | Positive, inverse, full-color, native-size, and enlarged optical comparison. |

The symbolic masters use `currentColor`, square terminals, integer-grid geometry,
and no internal lane thinner than 2 px at the target size. They are separate
drawings, not scaled copies. The selected route reaches the outside edge and is
heavier; the unselected branch ends at a visible stop. Forced-colors rendering
therefore does not depend on copper.

### Status and required platform work

These are **concept-test masters**, not store-ready assets. Before identity
approval, the project still needs:

- black-on-white, white-on-black, and OS forced-colors screenshots;
- real Windows tray, macOS menu-bar, and representative Linux panel captures at
  100% and 200% scaling;
- a Windows ICO/PNG family plus explicit monochrome high-contrast resources;
- a macOS layered app-icon composition built through the Apple asset pipeline;
- Linux full-color application assets and desktop-appropriate symbolic exports;
- reviewed sRGB exports, with Display-P3 only where the platform asset pipeline
  benefits from it;
- a separate store/app-icon composition. The symbolic status mark should not be
  dropped unchanged into a rounded store tile.

## Visual system

### Two independent preference axes

```text
ColorScheme    = system | light | dark
SurfaceMaterial = system | solid
```

There are no user themes named Glass or Acrylic. Color and material are different
decisions. `system` is the default on both axes; `solid` is the deterministic,
accessible fallback and the explicit reduced-transparency choice.

### Native material resolution

| Platform | `SurfaceMaterial = system` | Required fallback |
| --- | --- | --- |
| Windows | Use supported system Acrylic only for the transient popup or mixer surface when composition, power policy, and accessibility allow it. Never layer Acrylic. Never add custom noise over the native material. | Resolve to the matching solid surface token when transparency is disabled, unsupported, inactive, in power-saving mode, or fails contrast. |
| macOS | Use the semantic AppKit popover/menu material appropriate to the native surface, not a copied Windows tint recipe. Respect Reduce Transparency. | Replace the effect view with the matching opaque semantic surface and separator tokens. |
| Linux | Use only material/transparency behavior exposed by the selected native toolkit and desktop/compositor. Do not ship a private blur protocol or simulated Acrylic. | Solid is expected on many desktops and is a first-class result, not a degraded theme. |

The platform shell owns material resolution. Shared code owns semantic color,
spacing, hierarchy, and state meaning. No fixed blur radius, tint percentage, or
noise texture is part of the cross-platform identity contract.

### Shared semantic source tokens

OKLCH is the editable design source. Platform assets are exported to reviewed
sRGB/P3 values by the native asset pipeline; production code must not assume that
every SVG or system icon consumer accepts OKLCH.

#### Light scheme

| Token | OKLCH source | sRGB check value | Role |
| --- | --- | --- | --- |
| `surface` | `oklch(97% 0.010 80)` | `#F9F4EE` | Opaque window/popup fallback. |
| `surface-raised` | `oklch(99% 0.004 80)` | `#FDFBF9` | Selected row or nested menu where native semantics require elevation. |
| `ink` | `oklch(22% 0.015 55)` | `#201914` | Primary text and symbolic icons. |
| `ink-muted` | `oklch(45% 0.025 55)` | `#615248` | Secondary text. |
| `route` | `oklch(52% 0.160 43)` | `#B03F00` | Selected route and primary route action. |
| `route-hover` | `oklch(47% 0.150 43)` | `#9B3400` | Hover/pressed fill where appropriate. |
| `route-ink` | `oklch(98% 0.005 80)` | `#FAF8F5` | Normal-size text or icon on `route` and `route-hover`. |
| `focus-outer` | `oklch(45% 0.160 255)` | `#0052AB` | External keyboard-focus ring against the surrounding surface. |
| `focus-separator` | `oklch(99.5% 0 0)` | `#FDFDFD` | Inner separator ring between focus blue and copper/control fill. |

#### Dark scheme

| Token | OKLCH source | sRGB check value | Role |
| --- | --- | --- | --- |
| `surface` | `oklch(18% 0.012 55)` | `#16100D` | Opaque window/popup fallback. |
| `surface-raised` | `oklch(22% 0.014 55)` | `#201914` | Selected row or nested menu. |
| `ink` | `oklch(95% 0.008 80)` | `#F1EEE9` | Primary text and symbolic icons. |
| `ink-muted` | `oklch(72% 0.020 70)` | `#ADA397` | Secondary text. |
| `route` | `oklch(72% 0.140 48)` | `#EA8751` | Selected route and primary route action. |
| `route-hover` | `oklch(78% 0.130 50)` | `#FA9D68` | Hover/pressed fill where appropriate. |
| `route-ink` | `oklch(18% 0.012 55)` | `#16100D` | Normal-size text or icon on `route` and `route-hover`. |
| `focus-outer` | `oklch(82% 0.120 245)` | `#7ECCFF` | External keyboard-focus ring against the surrounding surface. |
| `focus-separator` | `oklch(13% 0.010 55)` | `#0A0604` | Inner separator ring between focus blue and copper/control fill. |

`focus-outer` is intentionally not a brand orange. Focus is an input modality
signal, not a brand flourish. On filled controls, draw an immediate two-color
ring: 1 px `focus-separator` adjacent to the control, then 2 px `focus-outer`
outside it. Do not animate focus appearance.

### Machine-checked contrast matrix

The ratios below were produced by a script that converts the declared OKLCH
values to gamut-clipped sRGB, calculates WCAG 2 relative luminance, and applies
`(L1 + 0.05) / (L2 + 0.05)`. Values are rounded to two decimals. Re-run the
calculation whenever a source token changes; do not copy the rounded sRGB values
back into the design source.

| Scheme | Foreground / adjacent color | Background / adjacent color | Ratio | Contract |
| --- | --- | --- | ---: | --- |
| Light | `ink` | `surface` | 15.91:1 | Pass normal text. |
| Light | `ink-muted` | `surface` | 6.87:1 | Pass normal text. |
| Light | `route` | `surface` | 5.41:1 | Pass non-text and large display; not used as small text. |
| Light | `route-ink` | `route` | 5.57:1 | Pass normal text. |
| Light | `route-ink` | `route-hover` | 6.87:1 | Pass normal text. |
| Light | `focus-outer` | `surface` | 6.88:1 | Pass focus perimeter. |
| Light | `focus-separator` | `route` | 5.82:1 | Pass inner focus separation. |
| Light | `focus-separator` | `route-hover` | 7.18:1 | Pass inner focus separation. |
| Light | `focus-outer` | `focus-separator` | 7.40:1 | Pass two-color ring boundary. |
| Dark | `ink` | `surface` | 16.27:1 | Pass normal text. |
| Dark | `ink-muted` | `surface` | 7.57:1 | Pass normal text. |
| Dark | `route` | `surface` | 7.25:1 | Pass non-text and large display; not used as small text. |
| Dark | `route-ink` | `route` | 7.25:1 | Pass normal text. |
| Dark | `route-ink` | `route-hover` | 9.06:1 | Pass normal text. |
| Dark | `focus-outer` | `surface` | 10.74:1 | Pass focus perimeter. |
| Dark | `focus-separator` | `route` | 7.75:1 | Pass inner focus separation. |
| Dark | `focus-separator` | `route-hover` | 9.68:1 | Pass inner focus separation. |
| Dark | `focus-outer` | `focus-separator` | 11.48:1 | Pass two-color ring boundary. |

Material surfaces must be contrast-tested from their composited pixels in each
real platform capture. The solid matrix is the guaranteed fallback, not evidence
that an arbitrary translucent background passes.

## Typography and memory budget

Application UI uses the native system UI family only:

- Windows: Segoe UI Variable with Segoe UI fallback;
- macOS: San Francisco through system APIs;
- Linux: the toolkit/desktop UI font, with its normal fallback chain.

No application font files are bundled. This preserves native metrics, scripts,
localization behavior, startup time, and the low-memory target. IBM Plex Sans and
IBM Plex Mono may remain documentation/release typography, but they are not a
distinctive wordmark and are not shipped merely to make the app look branded.
A custom wordmark is deferred until the name clears.

## Interaction contract

Universal states are only those every applicable native control can actually
enter:

- default;
- hover where a pointing device/platform exposes hover;
- keyboard focus;
- pressed/active;
- disabled.

Busy, error, and success are defined only for commands or fields that can enter
those states. A routing row does not receive decorative loading/error/success
variants simply to complete a matrix. Route success is normally silent: the
selected output label and check state update together; failures retain the
previous route and explain what could not change.

Search filters the local session/device list synchronously on every keystroke.
No debounce is specified. A measured expensive provider may add a short delay
later, with profiling evidence and an explicit cancellation contract.

Motion remains functional and small: route confirmation may crossfade for
120 ms; a popup may use the native open/close transition; reduced-motion removes
spatial movement. The focus ring appears immediately.

## Accessibility and localization

- Copper never carries state alone; geometry, labels, and the selected check do.
- The one-color symbolic mark must be used for forced-colors and status surfaces.
- Route labels retain at least 4.5:1 text contrast; focus and selected geometry
  retain at least 3:1 against adjacent colors.
- Reduced transparency resolves `SurfaceMaterial` to solid without changing
  information hierarchy.
- Rows preserve a platform-appropriate minimum target size and never hide the
  current output behind hover.
- Layout mirrors for right-to-left locales; the routing metaphor mirrors with the
  reading direction only after native usability testing.
- Product code, identifiers, comments, and source documentation remain English.
  English is the first-run language; users can select localized UI languages as
  translations are added.

## Audit-resolution ledger

| Review item | V2 response |
| --- | --- |
| Focus and route-text contrast | Separate blue focus tokens, inner separator tokens, tested route text, and published matrix. |
| `Sulco` overclaim and pronunciation | Downgraded to working candidate; corrected to `/ˈsuw.ku/` / SOOL-koo; formal and user tests required. |
| Preferred icon failing below 32 px | Dedicated one-color 16/20/24 optical drawings; selection encoded by weight, exit, and stop geometry. |
| Dual-groove ambiguity | Branch retired; no V2 asset. |
| Gate/export ambiguity | Branch retired; no V2 asset. |
| Missing icon package | Symbolic optical concept set and contact sheet added; platform/store deliverables explicitly remain pending. |
| Appearance-axis multiplication | Replaced with `ColorScheme` and `SurfaceMaterial`; platform-native resolution and solid fallback specified. |
| “Implementation-ready” claim | Removed. V2 is concept work with named approval evidence still missing. |
| Generic wordmark typography | Native UI type retained; Plex limited to docs; custom wordmark deferred until name clearance. |
| Universal eight-state rule | Replaced with five universal applicable states plus contextual busy/error/success. |
| Speculative search debounce | Removed; synchronous filtering is the default until profiling proves otherwise. |

## Hallmark pre-emit critique

| Axis | Score | Evidence |
| --- | ---: | --- |
| Philosophy | 5 | Every identity decision begins with fast, quiet per-application routing and native resource restraint. |
| Hierarchy | 5 | Product job, naming status, icon status, preference axes, accessibility contract, and remaining evidence are clearly separated. |
| Execution | 4 | Optical SVGs and scripted contrast records exist; real OS captures and platform asset catalogs intentionally remain future validation. |
| Specificity | 5 | The stopped alternate channel, platform material resolution, synchronous local search, and bilingual name tests are specific to this utility. |
| Restraint | 5 | One mark idea, one brand accent, no decorative glass, no bundled UI fonts, and no universal state theater. |
| Variety | 4 | The system supports scheme, material, forced-colors, and platform context without fragmenting into unrelated visual themes. |

No axis is below 3. The honest handoff remains: **better tested concept, still not
an approved name, final icon, or production asset package.**
