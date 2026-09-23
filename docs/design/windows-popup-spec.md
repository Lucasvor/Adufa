# Windows compact popup specification

Status: accepted direction for the first Windows surface.

This document turns the selected concept (option 1, the tray menu) into a
testable implementation contract. It prevents a technical prototype from
silently becoming the product design.

## Product shape

- The primary surface is a compact, native-feeling tray popup, not a dashboard.
- Its visual hierarchy is: product header, active applications, separator,
  Settings, Exit.
- The surface uses the Windows typography and spacing rhythm. It may use the
  platform's native material and corner treatment, but must remain readable
  without transparency.
- The provisional display name is `Adufa`. Internal crate and module names stay
  brand-neutral until naming and trademark review are complete.
- The process owns a notification-area icon. Dismissing the popup keeps the
  process and tray icon alive; `Exit` is the explicit termination action.
- Clicking the tray icon toggles the popup. Windows may place a newly installed
  icon inside the notification-area overflow until the user pins it.
- Clicking outside dismisses the popup. Dismissal ends every temporary
  inspection mode without terminating the resident process.

## Header

- Use a recognizable speaker-and-sound-wave symbol at 20--24 logical pixels.
- Do not use the abstract Cut Channel exploration as the primary audio glyph.
- Show the product name with the short `Audio router` descriptor beneath it.
  Treat the icon and both text lines as one optically centered identity block,
  with a full spacing step before the header rule and application list.
- Settings has at least a 40 x 40 logical-pixel hit target.

## Sound Locator

- Application and device discovery updates automatically from native platform
  events. The interface does not expose a manual rescan action.
- The header uses a compact labelled control instead of an unexplained icon.
  Its stable labels are `Find sound` while inactive and `Listening` while
  active. Activating it toggles Sound Locator without changing routes or
  opening a separate window.
- While active, it shows the current signal level of every audible Application
  and emphasizes the strongest source without hiding the selected output.
- Each Application row keeps the same vertically centered name, icon, and
  selected-output alignment in both modes. A short rounded level meter appears
  at the lower edge only while Sound Locator is active; it never becomes a
  full-width underline or pushes the labels upward.
- An Application's displayed level is the highest current peak among its child
  audio sessions. Session levels are never added together.
- The strongest audible Application receives a restrained row highlight and a
  two-pixel leading rail. Together with its live meter, identification never
  depends on color alone and adds no second-line status copy.
- Strongest-source emphasis is stable: another Application must remain stronger
  for approximately 300 ms before taking the highlight. The last identified
  source remains emphasized for 1 second after its signal becomes silent.
- Residual signal below approximately -50 dBFS is treated as silence for source
  identification. This threshold is an internal product default, not a V1 user
  preference.
- While Sound Locator is active, level meters sample at 20 Hz and invalidate
  only rows whose visible level changed. No level polling continues after the
  mode or popup closes.
- Application rows retain their normal routing action while Sound Locator is
  active. Opening and using the adjacent output selector counts as interaction
  inside Adufa, so it neither dismisses the owner popup nor stops level meters.
- Sound Locator remains active until the user toggles it off or dismisses the
  popup. Closing the popup stops level sampling immediately.
- When no Application exceeds the approximately -50 dBFS identification
  threshold, the control remains accent-colored and the header shows the
  compact, static `Listening` label. The interface adds no status row, decorative
  looping animation, or automatic timeout; it waits until audio appears, the
  user toggles Sound Locator off, or the popup closes.

## Application audio controls

- The adjacent Application surface places one compact volume row between the
  Application identity and the `Audio output` label.
- The row contains a mute toggle, a 0--100% slider, and a compact percentage.
  Dragging the slider is visually continuous and commits when the pointer is
  released; changing volume does not change the selected output.
- The volume control is not a selectable menu row. Hover and keyboard focus
  affect only its interactive affordances; no full-width selection fill is
  drawn. Eight-pixel gaps and rules separate identity, volume, and output
  choices.
- One grouped Application command applies the same value to every active audio
  session represented by that Application. The displayed value is the highest
  session value, and the group is shown muted only when all represented
  sessions are muted.
- Up and Down move keyboard focus between the volume row and output choices.
  Left and Right adjust focused volume by five percent; Space or Enter toggles
  mute. Escape continues to dismiss the surface.
- Native audio-session change events refresh the displayed value when another
  mixer changes volume or mute state. Changes tagged as originating in Adufa
  are already reflected locally and do not trigger a redundant full refresh.
- Slider repainting is double-buffered so pointer movement never exposes an
  intermediate white surface.

## Quick access surfaces

- `Ctrl+Alt+A` is the initial system-wide shortcut. Registration uses the
  supported Win32 `RegisterHotKey` API with key-repeat suppression and is
  released when Adufa exits.
- When the shortcut is pressed over a window belonging to an audible
  Application, Adufa opens that Application's output selector near the cursor.
  This is the supported implementation of the contextual overlay concept.
- When the cursor does not resolve to an audible Application, the shortcut
  opens the compact Application list near the cursor and preserves normal
  keyboard navigation and output selection.
- Both quick-access surfaces close on Escape or outside click. They never keep
  an invisible sampling or input-hook loop alive after dismissal.
- Adufa does not inject a command into another application's taskbar Jump List.
  Windows taskbar extension APIs let an application customize its own Jump
  List, not another application's button.
- Adufa does not inject into the Windows 11 Quick Settings volume flyout because
  Windows exposes no supported extension API for that system surface. A native
  Adufa-owned surface may mirror the workflow without modifying Explorer.
- A regular right-click normally opens the application's Jump List, while
  `Shift + right-click` may open its window menu. Neither is a public
  third-party extension surface. Adufa does not ship code injection or private
  Explorer/ShellExperienceHost hooks.
- The experimental Windows build observes right-clicks out of process and uses
  monitor work-area geometry to recognize the taskbar without private Explorer
  classes. The low-level callback captures only the pointer location; the UI
  queue uses documented UI Automation to inspect the button AppID and resolves
  its running executable independently of audio activity, with the accessible
  name and executable path as fallbacks. The original
  right-click always continues to Explorer. After the native Jump List opens,
  the non-activating Adufa selector is placed beside its visible content
  rectangle rather than the larger host bounds. The existing icon match remains
  a fallback. The calculation removes both the DWM shadow and
  the DPI-scaled transparent inset used by Windows' XAML Jump List host,
  preserving both sets of commands without overlap and aligning their visible
  top edges. If resolution fails, only the Windows menu remains. Choosing an
  Adufa output closes both companion surfaces only after routing succeeds. While
  the selector is open, a separate temporary mouse hook closes it on any pointer
  press outside its bounds and never consumes that input.

## Settings

- Settings uses the same 304-pixel native compact surface, spacing rhythm,
  colors, typography roles, and input states as the main popup.
- The header is one line: Back and `Settings`; it does not repeat the product
  name as a subtitle.
- `Open at login` is a real full-row control with the platform-neutral
  description `Start Adufa when you sign in`. The `System` section label and
  control surface are separated by one four-pixel spacing step; labels never
  touch hover or focus fills.
- `Quick access` documents the active shortcut and its outcome instead of
  showing inert placeholder copy.
- `System audio settings` opens the platform's native audio settings. On
  Windows the adapter opens the volume mixer.
- Mouse hover and keyboard focus are separate states. A surface opened by mouse
  must not display a permanent keyboard-focus block.

## Cross-platform visual contract

- Platforms share semantic roles (`surface`, `hover`, `pressed`, `rule`,
  `text`, `secondary_text`, `accent`, `accent_soft`, and `meter_track`) instead
  of sharing renderer-specific paint code.
- Windows uses Segoe UI/Segoe Fluent Icons, macOS uses SF Pro/SF Symbols, and
  Linux uses the desktop's native UI font and symbolic icon theme.
- Geometry, hierarchy, copy, target sizes, routing behavior, and locator states
  remain consistent. Native text rasterization, material, and shell placement
  may differ.
- Popup reveal uses one restrained opacity transition of approximately 180 ms.
  It never animates layout dimensions, delays input, or adds decorative motion.
  Windows follows `SPI_GETCLIENTAREAANIMATION`; macOS and Linux adapters must
  honor their equivalent reduced-motion preference and reveal immediately when
  motion is disabled.

## Application rows

- One row represents one grouped application identity.
- Show the real application icon at 24 logical pixels when Windows exposes one.
- Resolve a 32, 48, or 256-pixel Shell source according to DPI; never enlarge a
  16-pixel small icon into a 24-DIP row.
- Use one consistent neutral application fallback glyph; never generate letter
  avatars from process names.
- Show the application name on the left and the friendly selected output name on
  the right, followed by a chevron.
- Every row is interactive across its full width. Hover, keyboard focus, pressed,
  disabled, and routing-error states must be distinguishable.
- Truncation must preserve the application name before the output label.

## Output selection

- Activating an application row opens a menu adjacent to that row. The menu
  begins with the real Application icon and name, followed by an `Audio output`
  section label and the available choices.
- The first choice is `System default`; following choices are active outputs by
  their Windows friendly names.
- The current choice is marked. Selecting a choice invokes the platform routing
  backend for every process in that grouped application.
- The visible selection changes only after the backend reports success.
- Failure is explicit and concise. The interface must never simulate success.

## Footer actions

- `Settings` and `Exit` are full-width action rows with consistent 20-pixel
  outline icons.
- Both actions are real hit targets. Settings contains only working controls or
  truthful explanatory copy; it never displays an inert prototype placeholder.

## Acceptance checks

1. Mouse selection opens the correct application's output menu.
2. Keyboard navigation reaches Sound Locator, every application, Settings, and
   Exit.
3. Enter or Space performs the focused action; Escape closes the popup.
4. Device labels never expose MMDevice IDs.
5. Application rows never display generated initials when an icon lookup fails.
6. Route state changes only after a successful platform call.
7. The empty state remains useful and visually consistent.
8. At 100%, 125%, 150%, and 200% scaling, icons and text remain aligned and no
   clickable label wraps.
