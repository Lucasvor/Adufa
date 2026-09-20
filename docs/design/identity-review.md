<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->

# Independent identity review

Status: preliminary design audit, not trademark clearance  
Date: 2026-09-19

## Executive verdict

**Rework before approval.** The underlying direction is good: a quiet native
utility, one cut-copper route signal, native typography, restrained motion, and
translucency only where it represents a real desktop layer. That is specific to
the product and avoids the usual audio-app clichés.

The proposal is not yet a production identity. `Sulco` is the best candidate in
the current list but is not cleared, is weak in English search and word-of-mouth,
and has unrelated exact-name uses. None of the three SVGs is ready for tray,
menu-bar, store, or high-contrast delivery. The color system also contains a
measurable focus-contrast contradiction.

| Item | Verdict | Short reason |
| --- | --- | --- |
| `Sulco` | **REWORK** | Keep on the shortlist; do not commit package/store IDs yet. |
| `Desvio` | **REJECT** | Exact App Store collision and negative Portuguese meanings. |
| `Runnel` | **REJECT** | Exact consumer app, developer package, and software-service collisions. |
| `Rivus` | **REJECT** | Exact active app and software namespace collisions. |
| `Trilho` | **REJECT** | Active Brazilian software product uses the name. |
| `Viarum` | **REJECT** | Established technology-company use and confusing near-match `Viarium`. |
| `Patchfield` | **REJECT** | Exact prior audio-routing project. |
| `sulco-switch-cut.svg` | **REWORK** | Best concept, but generic in silhouette and subpixel at small sizes. |
| `sulco-dual-groove.svg` | **REJECT** | Reads as crossover, shuffle, or an `X`, not source-to-output routing. |
| `sulco-gate.svg` | **REJECT** | Reads as exit/export/login before it reads as audio routing. |
| Visual system | **REWORK** | Strong direction; contrast, material semantics, and platform assets are incomplete. |

## Ranked findings

### Critical

#### 1. The focus contract fails on the brand's own active controls

**Tell:** inaccessible state token  
**Where:** `identity-exploration.md:138-147`, `identity-exploration.md:301-306`

The document promises a focus indicator with at least 3:1 contrast against both
the control and its surroundings, but `color-focus` and every route-active color
occupy nearly the same orange lightness/hue. Converting the declared OKLCH values
to sRGB and applying the WCAG contrast formula gives approximately:

- `color-focus` against `color-route`: **1.26:1**;
- `color-focus` against `color-route-dark`: **1.40:1**;
- `color-focus` against the light active-hover color: **1.56:1**.

Also, `color-route-ink` against `color-route` is about **4.12:1**, below the
4.5:1 threshold if the token is used for normal-size text as its documented role
permits. Thin anti-aliased graphics can perform worse than their nominal ratio,
which the W3C explicitly warns about in its [non-text contrast guidance](https://www.w3.org/WAI/WCAG22/understanding/non-text-contrast.html).

**Required fix:** separate brand accent from keyboard focus. Define tested
light/dark `focus-outer` and `focus-separator` tokens, use a two-color focus ring
on copper-filled controls, and limit `route-ink` to sufficiently large/bold text
or replace it with a tested text color. Publish the contrast matrix before
implementation.

### Major

#### 2. `Sulco` is a promising metaphor, not an approved global product name

**Tell:** premature distinctiveness claim  
**Where:** `identity-exploration.md:38-51`, `identity-exploration.md:69-85`

No exact desktop audio router named `Sulco` surfaced in this preliminary screen,
which is positive. However, exact-name uses include a current music artist called
[Sulco on Audiomack](https://audiomack.com/sulco) and the established
[SULCO Automotive Group](https://sulco-group.com/en/about-us/), whose services
include software engineering. The word is also ordinary Portuguese vocabulary
covering furrows, grooves, wrinkles, incisions, and anatomical depressions
([Michaelis](https://michaelis.uol.com.br/moderno-portugues/busca/portugues-brasileiro/sulco),
[Academia das Ciências de Lisboa](https://dicionario.acad-ciencias.pt/pesquisa/sulco/)).
That makes exact-word search results noisy and gives the name medical/agricultural
associations alongside the useful vinyl-groove meaning.

English speakers have no established pronunciation cue and may say “SULL-co.”
Brazilian Portuguese is approximately `/ˈsuw.ku/`, so the current `SOOL-koh`
guide is not a good general Brazilian rendering; `SOOL-koo` is closer
([pronunciation reference](https://en.wiktionary.org/wiki/sulco)).

**Required fix:** retain `Sulco` only as a working candidate, always paired with
`Audio Router` in research. Run formal phonetic trademark clearance and store,
domain, package, and handle checks. Test unaided pronunciation, spelling recall,
and the prompt “What do you think this app does?” with English and Brazilian
Portuguese users. If the descriptor must remain permanently for discovery, the
name is not carrying enough of the job alone.

#### 3. The preferred switch-cut loses its defining lane below 32 px

**Tell:** fine-detail master presented as small-size mark  
**Where:** `icons/sulco-switch-cut.svg:6-9`, `identity-exploration.md:98-99`

The inner 34-unit lanes become 0.53 px at 16 px, 0.66 px at 20 px, and 0.80 px at
24 px. Their copper-versus-light distinction will blur or disappear depending on
rasterizer and display scale. In monochrome, the selected-route idea vanishes and
the remaining silhouette is a familiar branching/Y-routing symbol rather than a
distinctive mark.

**Required fix:** keep this concept but redraw dedicated 16, 20, and 24 px masters
on their final pixel grids. The selected route must survive by geometry, not only
color. Test black-on-white, white-on-black, forced colors, and actual Windows,
macOS, and Linux panel backgrounds before calling it preferred.

#### 4. The dual-groove mark communicates the wrong operation

**Tell:** ambiguous icon metaphor  
**Where:** `icons/sulco-dual-groove.svg:5-9`

Two equal crossing paths create an `X`, crossover, swap, or shuffle symbol. There
is no clear source or chosen destination. Its 30-unit lanes are even thinner:
0.47 px at 16 px and 0.70 px at 24 px. The crossing order also makes one lane
visually overwrite the other at the center.

**Required fix:** reject this branch rather than polishing it.

#### 5. The gate mark collides with exit, export, and sign-in iconography

**Tell:** familiar symbol with a conflicting conventional meaning  
**Where:** `icons/sulco-gate.svg:5-7`

The open-sided enclosure is read as a door or boundary, and the path leaving it
resembles common sign-out/export semantics. It is the most robust of the three at
small size, but recognizability of the wrong action is not an advantage.

**Required fix:** reject it as the primary identity. Do not use it for an action
inside the app either, where the exit/export reading would become stronger.

#### 6. The icon package described by the document does not exist yet

**Tell:** master-only asset presented as a system  
**Where:** `identity-exploration.md:105-125`, all three SVG files

There is no one-color symbolic master, no small-size optical master, no app/store
composition, no light/dark export, and no platform manifest. Raw `oklch(...)`
paint values are useful design-source data but should not be the interchange
format for Windows ICO/PNG assets, Apple asset catalogs, or Linux symbolic icons.
Apple expects platform-shaped, layered app-icon inputs and warns that fine detail
fails at small sizes ([Apple HIG](https://developer.apple.com/design/human-interface-guidelines/app-icons?country=63)).
GNOME's symbolic guidance uses a 16 px grid, recommends 2 px main strokes, and
avoids 1 px strokes where possible ([GNOME HIG](https://developer.gnome.org/hig/guidelines/ui-icons.html)).
Windows likewise recommends separate theme-sensitive assets and a direct
black-and-white high-contrast representation
([Microsoft guidance](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-design)).

**Required fix:** keep OKLCH in the design-token source, then generate reviewed
sRGB/P3 platform assets. Create separate product-mark, symbolic-status, Windows
app, macOS layered app, and Linux full-color/symbolic deliverables.

#### 7. Light/Dark and Glass/Acrylic are modeled as peers when they are different axes

**Tell:** material-as-theme multiplication  
**Where:** `identity-exploration.md:152-227`

Light and Dark are color schemes. Glass and Acrylic are materials whose appearance
and availability are controlled by the OS. Treating all four as equivalent user
themes creates a large test matrix and encourages a Windows material imitation on
macOS and Linux. It also conflicts with the architecture's correct statement that
platform effects resolve inside each native shell (`architecture.md:340-345`).

Microsoft recommends Acrylic for transient surfaces, warns against layering it,
and notes its GPU/power cost and automatic disabling
([Microsoft Acrylic guidance](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic)).
Apple similarly says to select materials by semantic purpose rather than apparent
color and to respect reduced transparency
([Apple materials](https://developer.apple.com/design/human-interface-guidelines/materials),
[reduced-transparency API](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducetransparency?changes=_8)).

**Required fix:** define two axes: `ColorScheme = system/light/dark` and
`SurfaceMaterial = system/solid`. Let Windows resolve `system` to supported
Acrylic where appropriate, macOS to the semantic popover material, and Linux to
the desktop's supported surface. Fixed blur, tint, and noise values become fallback
recipes only. Do not add custom noise on top of native Acrylic.

#### 8. “Implementation-ready” is not supported by test artifacts

**Tell:** unverified completion language  
**Where:** `identity-exploration.md:352-364`

The document itself says small-size masters and formal contrast testing remain,
yet scores execution as 4/5 and calls the masters implementation-ready. There are
no contact sheets, real-row mockups, contrast records, or platform captures.

**Required fix:** downgrade the identity to concept status until the required
artifact checklist below exists. Claims should follow evidence.

### Minor

#### 9. The wordmark typography is professional but not distinctive

**Tell:** generic developer-tool typography  
**Where:** `identity-exploration.md:242-245`

IBM Plex is credible and open, but common in open-source and technical products.
Simply outlining `Sulco` in Plex will not create a unique wordmark.

**Required fix:** use Plex as documentation/release typography, not as proof of
brand distinctiveness. Defer a custom wordmark until the name clears.

#### 10. Eight states on every control over-specifies native UI

**Tell:** state-matrix boilerplate  
**Where:** `identity-exploration.md:301-306`

Error, loading, and success are not meaningful visual states for every button or
row. Enforcing them universally adds code and screenshots without improving the
workflow.

**Required fix:** require default, hover where available, focus, pressed, and
disabled universally; define busy, error, and success only on controls or commands
that can actually enter those states.

#### 11. Search debounce is unnecessary until measurement proves otherwise

**Tell:** speculative interaction delay  
**Where:** `identity-exploration.md:294-299`

This is a local list expected to be small. A debounce can make a utility feel less
immediate and adds a timer/state edge case.

**Required fix:** filter synchronously on each keystroke first; introduce a short
debounce only if profiling or an expensive provider requires it.

## Preliminary collision screen

This research is a discovery screen, not legal clearance.

| Candidate | Current evidence | Risk |
| --- | --- | --- |
| Sulco | No exact desktop audio utility found; exact music-artist and engineering-company uses; ordinary Portuguese word | Medium; formal clearance required |
| Desvio | Exact current [Apple App Store app](https://apps.apple.com/co/app/desvio/id6779668536); meanings include error, theft/misappropriation, and deviation ([dictionary](https://www.infopedia.pt/dicionarios/lingua-portuguesa/desvio)) | High |
| Runnel | Exact [consumer app](https://www.runnel.app/), [PyPI stream-processing package](https://pypi.org/project/runnel/), and security/link software uses | High |
| Rivus | Exact active [Rivus mobile app/company](https://www.rivusapp.com/privacy) plus existing software namespace | High |
| Trilho | Active Brazilian [Trilho AI](https://www.trilho.ai/) software product | High |
| Viarum | Established [VIARUM technology company](https://www.viarum.mx/index.html) and near-identical Viarium technology brand | High |
| Patchfield | Google's exact [audio infrastructure and inter-app routing project](https://github.com/google/patchfield) | Disqualifying |

## Approval checklist

Before approving any identity:

1. Complete professional trademark clearance in launch markets and relevant
   software/audio classes, including phonetic equivalents.
2. Run a bilingual name test with unaided pronunciation, spelling recall,
   meaning, and task-association questions.
3. Produce 16/20/24 px optical masters and a one-color symbolic mark whose route
   remains legible without copper.
4. Produce Windows, macOS, and Linux app/status assets using each platform's
   native pipeline, then capture them in real taskbar/menu/panel contexts.
5. Publish a machine-checked contrast table for every text, icon, selected,
   focus, error, and disabled pairing in solid and material fallbacks.
6. Prototype the compact popup and mini mixer in Light, Dark, system material,
   high contrast, reduced transparency, and 200% scaling. Review the hierarchy
   against the accepted layout family, not on isolated mood boards.

## Final recommendation

Keep the **cut-channel concept** and **copper-on-warm-neutral visual language**.
Do not adopt a final name or icon yet. `Sulco` deserves one properly tested round,
but the project should reopen naming if it cannot clear search/legal review or if
English users cannot pronounce and recall it unaided. Continue only with a
redrawn switch-cut family; retire dual-groove and gate. Reduce the appearance
model to Light/Dark color schemes plus a platform-resolved system material, with
solid accessible fallbacks.

Summary — **1 critical · 7 major · 3 minor**  
Verdict — **strong concept, not yet approvable for a public application**
