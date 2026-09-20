<!-- Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 -->

# Independent identity review — V2

Status: **preliminary design and collision audit; not legal trademark clearance or platform-production approval**  
Date: 2026-09-19  
Reviewed: `identity-exploration-v2.md` and every SVG in `icons/v2/`  
Baseline: `identity-review.md`

## Executive verdict

**Rework before identity approval.** V2 resolves most of the conceptual defects in
V1: the solid-palette contrast math is correct, keyboard focus is no longer brand
orange, the appearance model separates color from material, the icon has real
monochrome optical drawings, and the interaction contract is notably more
restrained.

Two claims do not survive independent checking. First, `Rillpath` is no longer a
clean research leader: an exact-name live web property now exists. Second, the
selected route in the SVGs does not actually exit the icon boundary, and the
optical drawings are not all integer-coordinate geometry. Neither issue destroys
the product direction, but both prevent approval of the current naming and icon
claims.

| Area | Verdict | Approval level |
| --- | --- | --- |
| Naming direction | **REWORK** | The research process is accepted; no candidate is approved, and `Rillpath` should not remain the leader without a new round. |
| Cut-channel symbol system | **REWORK** | **Accept the concept**, not the final geometry or platform assets. |
| Color/material system | **REWORK** | **Accept the solid palette and contrast correction**; material resolution and reproducible exports remain incomplete. |
| Interaction/accessibility system | **ACCEPT** | Accept as a specification direction; implementation still requires native assistive-technology and real-surface tests. |
| Overall readiness | **REWORK** | Ready for prototypes and bilingual testing, not naming commitment, trademark filing, stores, or platform asset production. |

## Ranked findings

### Critical

#### 1. `Rillpath` has acquired an exact-name collision and should not remain the research leader

**Tell:** stale absence claim  
**Where:** `identity-exploration-v2.md:52`, `identity-exploration-v2.md:59`

The V2 statement that no exact active software or audio product surfaced is no
longer safe. `rillpath.online` is a live HTTP property titled **“Rillpath —
Flushing plans grounded in the network”** and was registered on 2026-09-02,
according to current third-party domain/site metadata
([record](https://www.scamadviser.com/check-website/rillpath.online)). The source's
reputation verdict is not relied on here; only its concrete domain, registration,
HTTP, title, and description fields are relevant. This evidence does not establish
the operator's legitimacy, trademark rights, or legal priority. It is nevertheless
enough to disprove the clean exact-name screen and to occupy the most obvious exact
domain with another routing/network-adjacent service.

The linguistic case is also weaker than the table suggests. English speakers can
say `RILL-path`, but *rill* is uncommon and can be heard as “real,” “rail,” or
“will.” Brazilian Portuguese provides no stable, obvious reading of the English
`path` ending; `RIL-péti` is only one possible adaptation. The likely need to
repeat or spell the name harms word-of-mouth discovery for a utility.

**Required fix:** remove `Rillpath` from the leader position. Preserve it only as
a challenged test stimulus if counsel considers it worth screening. Run a new
coined-name round, then repeat exact domain, repository, package, store, company,
and phonetic trademark checks on the same day that a finalist is chosen. Do not
reserve package IDs or commission a wordmark before that gate.

V2 handles `Sulco` more honestly. The `/ˈsuw.ku/` / **SOOL-koo** correction is
sound for Brazilian Portuguese, and the document now clearly keeps the word away
from package IDs and stores. Its original risks remain: it is ordinary Portuguese
vocabulary, carries medical and agricultural meanings, and has unrelated exact
commercial and music uses. **Verdict: retain only as a tested candidate; do not
approve it by default when rejecting `Rillpath`.**

### Major

#### 2. The selected route does not exit the icon boundary

**Tell:** geometry claim contradicted by source  
**Where:** `identity-exploration-v2.md:74-77`, `identity-exploration-v2.md:93-97`, all three monochrome SVG descriptions

Every monochrome selected path ends one unit inside its view box: `x=15` in the
16 px file, `x=19` in the 20 px file, and `x=23` in the 24 px file. Because the
terminal is `stroke-linecap="butt"`, the stroke does not extend beyond its
endpoint. The full-size master ends at `x=920` in a 1024-unit view box. It has a
104-unit right margin. The route therefore approaches an edge; it does not exit
the boundary.

This does not make the mark unusable. Weight, longer reach, and the stopped
alternate branch still communicate selection. It does mean that the documented
three-part rationale overclaims the actual drawing.

**Required fix:** either change the language to “continues farther toward the
output edge” and validate that cue, or deliberately redraw an edge-clipped variant
and test it in real trays and menu bars. Do not sacrifice platform-required icon
margins merely to make the prose literally true.

#### 3. Dedicated optical masters exist, but optical consistency is not proven

**Tell:** source geometry presented as validation  
**Where:** `icons/v2/cut-channel-mono-16.svg:5-8`, `cut-channel-mono-20.svg:5-8`, `cut-channel-mono-24.svg:5-8`

The files are genuinely separate drawings and no lane is below 2 px. That resolves
the most serious V1 defect. The selected-to-alternate stroke ratios vary markedly,
however: `3:2` at 16 px, `4:2` at 20 px, and `4:3` at 24 px. The 20 px version
therefore makes selection much more forceful than the 24 px version. At 16 px the
diagonal and miter produce a dense switch node, and the perpendicular stop can
read as a small `T` rather than a blocked output. Those are legitimate optical
choices, but a contact sheet that repeats SVGs is not evidence of OS rasterization.

The phrase “integer-grid geometry” is also inaccurate. The 16 and 24 px drawings
intentionally use half-unit coordinates. Some of those coordinates correctly
align odd-width strokes to physical pixel boundaries, so the drawing should be
described as **stroke-aware pixel alignment**, not integer-only geometry.

**Required fix:** render and capture each native asset at 100% and 200% scale on
the three target platforms, then normalize the perceived weight difference if it
is visible. Test recognition without the document title: “Which route is active?”
and “What does this symbol mean?” A successful answer cannot be inferred from the
SVG structure alone.

#### 4. The two user-facing axes are sound, but runtime material resolution needs a surface role

**Tell:** platform material chosen without lifetime/context  
**Where:** `identity-exploration-v2.md:117-138`

`ColorScheme = system | light | dark` and a transparency preference of
`system | solid` are the right two user choices. They do not fully specify runtime
material, because platform guidance depends on the role and lifetime of a surface.
Microsoft recommends Acrylic for transient, light-dismiss UI and Mica or opaque
surfaces for long-lived windows
([Acrylic guidance](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic),
[Windows best practices](https://learn.microsoft.com/en-us/windows/apps/get-started/best-practices)).
An expanded mini mixer may cease to be transient even if it opens from the tray.

Current Apple guidance likewise distinguishes Liquid Glass for controls and
navigation from standard materials in content, and asks developers to let system
components adapt to accessibility and appearance settings
([Apple materials](https://developer.apple.com/design/human-interface-guidelines/materials)).
“Semantic AppKit popover/menu material” is directionally correct but insufficient
as a cross-version rule. Linux solid fallback remains the honest default.

**Required fix:** retain the two preference axes, but add an internal, non-user
`SurfaceRole`, for example `transient_popup | native_menu | persistent_window |
overlay`. Resolve material from platform, OS version, role, transparency setting,
power/compositor state, and accessibility settings. A long-lived Windows mixer
must not receive Acrylic merely because the popup did.

#### 5. The contrast ratios are correct, but the claimed reproducible record is absent

**Tell:** verified result without checked-in verifier  
**Where:** `identity-exploration-v2.md:179-210`, `identity-exploration-v2.md:289`

I independently converted the declared OKLCH values through the standard OKLab
matrix, hard-clipped linear sRGB channels to the displayable gamut, applied the
sRGB transfer function, and used the WCAG 2 relative-luminance formula. Every
published value reproduces to its stated two decimals:

| Scheme | Pair | Published | Recalculated |
| --- | --- | ---: | ---: |
| Light | ink / surface | 15.91 | 15.9069 |
| Light | ink-muted / surface | 6.87 | 6.8687 |
| Light | route / surface | 5.41 | 5.4087 |
| Light | route-ink / route | 5.57 | 5.5699 |
| Light | route-ink / route-hover | 6.87 | 6.8708 |
| Light | focus-outer / surface | 6.88 | 6.8841 |
| Light | focus-separator / route | 5.82 | 5.8174 |
| Light | focus-separator / route-hover | 7.18 | 7.1760 |
| Light | focus-outer / focus-separator | 7.40 | 7.4042 |
| Dark | ink / surface | 16.27 | 16.2708 |
| Dark | ink-muted / surface | 7.57 | 7.5723 |
| Dark | route / surface | 7.25 | 7.2493 |
| Dark | route-ink / route | 7.25 | 7.2493 |
| Dark | route-ink / route-hover | 9.06 | 9.0552 |
| Dark | focus-outer / surface | 10.74 | 10.7386 |
| Dark | focus-separator / route | 7.75 | 7.7483 |
| Dark | focus-separator / route-hover | 9.68 | 9.6786 |
| Dark | focus-outer / focus-separator | 11.48 | 11.4778 |

The two-color focus construction is a valid resolution of V1: the separator is
the adjacent high-contrast boundary against copper, and the blue outer ring is the
high-contrast boundary against the surrounding surface. Direct blue-to-copper
contrast remains low, but it is no longer the adjacent boundary and therefore is
not the asserted contract. This assumes the full 1 px separator and 2 px outer
ring are rendered without clipping.

The repository contains no contrast script or generated record, despite the claim
that “scripted contrast records exist.” In addition, light `route`, light
`route-hover`, light `focus-outer`, and dark `focus-outer` exceed sRGB in at least
one channel before clipping. Hard clipping is explicitly disclosed, but different
OKLCH converters or perceptual gamut-mapping implementations can produce different
exports and ratios. WCAG also warns that anti-aliasing can make thin rendered
features appear lower contrast than nominal colors
([WCAG contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)).

**Required fix:** check in the exact converter/contrast test, pin the conversion
and gamut-mapping method, make the reviewed sRGB values test fixtures, and fail CI
when a source token changes without regenerated evidence. Keep the current solid
token values unless real composited captures reveal a problem.

#### 6. The SVGs are design sources, not portable native assets

**Tell:** source-level portability mistaken for deployment portability  
**Where:** `identity-exploration-v2.md:83-113`, `icons/v2/cut-channel-master.svg`

`currentColor` is appropriate for a conceptual symbolic source, and the master
uses ordinary sRGB hex colors rather than unsupported OKLCH paint. However, the
full mark also depends on an SVG mask and CSS custom properties, while the small
assets depend on runtime recoloring. Those are not universal contracts across
Windows ICO/PNG resources, macOS template/menu-bar and layered app-icon pipelines,
and Linux symbolic exports. V2 correctly says the platform package is pending;
therefore these files should not be described as deployable assets.

GNOME's current guidance supports the 16 px monochrome direction and 2 px main
strokes, while also requiring pixel-grid alignment and context previews
([GNOME UI icons](https://developer.gnome.org/hig/guidelines/ui-icons.html)).
The Windows and Apple pipelines likewise require their own asset treatments rather
than one raw SVG passed everywhere
([Windows app icons](https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-design),
[Apple app icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)).

**Required fix:** keep the SVGs as editable design sources. Flatten/outline and
export reviewed assets through each native pipeline, preserve a separate one-color
source, and compare raster output rather than assuming `currentColor`, masks, or
CSS survive ingestion.

### Minor

#### 7. `Calha` and `Rillway` do not provide a safe fallback shortlist

**Tell:** backup candidates with weak discovery  
**Where:** `identity-exploration-v2.md:53-54`

`Calha` is intelligible and pronounceable in Brazilian Portuguese but is ordinary
construction vocabulary and awkward for English speakers to pronounce or spell.
Store search is already occupied by the active **Calha Fácil** utility on both
[Google Play](https://play.google.com/store/apps/details?id=com.jhernandezch.calhafacilappmobile)
and the [Apple App Store](https://apps.apple.com/us/app/calha-facil/id6670519984).
That is not an exact-name legal collision, but it confirms poor unqualified store
discoverability.

`Rillway` has the documented exact adjacent-audio use on
[Spotify](https://open.spotify.com/artist/4naEeqLxfkfDF97ImeohBk), and an active
`RILLWAY PTY. LTD.` company record also appears in the Australian government's
[ABN Lookup](https://abr.business.gov.au/Search/ResultsActive?SearchText=rillway).
Its resemblance to “railway” creates an additional spelling/search correction
problem. It should be rejected, not merely held.

**Required fix:** do not promote either candidate when removing `Rillpath` from
the lead. Reopen naming with global pronunciation and store-search constraints in
the brief.

#### 8. The interaction contract is strong, but the focus perimeter needs a clipping rule

**Tell:** correct focus design without edge behavior  
**Where:** `identity-exploration-v2.md:174-177`, `identity-exploration-v2.md:226-265`

The reduced universal state set, synchronous local search, silent successful
routing, explicit failure behavior, reduced motion, non-color selection cues, and
solid reduced-transparency fallback are all appropriate for a small native
utility. The remaining specification gap is mechanical: a 3 px external focus
perimeter can be clipped at a popup or scroll boundary unless layout reserves
space or provides an inset equivalent.

**Required fix:** require an unclipped 1+2 px ring in layout tests, with a native
or inset equivalent when an external ring cannot fit. Validate keyboard order,
screen-reader names/state announcements, forced colors, 200% scaling, and reduced
motion/transparency on each shell before production approval.

## Candidate verdicts after the fresh screen

This table is a preliminary discovery screen as of 2026-09-19. It is not a
trademark opinion, registration search, or proof of availability.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `Rillpath` | **REJECT as leader / legal status unknown** | Exact live domain and routing/network-adjacent web property; weak hear-and-spell behavior across EN and PT-BR. |
| `Sulco` | **REWORK** | Still a viable bilingual test stimulus, but ordinary Portuguese vocabulary and existing exact unrelated uses prevent approval. |
| `Calha` | **REJECT as global lead** | Common construction word, difficult `lh` sound for English users, and noisy app-store results. |
| `Rillway` | **REJECT** | Exact music use, exact company use, and “railway” confusion. |
| `Leito` | **REJECT** | Existing software use and dominant medical/bed association. |
| `Canalis` | **REJECT** | Exact app/software use already documented. |
| `Duto` | **REJECT** | Exact active software/audio-adjacent use already documented. |

Negative package/store queries did not surface an exact `Rillpath` crate, npm
package, or Apple listing during this check. That absence is not availability
evidence and does not cancel the exact web-property collision.

## Asset verdicts

| Asset | Concept verdict | Production verdict | Reason |
| --- | --- | --- | --- |
| `cut-channel-master.svg` | **ACCEPT** | **REWORK** | Stronger product-specific metaphor; mask/CSS source must be flattened and tested per pipeline. |
| `cut-channel-mono-16.svg` | **REWORK** | **REWORK** | Geometry survives without copper, but the dense switch node and stop require native-size recognition testing. |
| `cut-channel-mono-20.svg` | **ACCEPT** | **REWORK** | Clearest selected-weight separation; still no native capture/export proof. |
| `cut-channel-mono-24.svg` | **REWORK** | **REWORK** | Selected/alternate weight separation is weaker than at 20 px and needs optical normalization. |
| `cut-channel-contact-sheet.svg` | **ACCEPT as documentation** | **Not a deployable asset** | Useful comparison, but cannot substitute for OS raster captures. |

## Approval gates

Concept work may continue immediately. Public naming, store packaging, and final
platform production must wait for separate gates:

1. **Name gate:** a new bilingual shortlist, unaided hear/say/spell/recall tests,
   exact same-day store/domain/package/company screening, then professional word
   and phonetic trademark clearance in launch territories and classes.
2. **Symbol gate:** blind meaning/active-route testing plus real 16/20/24 px tray,
   menu-bar, and panel captures at 100% and 200%, positive, inverse, and forced
   colors.
3. **Color gate:** checked-in conversion/contrast test and composited-pixel tests
   over every native material, with solid fallback as the guaranteed baseline.
4. **Material gate:** role-aware platform resolver, version gates, reduced
   transparency, power/inactive fallbacks, and no simulated cross-platform glass.
5. **Accessibility gate:** keyboard, focus clipping, screen reader, high contrast,
   scaling, RTL, reduced motion, and reduced transparency tested in each native
   shell.

## Final recommendation

Keep the **cut-channel concept**, **warm neutral/copper palette**, **native type**,
and **scheme plus transparency preference**. The design no longer looks like a
generic AI-generated audio dashboard; its restraint and route-specific metaphor
are credible. Do not select `Rillpath`, freeze the current icon geometry, or call
the asset package portable yet. Run a fresh naming round, correct the icon claims,
add role-aware material resolution, and turn the verified contrast calculation
into a checked-in test.

Summary — **1 critical · 5 major · 2 minor**  
Verdict — **concept direction accepted; identity, legal name, and platform assets require rework before public release**
