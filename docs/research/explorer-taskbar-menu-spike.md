# Windows 11 taskbar window-menu integration spike

Research date: 2026-09-19  
Scope: architecture and evidence only; no injector, payload, hook, registry change, or
production code was created or executed.

## Decision summary

The exact request is:

> Add an **Audio output** item to the menu opened by **Shift + right-click** on the
> taskbar button of an application that Adufa does not own.

[Microsoft documents](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec)
this gesture as showing the application's **window menu** (and the group window menu
for a grouped button). It is not the normal right-click Jump List. This distinction
matters: [Jump List APIs](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar-extensions)
let an application populate the list associated with its own taskbar identity, while
the window-menu APIs expect the window-owning application to modify and handle its
own menu commands.

There is no documented Windows extension point for a third party to add a live
command to another application's taskbar window menu. The exact experience therefore
requires an unsupported in-process interception of private Shell behavior, or a
cross-process mutation plus an unsupported way to intercept command dispatch. Neither
is suitable for Adufa production.

**Recommendation:**

- **GO** for a read-only, disposable-VM ownership study and for an Adufa-owned
  look-alike popup experiment.
- **NO-GO** for implementing or shipping injection into `explorer.exe`,
  `ShellExperienceHost.exe`, or another application's process.
- Reconsider the exact native-menu requirement only if Microsoft publishes a taskbar
  window-menu extension contract. A successful private-hook prototype would not by
  itself change the production decision.

This conclusion is stronger than “injection is difficult”: the design would depend
on an interface Microsoft does not support and explicitly advises applications not
to depend on. Microsoft warns against undocumented APIs, private exports, registry
keys, offsets, and version checks as compatibility mechanisms in its
[Windows app-compatibility guidance](https://learn.microsoft.com/en-us/windows/uwp/updates-and-versions/application-development-for-windows-as-a-service).
[Microsoft Store policy 10.2.8](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies-and-code-of-conduct)
also requires supported methods and user consent for changes to the Windows
experience, explicitly calling out unsupported use of accessibility or undocumented
APIs.

## What the gesture actually opens

[Microsoft's Windows shortcut reference](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec)
defines:

- `Shift + right-click` on a taskbar button: show the **window menu for the app**;
- the same gesture on a grouped button: show the **window menu for the group**.

That is a different surface from the normal taskbar right-click **Jump List**. A Jump
List is application-specific, and its custom destinations and tasks are supplied by
the application associated with that taskbar identity. `ICustomDestinationList`
therefore cannot be used as a general-purpose extension point for Spotify, Chrome,
Voicemeeter, or any other publisher's button.

[The classic Win32 `GetSystemMenu` API](https://learn.microsoft.com/en-us/windows/desktop/api/Winuser/nf-winuser-getsystemmenu)
initially looks closer. It returns a modifiable copy of a window's menu, whose
predefined commands include `SC_MOVE`, `SC_SIZE`, and `SC_CLOSE`. But Microsoft's
contract is framed around an application modifying the menu of its window and handling
the resulting `WM_SYSCOMMAND` messages. It does not define third-party command
ownership or callback registration for another process.
Even if an external process could make an item visible in a particular build, the
foreign window would receive the command and would not know how to execute Adufa's
action. Grouped taskbar buttons add another Shell-owned composition layer.

Traditional Shell context-menu handlers do not fill this gap. Microsoft's
[documented handlers](https://learn.microsoft.com/en-us/windows/win32/shell/shell-exts)
attach to file types and Shell namespace objects. There is no documented handler
category for another publisher's taskbar button or window menu.

Consequently, a lab must first establish whether a tested build displays the target
window's actual system menu, a Shell-composed copy, or a private XAML/Win32 menu. The
answer must not be assumed from appearance.

## Comparison of candidate approaches

| Approach | Can add the item to the exact native menu? | Process boundary | Build/architecture burden | Principal failure mode | Verdict |
| --- | --- | --- | --- | --- | --- |
| DLL injection into `explorer.exe` | Potentially, after discovering a private interception point | In the desktop Shell process | Separate native payload for the target architecture; private seam must be revalidated for every serviced build | Crash, hang, or restart of desktop/taskbar; silent corruption after an update | **NO-GO** |
| DLL injection into `ShellExperienceHost.exe` | Unproven and probably the wrong boundary for this gesture | In a packaged Windows shell host | Same architecture constraints plus process isolation/mitigation uncertainty | No effect on the target menu, or destabilization of unrelated shell experiences | **NO-GO** |
| ExplorerPatcher-style hooks | Potentially; this is a family of private patches, not a supported API | Usually one or more Windows shell processes | Highest: per-build patterns/symbol knowledge and x64/ARM64 variants | Cumulative-update breakage, Explorer crash loops, security-product detections | **NO-GO** |
| UI Automation + Adufa overlay | No; it can identify/observe an element and place Adufa UI nearby | Entirely out of process | One client can cross process/bitness through UIA helpers; UIA identity still varies by build | Wrong app correlation, localization/UI-tree changes, race with disappearing elements | **GO for approximation** |
| `WH_MOUSE_LL` + Adufa overlay | No; it can detect or suppress the gesture, not append to the native menu | Callback runs in Adufa, not injected into the target | No matching target-process DLL required for the low-level hook | Global-input latency, hook timeout/removal, duplicate native and Adufa menus | **GO only as opt-in experiment** |

### 1. Direct injection into `explorer.exe`

[Microsoft's Shell debugging documentation](https://learn.microsoft.com/en-us/windows/win32/shell/debugging-with-the-shell)
says the desktop and taskbar windows are created in the desktop `Explorer.exe`
process. That makes Explorer the first ownership hypothesis for the target gesture,
but not a supported extensibility promise.

An exact implementation would need, at minimum, an in-process adapter at the private
point where Explorer constructs or dispatches the window menu. Public documentation
does not name such a point. Finding it would require runtime observation and private
implementation knowledge. Any in-process defect shares Explorer's fate. Microsoft
likewise notes in its [Shell extension guidance](https://learn.microsoft.com/en-us/windows/win32/shell/shell-exts)
that in-process extension DLL failures can crash or hang Explorer; an undocumented
taskbar patch has even less isolation.

This option is not made safe by keeping audio logic small. The minimal injected code
still must participate in a private UI lifetime, identify grouped/pinned/running
targets, allocate a collision-free command identity, and communicate selection to an
out-of-process broker. Teardown races occur exactly while Explorer or the menu is
closing.

### 2. Direct injection into `ShellExperienceHost.exe`

Microsoft does not document `ShellExperienceHost.exe` as an extension host for the
taskbar window menu. Microsoft's troubleshooting material primarily discusses it as a
Windows shell experience package and, on older Windows versions, as a host involved in
Start. The open-source
[ExplorerPatcher project describes](https://github.com/valinet/ExplorerPatcher/wiki/Using-ExplorerPatcher-as-shell-extension)
injecting it for taskbar flyouts such as network, battery, and sound, while identifying
`explorer.exe` as where the taskbar and most related UI live. That is useful
implementation evidence, but it is not a Microsoft compatibility contract.

For this exact gesture, selecting `ShellExperienceHost` before proving ownership would
be architecture by process name. The read-only ownership study must decide which
process creates the menu on each build. Unless that study identifies the host and a
public contract, this approach remains no-go.

### 3. ExplorerPatcher-style hooks

“ExplorerPatcher-style” is not an alternative loading API; it combines getting code
into Windows shell processes with private hooks, detours, replacement components, and
build-aware compatibility work. The project's
[release history](https://github.com/valinet/ExplorerPatcher/releases) is evidence of
the maintenance model: taskbar variants are restricted to named build families, fixes
regularly mention particular build numbers, x64 and ARM64 artifacts are separate, and
some changes address Explorer deadlocks or crash loops.

This can prove that unsupported shell modification is technically possible. It does
not prove that a narrow Adufa patch will be stable. Adufa would inherit a permanent
Windows-internals compatibility program for one menu item, including preview-build
testing before every Windows rollout and emergency kill-switch releases afterward.

### 4. UI Automation overlay

[`IUIAutomation::ElementFromPoint`](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-elementfrompoint)
can return the element under the pointer, and its bounding rectangle can anchor an
Adufa-owned popup. It does not allow insertion of a new child into a UIA provider's
UI. UIA properties are read-only unless an exposed control pattern defines an
operation.

Target identity is best-effort. [Microsoft states](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids)
that `AutomationId` is optional, is only expected to be unique among siblings, and is
not guaranteed stable across an
application's releases or builds. Names can be localized, and the UIA process ID
identifies the element provider; for a Shell-owned taskbar element it must not be
assumed to identify the application represented by the button. Grouped buttons,
pinned-but-not-running apps, multiple profiles, and apps with several audio processes
need explicit fallback behavior.

[Microsoft's UIA threading guidance](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)
requires desktop-wide UIA work to run on a separate MTA thread that owns no windows.
It also documents cross-bitness helper lifetime issues and requires clients to handle
invalidated elements gracefully. These are manageable for an overlay, but not evidence
that UIA can modify the native menu.

### 5. Low-level mouse hook

`WH_MOUSE_LL` is the least invasive way to observe the gesture. Microsoft
[explicitly states](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)
that this low-level callback is called in the installing process and is **not**
injected into another process. It must have a message loop and hand work off
immediately. If it exceeds `LowLevelHooksTimeout`, Windows 7 and later may silently
remove it; on Windows 10 1709 and later, the effective maximum timeout is 1000 ms.

The hook can either pass the click through, producing the native menu as well as an
Adufa popup, or suppress it and replace the native experience. The latter is visibly
not “add one native item” and affects a global input path. Microsoft describes global
hooks as a shared resource to restrict to special-purpose or debugging applications
and recommends removing them as soon as possible.

For an opt-in approximation, the callback should record only coordinates, button, and
modifier state, enqueue the event, and return. UIA lookup, application/audio-session
correlation, and popup construction belong off the hook thread.

## Architecture for an isolated laboratory

The laboratory has two gates. Gate A is authorized by this document and is entirely
read-only. Gate B would involve in-process experimentation and requires a separate,
explicit authorization after Gate A results are reviewed. This document does not
authorize or supply an injector or payload.

```text
physical host
  |
  +-- test controller and result archive (no shell modification)
  |
  `-- disposable Hyper-V VM, one Windows build + architecture
        |
        +-- stock taskbar / test user / synthetic target apps
        +-- read-only observer
        |     process tree, HWND ownership, UIA tree, ETW/WER evidence
        +-- Adufa broker stub (out of process; no audio mutation required)
        `-- external watchdog + VM checkpoint rollback
```

### Gate A — ownership and behavior study (GO)

1. Use a clean Generation 2 VM with no personal account, secrets, production audio
   configuration, or shared host folders. Take a powered-off checkpoint.
2. Record the exact Windows edition, full build/UBR, architecture, installed update,
   display scale, language, and taskbar grouping configuration.
3. Use synthetic Win32 and packaged test applications representing a single window,
   grouped windows, a pinned-but-not-running app, and a multi-process audio app.
4. Observe the gesture without mutation: process/window ownership, UIA subtree and
   properties, menu lifecycle, target HWND/PID/AppUserModelID, and Explorer or shell
   crash events. Microsoft Sysinternals Process Explorer can verify processes, loaded
   DLLs, and handles; Windows debuggers/ETW can establish provenance without loading
   Adufa code into a Windows process.
5. Repeat after one cumulative update. A changed owner, UI tree, class, event order, or
   menu technology is a compatibility change, even when the screenshot looks identical.
6. Archive only metadata, traces, and screenshots. Revert the VM checkpoint after each
   build run.

Gate A answers these questions:

- Is the displayed menu the target window's `HMENU`, an Explorer-owned copy, or a
  different implementation?
- Which process owns menu construction and command dispatch?
- Does the answer differ for running, pinned, and grouped buttons?
- Is there any documented interface surfaced by inspection? If not, Gate A ends with
  the production no-go unchanged.

### Gate B — hypothetical exact-menu experiment (not authorized)

If separately approved, keep the architecture deliberately split:

- a **build gate** outside the Shell refuses every OS build and architecture not
  explicitly qualified;
- a minimal **in-process adapter** contains no audio engine, configuration, network,
  updater, or drawing system;
- the existing **Adufa broker** remains out of process and owns app/audio correlation,
  routing commands, localization, and state;
- a versioned local IPC boundary passes only `{target identity, command, request id}`;
- an **external watchdog** can disable the experiment without depending on Explorer;
- the feature is off by default and has no auto-start or persistent registration in
  the first experiment.

Do not load the production Adufa binary into a Windows process. Do not reuse the
experimental adapter as a production Shell extension. Do not use remote downloads,
runtime pattern updates, or broad process injection. A test adapter is disposable per
build family.

## Reversal and recovery plan

The recovery design must not depend on the process being modified.

1. **Primary reversal:** power off and revert the VM to the pre-test checkpoint.
2. **Runtime kill switch:** an external per-VM marker disables loading before Explorer
   starts; absence or parse failure means disabled.
3. **Crash-loop threshold:** after one unexpected Explorer termination or hang, the
   watchdog disables the experiment and starts a clean Explorer session. It does not
   retry automatically.
4. **No persistent first run:** the initial separately authorized experiment is
   manually started for one session. Persistence is a later gate, not a convenience.
5. **Artifact inventory:** record every file, scheduled action, package/registry entry,
   and certificate introduced by the experiment. Removal must leave a stock DLL list
   in Explorer, verified with Process Explorer.
6. **Break-glass access:** retain an administrator account that is not configured to
   launch the experiment and document Safe Mode/checkpoint recovery before testing.

No production machine, daily-driver profile, or machine containing user credentials
is an acceptable test target.

## Architecture and Windows-build requirements

### CPU architecture

- Test and package x64 and ARM64 as different targets. In-process code must match the
  target process architecture.
- Microsoft documents that a 32-bit DLL cannot be injected into a 64-bit process and
  vice versa, and that the machine architectures must match in its
  [`SetWindowsHookEx` documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
  and [WOW64 implementation notes](https://learn.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details).
  On Windows on Arm, an ARM64 process cannot load x64 or ARM64EC binaries; it can load
  ARM64 (and compatible ARM64X) binaries according to Microsoft's
  [Arm64EC interoperability table](https://learn.microsoft.com/en-us/windows/arm/arm64ec).
  Therefore an ARM64 Explorer requires an ARM64-compatible adapter.
- ARM64EC is useful for an x64-compatible application process, not as a shortcut for
  loading x64 code into a classic ARM64 Shell process.
- The UIA/`WH_MOUSE_LL` approximation does not require an in-process target DLL, but it
  still needs native ARM64 qualification for performance, packaging, and behavior.
- x86 is out of scope for the Windows 11 Shell payload. It may appear among target
  applications, so correlation tests must include one x86 app on x64 Windows.

### Windows builds

Never support “Windows 11” as one target for a private hook. Qualify exact build
families and UBR ranges. At the research date, Microsoft's
[supported-version table](https://learn.microsoft.com/en-us/windows/release-health/supported-versions-windows-client)
lists Windows 11 24H2 as build 26100 and 25H2 as build 26200; supported editions and
additional device-specific releases must be taken from the current Windows
release-health table at test time.

Minimum matrix for Gate A and any later experiment:

| Dimension | Required coverage |
| --- | --- |
| Architecture | x64 and native ARM64 hardware/VM |
| GA versions | Every Windows 11 build family Adufa claims to support |
| Servicing | Earliest supported UBR, current security update, current optional preview |
| Pre-release warning | Latest Release Preview as an early signal; never claim support from Insider success |
| Taskbar state | centered/left, auto-hide, multiple monitors, combined/grouped buttons, pinned/running |
| Display | 100%, 150%, 200%, mixed-DPI monitors, high contrast, reduced animation |
| Locale/input | en-US and pt-BR minimum, keyboard invocation, mouse, touchpad |
| Targets | Win32 x64, Win32 x86, packaged app, multi-window, elevated app, multi-process audio app |

Microsoft's [release notes](https://learn.microsoft.com/en-us/windows-insider/release-notes/release-preview-24h2-25h2/build-26100-9267-26200-9267)
document gradual rollout, in which feature availability can vary by device within the
same build. Build number alone is therefore insufficient evidence for private UI
compatibility. Record feature rollout state and repeat on representative machines.

## Risk register

| Risk | Likelihood | Impact | Control | Residual decision |
| --- | --- | --- | --- | --- |
| Explorer crash/hang or crash loop | High for private in-process hooks over time | Critical | Disposable VM, external watchdog, no persistence first | Production no-go |
| Cumulative update changes private seam | High | High | Exact-build allowlist, preview testing, kill switch | Still no compatibility contract |
| Wrong taskbar item maps to wrong audio app | Medium/high | High | Stable engine identity plus explicit chooser fallback | Exact menu remains unsafe |
| Grouped button has several windows/processes | High | Medium | Show app/session chooser rather than guess | Product rule required |
| x64/ARM64 binary mismatch | Medium | Critical | Architecture-specific builds and loader rejection | Test both architectures |
| Global mouse hook stalls input or disappears | Low/medium with minimal callback | High | Dedicated loop, enqueue-only callback, health signal | Approximation only |
| UIA identifiers change/localize | High | Medium | Cache minimal properties, multiple signals, fallback | Best-effort only |
| UIPI/integrity boundary blocks target | Medium | Medium | Do not request `uiAccess` merely to bypass it; degrade safely | Elevated targets unsupported |
| Security/EDR flags shell tampering | High | High | Avoid injection in distributable build | Production no-go |
| Store certification failure | High | High | Ship only supported Adufa-owned UI | Exact integration excluded |
| Uninstall leaves shell modification behind | Medium | Critical | No persistence, artifact manifest, VM rollback | Must be zero before any wider lab |

Requesting `uiAccess=true` is not a casual workaround. [Microsoft requires](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-only-elevate-uiaccess-applications-that-are-installed-in-secure-locations)
PKI signing and installation in secure locations for UIAccess, because it bypasses
UIPI boundaries.
Adufa's ordinary taskbar approximation should run at normal user integrity and fail
closed for elevated or inaccessible targets.

## Go/no-go criteria

### Gate A: read-only ownership study

**GO** when a disposable VM and checkpoint exist and the study makes no in-process or
persistent change. **STOP** if observation requires changing system binaries,
disabling security controls, or using a real user profile.

### Gate B: separately authorized exact-menu prototype

Proceed only if all are true:

1. the owner and dispatch path are proven for every matrix entry, not inferred from a
   process name;
2. the experiment has an exact-build and architecture fail-closed gate;
3. rollback works after forced Explorer termination and mid-menu teardown;
4. the in-process adapter has no routing engine, persistence, networking, or updater;
5. a security review accepts the IPC, loader, signing, and least-privilege model;
6. the team accepts that the result is a disposable research artifact, not production
   code.

Any raw-offset patch, undocumented export dependency, broad global injection,
security-control disablement, host-machine test, or failure to cleanly reverse is an
immediate **NO-GO**.

### Production

Production remains **NO-GO** unless Microsoft publishes a supported extension API for
this surface. Performance or reliability in a finite test matrix cannot create a
compatibility contract. Store policy, support burden, user trust, and the blast radius
of executing inside the desktop Shell remain disqualifying.

The production alternative is a single Adufa-owned selector, styled consistently on
all platforms and opened beside the taskbar button/pointer. On Windows it may use an
explicit opt-in gesture plus UIA for best-effort anchoring and an application chooser
when identity is ambiguous. That preserves the desired interaction without claiming
native menu insertion.

## Sources

Primary Microsoft sources unless marked otherwise:

- [Keyboard shortcuts in Windows — taskbar shortcuts](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec)
- [Taskbar Extensions and Jump Lists](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar-extensions)
- [`ICustomDestinationList`](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-icustomdestinationlist)
- [`GetSystemMenu`](https://learn.microsoft.com/en-us/windows/desktop/api/Winuser/nf-winuser-getsystemmenu)
- [`WM_INITMENU`](https://learn.microsoft.com/en-us/windows/win32/menurc/wm-initmenu)
- [Working with Shell Extensions](https://learn.microsoft.com/en-us/windows/win32/shell/shell-exts)
- [Registering Shell Extension Handlers](https://learn.microsoft.com/en-us/windows/win32/shell/reg-shell-exts)
- [Debugging with the Shell](https://learn.microsoft.com/en-us/windows/win32/shell/debugging-with-the-shell)
- [`IUIAutomation::ElementFromPoint`](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-elementfrompoint)
- [UI Automation element property identifiers](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids)
- [UI Automation threading issues](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)
- [`LowLevelMouseProc`](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)
- [`SetWindowsHookExW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- [Hooks overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks)
- [WOW64 implementation details — global hooks](https://learn.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details)
- [Arm64EC interoperability and binary loading](https://learn.microsoft.com/en-us/windows/arm/arm64ec)
- [Windows app compatibility guidance](https://learn.microsoft.com/en-us/windows/uwp/updates-and-versions/application-development-for-windows-as-a-service)
- [Microsoft Store policy 10.2.8](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies-and-code-of-conduct)
- [UIPI and UIAccess security policy](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-only-elevate-uiaccess-applications-that-are-installed-in-secure-locations)
- [Supported Windows client versions and build families](https://learn.microsoft.com/en-us/windows/release-health/supported-versions-windows-client)
- [Process Explorer (Microsoft Sysinternals)](https://learn.microsoft.com/en-us/sysinternals/downloads/process-explorer)
- [ExplorerPatcher shell-process notes](https://github.com/valinet/ExplorerPatcher/wiki/Using-ExplorerPatcher-as-shell-extension) — primary source for that project, not a Microsoft contract
- [ExplorerPatcher releases](https://github.com/valinet/ExplorerPatcher/releases) — primary evidence of its build-specific maintenance burden, not Microsoft guidance
