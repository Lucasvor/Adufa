# Windows shell integration options for Adufa

Research date: 2026-09-19

## Executive conclusion

The exact mockup interactions are not both available through supported public Windows extension points:

- **Surface 2 — add an audio-output submenu to another app's taskbar menu:** Windows exposes Jump List and taskbar customization for an application's own taskbar identity. It does not expose a supported API for Adufa to append a submenu to Chrome's, Spotify's, or another publisher's taskbar menu.
- **Surface 6 — add Adufa controls to the native Windows 11 volume / Quick Settings flyout:** Microsoft states that Windows 11 Quick Settings has no extension API.

Both experiences can still be approximated convincingly with Adufa-owned UI. The safe product direction is to keep the native shell untouched, identify the target application, and open the same lightweight Adufa selector beside the pointer or notification area.

## Surface 2: another application's taskbar menu

### What the supported taskbar APIs do

`ICustomDestinationList` lets an application provide a custom Jump List for the taskbar. Microsoft's documentation describes destinations and tasks as content supplied by **the application** for its own Jump List. AppUserModelID guidance also says an app uses its own identity consistently across its processes, windows, shortcuts, file associations, and Jump List calls. Calls made from another process have no effect when the target app has no explicit AppUserModelID. These APIs are not a general third-party menu-extension point.

Sources:

- [Taskbar Extensions](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar-extensions)
- [ICustomDestinationList](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-icustomdestinationlist)
- [Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)

Traditional Shell context-menu handlers are registered for file types or predefined Shell object types. The documented registration model does not provide a handler category for arbitrary third-party taskbar buttons.

- [Registering Shell Extension Handlers](https://learn.microsoft.com/en-us/windows/win32/shell/reg-shell-exts)

### Supported approximation: an Adufa-owned taskbar gesture

A practical approximation is:

1. The user explicitly enables an optional shortcut such as **Shift + right-click**.
2. Adufa installs a documented `WH_MOUSE_LL` hook and keeps its callback extremely small: capture the click coordinates and enqueue work.
3. If the pointer is over a normal application window, resolve the top-level window and PID, then match that PID to an active audio session.
4. If the pointer is over the taskbar, use documented UI Automation `ElementFromPoint` only to inspect the taskbar element under the pointer, then try to correlate its accessible name with an active audio application.
5. Display an **Adufa-owned output selector** next to the click. Do not modify the other app's Jump List.

This uses documented primitives, but taskbar-item-to-application correlation is inherently best-effort. UI Automation exposes the element under a desktop point, but its `AutomationId` is not mandatory and is not guaranteed stable between Windows builds. The element's process is normally the Shell provider (`explorer.exe`), not the application represented by the taskbar button, so its PID is not a reliable target-app identity.

Sources:

- [`LowLevelMouseProc`](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)
- [`SetWindowsHookEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- [`IUIAutomation::ElementFromPoint`](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-elementfrompoint)
- [UI Automation element properties](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids)

Important engineering constraints:

- A low-level hook is a global shared resource. Microsoft recommends restricting global hooks to special-purpose applications and removing them when no longer needed.
- The hook callback must return quickly. If it exceeds `LowLevelHooksTimeout`, Windows 7 and later can silently remove it.
- Passing the click through will also open the normal taskbar menu; swallowing it avoids two menus but changes Windows behavior. For a public Store build, prefer a distinct opt-in gesture and do not silently replace the native menu.
- UI Automation lookup and audio-session correlation must run outside the hook callback.

The more reliable variant is **Shift + right-click over an application's actual window**, because a normal window has a real HWND/PID. Taskbar targeting can be offered later as an experimental best-effort feature, with a fallback that asks the user to choose among currently audible apps.

## Surface 6: native volume / Quick Settings integration

Microsoft's published answer is direct: Windows 11 Quick Settings currently has no extension API. Adufa therefore cannot add its row to the native volume flyout through a supported SDK contract.

- [Microsoft Q&A: no Windows 11 Quick Settings extension API](https://learn.microsoft.com/en-us/answers/questions/1124942/api-for-adding-an-item-in-the-windows-11-taskbar-p)

### Supported alternatives

1. **Adufa rich flyout from its notification icon.** `Shell_NotifyIcon` and `Shell_NotifyIconGetRect` are supported Win32 APIs, and Microsoft's NotificationIcon sample explicitly demonstrates a rich flyout. This is the closest reliable visual and interaction match.
2. **Global keyboard popup.** `RegisterHotKey` is a documented system-wide shortcut API. Adufa can open the same selector near the pointer or focused application without touching the Shell.
3. **Open the native Volume mixer page.** Windows officially documents `ms-settings:apps-volume`. Adufa can expose “Open Windows volume mixer” as a secondary action while keeping its fast routing UI in its own flyout.

Sources:

- [Notifications and the Notification Area](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area)
- [Microsoft NotificationIcon sample](https://learn.microsoft.com/en-us/windows/win32/shell/samples-notificationicon)
- [`RegisterHotKey`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)
- [Launch Windows Settings (`ms-settings:apps-volume`)](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings)

## Invasive or undocumented approaches to reject

The following techniques could imitate exact shell integration, but they should not ship in Adufa:

- inject code or a DLL into `explorer.exe` / ShellExperienceHost;
- hook or patch private taskbar or Quick Settings implementation details;
- manipulate undocumented XAML islands, internal COM interfaces, or private window messages;
- impersonate another application's AppUserModelID and replace its Jump List.

These approaches have no stable compatibility contract, can break on cumulative Windows updates, increase crash and security exposure in the Shell, and create a substantial Microsoft Store certification risk. Store policy 10.2.8 requires supported methods and user consent for changes to Windows settings or experiences, and explicitly flags unsupported uses of accessibility or undocumented APIs.

- [Microsoft Store policy 10.2.8](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies#102-security)

## Recommendation for Adufa

Use one selector implementation and expose it through supported, consistent surfaces:

| Mockup surface | Adufa implementation | Status |
| --- | --- | --- |
| 1. Tray menu | Rich flyout anchored to Adufa's notification icon | Supported and reliable |
| 2. App shortcut | Opt-in Shift + right-click over an app window; taskbar correlation only as experimental best-effort | Supported approximation |
| 3. Global shortcut | `RegisterHotKey`, target app under cursor / foreground | Supported and reliable |
| 4. Cursor overlay | Adufa-owned popup near cursor | Supported and reliable |
| 6. Windows volume UI | Adufa-owned compact mixer plus “Open Windows volume mixer” deep link | Supported approximation |

For the public release, do **not** inject into Explorer or claim native Quick Settings integration. Ship surfaces 1, 3, and 4 first; add the window-targeted form of surface 2 behind an explicit setting; retain the taskbar-icon form as an experiment until it passes multiple Windows releases, languages, taskbar layouts, and Store certification review.
