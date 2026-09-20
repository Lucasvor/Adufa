//! Hallmark · pre-emit critique: P5 H5 E5 S5 R5 V5.
//! Hallmark native surface: compact, Windows-native and explicitly audio-first.
//! The layout follows the approved option-1 tray menu instead of a dashboard.

use std::borrow::Cow;
use std::mem::{size_of, size_of_val};
use std::time::Instant;

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_EXTENDED_FRAME_BOUNDS, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND, DwmGetWindowAttribute, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreatePen, CreateSolidBrush,
    DEFAULT_CHARSET, DEFAULT_PITCH, DT_END_ELLIPSIS, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER,
    DeleteObject, DrawTextW, Ellipse, EndPaint, FW_MEDIUM, FW_NORMAL, FW_SEMIBOLD, FillRect,
    GetMonitorInfoW, GetStockObject, HBRUSH, HDC, HGDIOBJ, InvalidateRect,
    MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, NULL_BRUSH, OUT_DEFAULT_PRECIS,
    PAINTSTRUCT, PS_SOLID, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem, GetDpiForWindow,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_DOWN, VK_ESCAPE,
    VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GA_ROOT, GWLP_USERDATA, GetAncestor, GetClassNameW, GetClientRect,
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowRect, IDC_ARROW, IsWindowVisible,
    KillTimer, LoadCursorW, MB_ICONERROR, MB_OK, MSG, MessageBoxW, PostMessageW, PostQuitMessage,
    RegisterClassW, SPI_GETWORKAREA, SW_HIDE, SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOZORDER,
    SetForegroundWindow, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    SystemParametersInfoW, TranslateMessage, WA_INACTIVE, WM_ACTIVATE, WM_CAPTURECHANGED, WM_CLOSE,
    WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE,
    WM_NCDESTROY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    WindowFromPoint,
};
use windows::core::{Error as WindowsError, PCWSTR, w};

use crate::audio_events::AudioChangeMonitor;
use crate::cursor_target;
use crate::locator::{LocatorState, MonitoredApplication, PeakMonitor};

use super::BoxError;
use super::hotkeys::{self, GlobalHotKey};
use super::i18n::{self, LanguageChoice, LanguagePreference, Locale, Text};
use super::icons::OwnedIcon;
use super::model::{PopupModel, RouteRequest, VolumeRequest};
use super::motion;
use super::selector;
use super::startup;
use super::symbols::{self, Symbol};
use super::taskbar_quick_access::{self, TaskbarQuickAccessMonitor};
use super::theme::Theme;
use super::tray::{self, TrayEvent, TrayIcon};

const WINDOW_CLASS: PCWSTR = w!("AdufaCompactPopup");
const WM_MOUSELEAVE: u32 = 0x02A3;

const WINDOW_WIDTH: i32 = 304;
const HEADER_HEIGHT: i32 = 64;
const CONTENT_TOP_PADDING: i32 = 8;
const APPLICATION_ROW_HEIGHT: i32 = 44;
const EMPTY_CONTENT_HEIGHT: i32 = 48;
const ACTION_ROW_HEIGHT: i32 = 38;
const BOTTOM_PADDING: i32 = 8;
const HEADER_ACTION_LEFT: i32 = 184;
const HEADER_ACTION_TOP: i32 = 12;
const HEADER_ACTION_BOTTOM: i32 = 52;
const SETTINGS_CONTENT_HEIGHT: i32 = 304;
const SETTINGS_STARTUP_TOP: i32 = 88;
const SETTINGS_STARTUP_BOTTOM: i32 = 144;
const SETTINGS_MIXER_TOP: i32 = 240;
const SETTINGS_MIXER_BOTTOM: i32 = 296;
const SETTINGS_LANGUAGE_TOP: i32 = 304;
const SETTINGS_LANGUAGE_BOTTOM: i32 = 360;
const LANGUAGE_ROW_HEIGHT: i32 = 32;
const LANGUAGE_CONTENT_TOP: i32 = HEADER_HEIGHT + 4;
const LANGUAGE_CHOICE_COUNT: usize = 9;
const DISMISS_TIMER_ID: usize = 1;
const DISMISS_DELAY_MS: u32 = 75;
const TASKBAR_COMPANION_TIMER_ID: usize = 2;
const TASKBAR_COMPANION_DELAY_MS: u32 = 220;
const TASKBAR_COMPANION_RETRY_MS: u32 = 80;
const TASKBAR_COMPANION_MAX_ATTEMPTS: u8 = 3;
const XAML_MENU_CONTENT_INSET: i32 = 12;

type RouteCallback = Box<dyn FnMut(RouteRequest) -> Result<(), String>>;
type VolumeCallback = Box<dyn FnMut(VolumeRequest) -> Result<(), String>>;
type RefreshCallback = Box<dyn FnMut() -> Result<PopupModel, String>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Target {
    HeaderLocator,
    Application(usize),
    Settings,
    Exit,
    Back,
    Startup,
    WindowsMixer,
    Language,
    Locale(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum View {
    Main,
    Settings,
    Languages,
}

#[derive(Clone, Copy, Debug)]
struct NativeTaskbarMenu {
    window: HWND,
    bounds: RECT,
}

struct WindowState {
    model: PopupModel,
    icons: Vec<Option<OwnedIcon>>,
    route: RouteCallback,
    set_volume: VolumeCallback,
    refresh: RefreshCallback,
    dpi: u32,
    hovered: Option<Target>,
    pressed: Option<Target>,
    tracking_mouse: bool,
    focus_index: usize,
    keyboard_focus_visible: bool,
    view: View,
    locator: Option<PeakMonitor>,
    locator_state: Option<LocatorState>,
    audio_events: Option<AudioChangeMonitor>,
    audio_discovery_error: Option<String>,
    quick_hotkey: Option<GlobalHotKey>,
    taskbar_quick_access_monitor: Option<TaskbarQuickAccessMonitor>,
    taskbar_pending_target: Option<taskbar_quick_access::TaskbarButtonTarget>,
    taskbar_companion_attempts: u8,
    taskbar_native_menu: Option<HWND>,
    startup_enabled: bool,
    language: LanguagePreference,
    /// Separate output-selector surface, owned by the selector module.
    selector_window: Option<HWND>,
    selector_application: Option<usize>,
    // Drop the shell registration before releasing the HICON it references.
    tray: Option<TrayIcon>,
    tray_icon: Option<OwnedIcon>,
}

impl WindowState {
    fn new(
        model: PopupModel,
        route: RouteCallback,
        set_volume: VolumeCallback,
        refresh: RefreshCallback,
        dpi: u32,
    ) -> Self {
        let icons = load_application_icons(&model, dpi);
        let language = LanguagePreference::load_default().unwrap_or_else(|error| {
            eprintln!("Language preference unavailable: {error}");
            LanguagePreference::automatic()
        });
        Self {
            model,
            icons,
            route,
            set_volume,
            refresh,
            dpi,
            hovered: None,
            pressed: None,
            tracking_mouse: false,
            focus_index: 0,
            keyboard_focus_visible: false,
            view: View::Main,
            locator: None,
            locator_state: None,
            audio_events: None,
            audio_discovery_error: None,
            quick_hotkey: None,
            taskbar_quick_access_monitor: None,
            taskbar_pending_target: None,
            taskbar_companion_attempts: 0,
            taskbar_native_menu: None,
            startup_enabled: startup::is_enabled().unwrap_or(false),
            language,
            selector_window: None,
            selector_application: None,
            tray: None,
            tray_icon: None,
        }
    }

    fn attach_tray(&mut self, window: HWND) -> Result<(), BoxError> {
        let icon = OwnedIcon::adufa_tray().ok_or("Windows could not create the Adufa tray icon")?;
        let tooltip = format!(
            "Adufa — {}",
            i18n::text(self.language.locale(), Text::AudioRouter)
        );
        let tray = TrayIcon::add(window, icon.handle(), &tooltip)?;
        self.tray = Some(tray);
        self.tray_icon = Some(icon);
        Ok(())
    }

    fn attach_audio_events(&mut self, window: HWND) -> Result<(), BoxError> {
        self.audio_events = Some(AudioChangeMonitor::start(window).map_err(std::io::Error::other)?);
        Ok(())
    }

    fn attach_quick_hotkey(&mut self, window: HWND) {
        match GlobalHotKey::register_quick_popup(window) {
            Ok(hotkey) => self.quick_hotkey = Some(hotkey),
            Err(error) => eprintln!("Quick popup shortcut unavailable: {error}"),
        }
    }

    fn attach_taskbar_quick_access(&mut self, window: HWND) {
        match TaskbarQuickAccessMonitor::start(window) {
            Ok(monitor) => self.taskbar_quick_access_monitor = Some(monitor),
            Err(error) => eprintln!("Taskbar quick access unavailable: {error}"),
        }
    }

    fn scale(&self, value: i32) -> i32 {
        value * self.dpi as i32 / 96
    }

    fn content_height(&self) -> i32 {
        if self.model.applications.is_empty() {
            EMPTY_CONTENT_HEIGHT
        } else {
            APPLICATION_ROW_HEIGHT * self.model.applications.len() as i32
        }
    }

    fn actions_top(&self) -> i32 {
        HEADER_HEIGHT + CONTENT_TOP_PADDING + self.content_height() + 1
    }

    fn logical_height(&self) -> i32 {
        match self.view {
            View::Main => self.actions_top() + ACTION_ROW_HEIGHT * 2 + BOTTOM_PADDING,
            View::Settings => HEADER_HEIGHT + SETTINGS_CONTENT_HEIGHT + BOTTOM_PADDING,
            View::Languages => {
                LANGUAGE_CONTENT_TOP
                    + LANGUAGE_ROW_HEIGHT * LANGUAGE_CHOICE_COUNT as i32
                    + BOTTOM_PADDING
            }
        }
    }

    fn target_at(&self, x: i32, y: i32) -> Option<Target> {
        hit_test_view_logical(x * 96 / self.dpi as i32, y * 96 / self.dpi as i32, self)
    }

    fn focused_target(&self) -> Option<Target> {
        target_for_view_navigation_index(self.focus_index, self)
    }

    fn navigation_count(&self) -> usize {
        match self.view {
            View::Main => self.model.applications.len() + 3,
            View::Settings => 4,
            View::Languages => LANGUAGE_CHOICE_COUNT + 1,
        }
    }

    const fn locale(&self) -> Locale {
        self.language.locale()
    }
}

impl Drop for WindowState {
    fn drop(&mut self) {
        if let Some(hotkey) = self.quick_hotkey.take() {
            if let Err(error) = hotkey.unregister() {
                eprintln!("Quick popup shortcut cleanup failed: {error}");
            }
        }
    }
}

fn load_application_icons(model: &PopupModel, dpi: u32) -> Vec<Option<OwnedIcon>> {
    model
        .applications
        .iter()
        .map(|application| {
            application
                .icon_path
                .as_deref()
                .and_then(|path| OwnedIcon::from_path_at_dpi(path, 24, dpi))
        })
        .collect()
}

pub fn run(
    model: PopupModel,
    route: RouteCallback,
    set_volume: VolumeCallback,
    refresh: RefreshCallback,
) -> Result<(), BoxError> {
    // SAFETY: Called before creating an HWND. Failure retains the process's valid
    // previous DPI mode and is therefore a safe fallback.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    // SAFETY: GetDpiForSystem has no pointer or lifetime preconditions.
    let dpi = unsafe { GetDpiForSystem() }.max(96);
    let state = Box::new(WindowState::new(model, route, set_volume, refresh, dpi));
    let width = state.scale(WINDOW_WIDTH);
    let height = state.scale(state.logical_height());

    // SAFETY: A null module name resolves the current executable module.
    let instance = unsafe { GetModuleHandleW(None)? };
    // SAFETY: The shared arrow cursor remains owned by Windows.
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance.into(),
        hCursor: cursor,
        lpszClassName: WINDOW_CLASS,
        ..Default::default()
    };
    // SAFETY: Callback and class-name addresses remain valid for process lifetime.
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(WindowsError::from_thread().into());
    }

    let (x, y) = popup_origin(width, height)?;
    let raw_state = Box::into_raw(state);
    // SAFETY: The HWND takes ownership of raw_state through WM_NCCREATE and returns
    // it exactly once in WM_NCDESTROY.
    let window = match unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            WINDOW_CLASS,
            w!("Adufa"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            Some(raw_state.cast()),
        )
    } {
        Ok(window) => window,
        Err(error) => {
            // SAFETY: CreateWindowExW failed before transferring ownership.
            drop(unsafe { Box::from_raw(raw_state) });
            return Err(error.into());
        }
    };

    // SAFETY: CreateWindowExW succeeded, so the HWND owns raw_state and the
    // pointer remains valid until WM_NCDESTROY.
    if let Err(error) = unsafe { (&mut *raw_state).attach_tray(window) } {
        // SAFETY: The current UI thread owns the newly created HWND.
        let _ = unsafe { DestroyWindow(window) };
        return Err(error);
    }
    if let Err(error) = unsafe { (&mut *raw_state).attach_audio_events(window) } {
        // SAFETY: The current UI thread owns the newly created HWND.
        let _ = unsafe { DestroyWindow(window) };
        return Err(error);
    }
    // A shortcut collision must not prevent tray routing from starting.
    unsafe { (&mut *raw_state).attach_quick_hotkey(window) };
    // Taskbar quick access is optional. If screen matching or the mouse hook
    // is unavailable, tray and keyboard routing remain fully functional.
    unsafe { (&mut *raw_state).attach_taskbar_quick_access(window) };

    apply_dwm_style(window);
    // SAFETY: window is a live top-level window owned by this UI thread.
    motion::show_popup(window);

    let mut message = MSG::default();
    loop {
        // SAFETY: message is writable and this thread owns the message loop.
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) };
        if result.0 == -1 {
            return Err(WindowsError::from_thread().into());
        }
        if !result.as_bool() {
            break;
        }
        // SAFETY: GetMessageW initialized the message for synchronous dispatch.
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

fn popup_origin(width: i32, height: i32) -> windows::core::Result<(i32, i32)> {
    let mut work_area = RECT::default();
    // SAFETY: SPI_GETWORKAREA writes one RECT and retains no pointer.
    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some((&mut work_area as *mut RECT).cast()),
            Default::default(),
        )?;
    }
    let margin = 16 * unsafe { GetDpiForSystem() }.max(96) as i32 / 96;
    Ok((
        work_area.right - width - margin,
        work_area.bottom - height - margin,
    ))
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: WM_NCCREATE guarantees CREATESTRUCTW in lparam.
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        let state = create.lpCreateParams.cast::<WindowState>();
        // SAFETY: The stable Box pointer fits LONG_PTR and lives to WM_NCDESTROY.
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, state as isize) };
    }

    // SAFETY: Zero before WM_NCCREATE, otherwise the owned WindowState pointer.
    let state_ptr = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) } as *mut WindowState;

    match message {
        value if hotkeys::is_quick_popup_hotkey(value, wparam) && !state_ptr.is_null() => {
            // SAFETY: Hotkey messages are serialized by this window's UI thread.
            show_quick_access(window, unsafe { &mut *state_ptr });
            LRESULT(0)
        }
        value if value == crate::audio_events::CHANGE_MESSAGE && !state_ptr.is_null() => {
            // SAFETY: The native callbacks post no pointers; observation runs on this UI thread.
            refresh_model(window, unsafe { &mut *state_ptr });
            LRESULT(0)
        }
        value if value == crate::locator::UPDATE_MESSAGE && !state_ptr.is_null() => {
            // SAFETY: The worker posts no pointers; state mutation stays on this UI thread.
            apply_locator_update(window, unsafe { &mut *state_ptr });
            LRESULT(0)
        }
        value if value == selector::ROUTE_MESSAGE && !state_ptr.is_null() => {
            // SAFETY: The selector sends integer indices to its owner on this UI thread.
            let state = unsafe { &mut *state_ptr };
            let accepted = route_by_choice(window, state, wparam.0, lparam.0 as usize);
            dismiss_native_taskbar_menu_after_route(state, accepted);
            LRESULT(isize::from(accepted))
        }
        value if value == selector::VOLUME_MESSAGE && !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            let (volume_percent, muted) = selector::decode_volume_message(lparam.0);
            let accepted = set_volume_by_value(window, state, wparam.0, volume_percent, muted);
            LRESULT(isize::from(accepted))
        }
        value if value == selector::CLOSED_MESSAGE && !state_ptr.is_null() => {
            // SAFETY: The selector sends its HWND value during synchronous teardown.
            let state = unsafe { &mut *state_ptr };
            selector_closed(state, HWND(lparam.0 as *mut core::ffi::c_void));
            invalidate(window);
            LRESULT(0)
        }
        value if value == taskbar_quick_access::RIGHT_CLICK_MESSAGE && !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            let icon_handles = state
                .icons
                .iter()
                .enumerate()
                .filter_map(|(index, icon)| icon.as_ref().map(|icon| (index, icon.handle())));
            if let Some(target) = taskbar_quick_access::take_pending_target(icon_handles) {
                state.taskbar_pending_target = Some(target);
                state.taskbar_companion_attempts = 0;
                let _ = unsafe {
                    SetTimer(
                        Some(window),
                        TASKBAR_COMPANION_TIMER_ID,
                        TASKBAR_COMPANION_DELAY_MS,
                        None,
                    )
                };
            }
            LRESULT(0)
        }
        value if value == tray::CALLBACK_MESSAGE && !state_ptr.is_null() => {
            if TrayIcon::event(lparam) == Some(TrayEvent::Activate) {
                toggle_from_tray(window, unsafe { &mut *state_ptr });
            }
            LRESULT(0)
        }
        WM_PAINT if !state_ptr.is_null() => {
            // SAFETY: Window messages are serialized on the owning UI thread.
            paint(window, unsafe { &*state_ptr });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_ACTIVATE if !state_ptr.is_null() => {
            if (wparam.0 as u32 & 0xffff) == WA_INACTIVE {
                // Deferring the check allows focus to move to Adufa's owned
                // selector without incorrectly dismissing the whole surface.
                unsafe { SetTimer(Some(window), DISMISS_TIMER_ID, DISMISS_DELAY_MS, None) };
            } else {
                let _ = unsafe { KillTimer(Some(window), DISMISS_TIMER_ID) };
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE if !state_ptr.is_null() => {
            // SAFETY: Serialized mutation of this HWND's owned state.
            let state = unsafe { &mut *state_ptr };
            state.keyboard_focus_visible = false;
            if !state.tracking_mouse {
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: window,
                    dwHoverTime: 0,
                };
                // SAFETY: tracking is fully initialized for this synchronous call.
                if unsafe { TrackMouseEvent(&mut tracking) }.is_ok() {
                    state.tracking_mouse = true;
                }
            }
            let hovered = state.target_at(point_x(lparam), point_y(lparam));
            if hovered != state.hovered {
                state.hovered = hovered;
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE if !state_ptr.is_null() => {
            // SAFETY: Serialized mutation of this HWND's owned state.
            let state = unsafe { &mut *state_ptr };
            state.tracking_mouse = false;
            state.hovered = None;
            invalidate(window);
            LRESULT(0)
        }
        WM_LBUTTONDOWN if !state_ptr.is_null() => {
            // SAFETY: Serialized mutation of this HWND's owned state.
            let state = unsafe { &mut *state_ptr };
            state.keyboard_focus_visible = false;
            if let Some(target) = state.target_at(point_x(lparam), point_y(lparam)) {
                if let Some(index) = navigation_index_for_target(target, state) {
                    state.focus_index = index;
                }
                state.pressed = Some(target);
                let _ = unsafe { SetCapture(window) };
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP if !state_ptr.is_null() => {
            // SAFETY: Serialized mutation of this HWND's owned state.
            let state = unsafe { &mut *state_ptr };
            let released_over = state.target_at(point_x(lparam), point_y(lparam));
            let pressed = state.pressed.take();
            let _ = unsafe { ReleaseCapture() };
            if let Some(target) = pressed.filter(|target| Some(*target) == released_over) {
                activate_target(window, state, target);
            }
            invalidate(window);
            LRESULT(0)
        }
        WM_CAPTURECHANGED if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            if state.pressed.take().is_some() {
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_TIMER if !state_ptr.is_null() && wparam.0 == TASKBAR_COMPANION_TIMER_ID => {
            let _ = unsafe { KillTimer(Some(window), TASKBAR_COMPANION_TIMER_ID) };
            let state = unsafe { &mut *state_ptr };
            if let Some(target) = state.taskbar_pending_target.take() {
                if let Some(native_menu) = native_taskbar_menu(target.bounds) {
                    show_taskbar_quick_access(window, state, target, native_menu);
                } else if state.taskbar_companion_attempts < TASKBAR_COMPANION_MAX_ATTEMPTS {
                    state.taskbar_companion_attempts += 1;
                    state.taskbar_pending_target = Some(target);
                    let _ = unsafe {
                        SetTimer(
                            Some(window),
                            TASKBAR_COMPANION_TIMER_ID,
                            TASKBAR_COMPANION_RETRY_MS,
                            None,
                        )
                    };
                }
            }
            LRESULT(0)
        }
        WM_TIMER if !state_ptr.is_null() && wparam.0 == DISMISS_TIMER_ID => {
            // SAFETY: This one-shot timer belongs to the current UI thread.
            let _ = unsafe { KillTimer(Some(window), DISMISS_TIMER_ID) };
            let state = unsafe { &mut *state_ptr };
            let foreground = unsafe { GetForegroundWindow() };
            if foreground != window && state.selector_window != Some(foreground) {
                hide_popup(window, state);
            }
            LRESULT(0)
        }
        WM_KEYDOWN if !state_ptr.is_null() => {
            // SAFETY: Serialized mutation of this HWND's owned state.
            handle_key(window, unsafe { &mut *state_ptr }, wparam);
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: Ends the message loop owned by this thread.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_NCDESTROY if !state_ptr.is_null() => {
            // SAFETY: Clear userdata before recovering the Box exactly once.
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, 0) };
            drop(unsafe { Box::from_raw(state_ptr) });
            // SAFETY: The HWND remains valid for default non-client teardown.
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        _ => {
            // SAFETY: Windows handles all messages not owned by this component.
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
    }
}

fn handle_key(window: HWND, state: &mut WindowState, key: WPARAM) {
    state.keyboard_focus_visible = true;
    match key.0 as u16 {
        value if value == VK_ESCAPE.0 => {
            match state.view {
                View::Main => {
                    // Escape dismisses the popup; the tray icon keeps the app alive.
                    hide_popup(window, state);
                }
                View::Settings => show_main_view(window, state),
                View::Languages => show_settings_view(window, state),
            }
        }
        value if value == VK_TAB.0 || value == VK_DOWN.0 => {
            state.focus_index = (state.focus_index + 1) % state.navigation_count();
            invalidate(window);
        }
        value if value == VK_UP.0 => {
            state.focus_index = state
                .focus_index
                .checked_sub(1)
                .unwrap_or(state.navigation_count() - 1);
            invalidate(window);
        }
        value if value == VK_RETURN.0 || value == VK_SPACE.0 => {
            if let Some(target) = state.focused_target() {
                activate_target(window, state, target);
                invalidate(window);
            }
        }
        value if value == VK_RIGHT.0 => {
            if let Some(Target::Application(index)) = state.focused_target() {
                open_selector(window, state, index);
            }
        }
        _ => {}
    }
}

fn toggle_from_tray(window: HWND, state: &mut WindowState) {
    // SAFETY: The callback originates from this live HWND.
    if unsafe { IsWindowVisible(window) }.as_bool() {
        hide_popup(window, state);
        return;
    }

    refresh_model(window, state);
    state.keyboard_focus_visible = false;

    let width = state.scale(WINDOW_WIDTH);
    let height = state.scale(state.logical_height());
    if let Ok((x, y)) = popup_origin(width, height) {
        // Re-anchor on every activation in case the taskbar or monitor changed.
        let _ = unsafe {
            SetWindowPos(
                window,
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
    motion::show_popup(window);
}

/// Opens the shared compact surface from the global shortcut.
///
/// When the cursor points at a currently audible application, the existing
/// adjacent output selector opens immediately for that row. Otherwise the same
/// shortcut remains a general application picker near the cursor.
fn show_quick_access(window: HWND, state: &mut WindowState) {
    let cursor_target = cursor_target::resolve();
    close_selector(state);
    state.view = View::Main;
    refresh_model(window, state);

    let application_index = cursor_target.as_ref().and_then(|target| {
        cursor_target::application_index_for_process(
            target.process_id,
            state
                .model
                .applications
                .iter()
                .map(|application| application.process_ids.as_slice()),
        )
    });
    state.focus_index = application_index.unwrap_or(0);
    state.keyboard_focus_visible = true;
    state.hovered = None;

    let width = state.scale(WINDOW_WIDTH);
    let height = state.scale(state.logical_height());
    let (x, y) = cursor_target
        .map(|target| quick_popup_origin(target.cursor_position, width, height, state.dpi))
        .or_else(|| popup_origin(width, height).ok())
        .unwrap_or((0, 0));
    // SAFETY: The current UI thread owns this live top-level window.
    unsafe {
        let _ = SetWindowPos(
            window,
            None,
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    motion::show_popup(window);
    invalidate(window);

    if let Some(index) = application_index {
        open_selector(window, state, index);
    }
}

/// Opens output choices beside Explorer's native menu for the audible taskbar
/// button whose right-click Adufa observed.
fn show_taskbar_quick_access(
    window: HWND,
    state: &mut WindowState,
    target: taskbar_quick_access::TaskbarButtonTarget,
    native_menu: NativeTaskbarMenu,
) {
    close_selector(state);
    state.taskbar_native_menu = Some(native_menu.window);
    state.view = View::Main;
    let application_index = target.application_index;
    if application_index >= state.model.applications.len() {
        return;
    }

    state.focus_index = application_index + 1;
    state.keyboard_focus_visible = true;
    state.hovered = None;
    stop_locator(state);
    let _ = unsafe { ShowWindow(window, SW_HIDE) };

    if !open_selector_at_anchor(
        window,
        state,
        application_index,
        native_menu.bounds,
        selector::Placement::TaskbarCompanion,
    ) {
        state.taskbar_native_menu = None;
        invalidate(window);
    }
}

fn native_taskbar_menu(taskbar_button: RECT) -> Option<NativeTaskbarMenu> {
    let center = POINT {
        x: taskbar_button.left + (taskbar_button.right - taskbar_button.left) / 2,
        y: taskbar_button.top + (taskbar_button.bottom - taskbar_button.top) / 2,
    };
    let work = monitor_work_area(center);
    let probe = taskbar_menu_probe(taskbar_button, work);
    let window = unsafe { WindowFromPoint(probe) };
    let root = unsafe { GetAncestor(window, GA_ROOT) };
    if root.0.is_null() {
        return None;
    }

    let mut class_name = [0_u16; 64];
    let length = unsafe { GetClassNameW(root, &mut class_name) };
    if length <= 0 {
        return None;
    }
    let class_name = String::from_utf16_lossy(&class_name[..length as usize]);
    if !is_native_taskbar_menu_class(&class_name) {
        return None;
    }

    let mut shadow_bounds = RECT::default();
    if unsafe { GetWindowRect(root, &mut shadow_bounds) }.is_err() {
        return None;
    }
    let mut extended_bounds = RECT::default();
    let extended_bounds = unsafe {
        DwmGetWindowAttribute(
            root,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&raw mut extended_bounds).cast(),
            size_of::<RECT>() as u32,
        )
    }
    .is_ok()
    .then_some(extended_bounds);
    let bounds = preferred_native_menu_bounds(extended_bounds, shadow_bounds);
    let dpi = unsafe { GetDpiForWindow(root) }.max(96);
    let bounds = content_bounds_for_native_menu(&class_name, bounds, dpi);
    let width = bounds.right - bounds.left;
    let height = bounds.bottom - bounds.top;
    (width > 0 && width <= 800 && height > 0 && height <= 900).then_some(NativeTaskbarMenu {
        window: root,
        bounds,
    })
}

fn preferred_native_menu_bounds(extended_bounds: Option<RECT>, shadow_bounds: RECT) -> RECT {
    extended_bounds
        .filter(|bounds| bounds.right > bounds.left && bounds.bottom > bounds.top)
        .unwrap_or(shadow_bounds)
}

fn content_bounds_for_native_menu(class_name: &str, host_bounds: RECT, dpi: u32) -> RECT {
    if !matches!(
        class_name,
        "Windows.UI.Core.CoreWindow" | "Xaml_WindowedPopupClass"
    ) {
        return host_bounds;
    }

    let inset = XAML_MENU_CONTENT_INSET * dpi.max(96) as i32 / 96;
    let content_bounds = RECT {
        left: host_bounds.left + inset,
        top: host_bounds.top + inset,
        right: host_bounds.right - inset,
        bottom: host_bounds.bottom - inset,
    };
    if content_bounds.right > content_bounds.left && content_bounds.bottom > content_bounds.top {
        content_bounds
    } else {
        host_bounds
    }
}

fn taskbar_menu_probe(taskbar_button: RECT, work: RECT) -> POINT {
    let center_x = taskbar_button.left + (taskbar_button.right - taskbar_button.left) / 2;
    let center_y = taskbar_button.top + (taskbar_button.bottom - taskbar_button.top) / 2;
    if taskbar_button.top >= work.bottom {
        POINT {
            x: center_x,
            y: taskbar_button.top - 1,
        }
    } else if taskbar_button.bottom <= work.top {
        POINT {
            x: center_x,
            y: taskbar_button.bottom,
        }
    } else if taskbar_button.left >= work.right {
        POINT {
            x: taskbar_button.left - 1,
            y: center_y,
        }
    } else {
        POINT {
            x: taskbar_button.right,
            y: center_y,
        }
    }
}

fn is_native_taskbar_menu_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "Windows.UI.Core.CoreWindow" | "Xaml_WindowedPopupClass" | "#32768"
    )
}

fn quick_popup_origin(point: POINT, width: i32, height: i32, dpi: u32) -> (i32, i32) {
    let work_area = monitor_work_area(point);
    let gap = 12 * dpi.max(96) as i32 / 96;
    clamp_popup_to_work_area(point, width, height, gap, work_area)
}

fn clamp_popup_to_work_area(
    point: POINT,
    width: i32,
    height: i32,
    gap: i32,
    work_area: RECT,
) -> (i32, i32) {
    let preferred_x = if point.x + gap + width <= work_area.right {
        point.x + gap
    } else {
        point.x - gap - width
    };
    let preferred_y = if point.y + gap + height <= work_area.bottom {
        point.y + gap
    } else {
        point.y - gap - height
    };
    let maximum_x = (work_area.right - width).max(work_area.left);
    let maximum_y = (work_area.bottom - height).max(work_area.top);
    (
        preferred_x.clamp(work_area.left, maximum_x),
        preferred_y.clamp(work_area.top, maximum_y),
    )
}

fn monitor_work_area(point: POINT) -> RECT {
    // SAFETY: A POINT is passed by value; Windows returns the nearest monitor handle.
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` declares its size and points to valid writable storage.
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return info.rcWork;
    }

    let mut work_area = RECT::default();
    // SAFETY: SPI_GETWORKAREA writes exactly one RECT and retains no pointer.
    if unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some((&mut work_area as *mut RECT).cast()),
            Default::default(),
        )
    }
    .is_ok()
    {
        work_area
    } else {
        RECT {
            left: point.x,
            top: point.y,
            right: point.x + 1,
            bottom: point.y + 1,
        }
    }
}

fn activate_target(window: HWND, state: &mut WindowState, target: Target) {
    match target {
        Target::Application(index) => open_selector(window, state, index),
        Target::HeaderLocator => toggle_locator(window, state),
        Target::Settings => show_settings_view(window, state),
        Target::Exit => {
            // SAFETY: The current UI thread owns this live HWND.
            let _ = unsafe { DestroyWindow(window) };
        }
        Target::Back => match state.view {
            View::Languages => show_settings_view(window, state),
            _ => show_main_view(window, state),
        },
        Target::Startup => toggle_startup(window, state),
        Target::WindowsMixer => open_windows_volume_mixer(window, state.locale()),
        Target::Language => show_languages_view(window, state),
        Target::Locale(index) => select_language(window, state, index),
    }
}

fn refresh_model(window: HWND, state: &mut WindowState) {
    let restart_locator = state.locator.is_some();
    if restart_locator {
        stop_locator(state);
    }
    match (state.refresh)() {
        Ok(refreshed) => {
            state.audio_discovery_error = None;
            state.icons = load_application_icons(&refreshed, state.dpi);
            state.model = refreshed;
            state.hovered = None;
            state.focus_index = state
                .focus_index
                .min(state.navigation_count().saturating_sub(1));
            if restart_locator {
                if let Err(error) = start_locator(window, state) {
                    show_operation_error(
                        window,
                        i18n::text(state.locale(), Text::LocatorFailed),
                        &error,
                    );
                }
            }
            resize_for_view(window, state);
        }
        Err(error) => {
            if restart_locator {
                let _ = start_locator(window, state);
            }
            invalidate(window);
            if should_present_discovery_error(&mut state.audio_discovery_error, &error) {
                show_operation_error(
                    window,
                    i18n::text(state.locale(), Text::DiscoveryFailed),
                    &error,
                );
            }
        }
    }
}

fn should_present_discovery_error(active_error: &mut Option<String>, error: &str) -> bool {
    if active_error.as_deref() == Some(error) {
        return false;
    }
    *active_error = Some(error.to_owned());
    true
}

fn toggle_locator(window: HWND, state: &mut WindowState) {
    if state.locator.is_some() {
        stop_locator(state);
        invalidate(window);
        return;
    }

    if let Err(error) = start_locator(window, state) {
        show_operation_error(
            window,
            i18n::text(state.locale(), Text::LocatorFailed),
            &error,
        );
    } else {
        invalidate(window);
    }
}

fn start_locator(window: HWND, state: &mut WindowState) -> Result<(), String> {
    let applications = state
        .model
        .applications
        .iter()
        .map(|application| MonitoredApplication {
            is_system_sounds: application.application_id == "windows:system-sounds",
            process_ids: application.process_ids.clone(),
        })
        .collect();
    let locator = PeakMonitor::start(window, applications)?;
    state.locator_state = Some(LocatorState::new(state.model.applications.len()));
    state.locator = Some(locator);
    Ok(())
}

fn stop_locator(state: &mut WindowState) {
    if let Some(mut locator) = state.locator.take() {
        locator.stop();
    }
    state.locator_state = None;
}

fn hide_popup(window: HWND, state: &mut WindowState) {
    close_selector(state);
    stop_locator(state);
    let _ = unsafe { ShowWindow(window, SW_HIDE) };
}

fn apply_locator_update(window: HWND, state: &mut WindowState) {
    let Some(snapshot) = state.locator.as_ref().and_then(PeakMonitor::take_latest) else {
        return;
    };
    let (previous_strongest, previous_listening, strongest, listening, mut changed_rows) = {
        let Some(locator_state) = state.locator_state.as_mut() else {
            return;
        };
        let previous_strongest = locator_state.strongest();
        let previous_listening = locator_state.is_listening();
        let changed_rows = locator_state.update(&snapshot.levels, Instant::now());
        (
            previous_strongest,
            previous_listening,
            locator_state.strongest(),
            locator_state.is_listening(),
            changed_rows,
        )
    };
    if previous_strongest != strongest {
        if let Some(index) = previous_strongest {
            changed_rows.push(index);
        }
        if let Some(index) = strongest {
            changed_rows.push(index);
        }
    }
    changed_rows.sort_unstable();
    changed_rows.dedup();
    for index in changed_rows {
        invalidate_application_row(window, state, index);
    }
    if previous_listening != listening {
        invalidate_header(window, state);
    }
}

fn toggle_startup(window: HWND, state: &mut WindowState) {
    let enabled = !state.startup_enabled;
    match startup::set_enabled(enabled) {
        Ok(()) => {
            state.startup_enabled = enabled;
            invalidate(window);
        }
        Err(error) => show_operation_error(
            window,
            i18n::text(state.locale(), Text::StartupFailed),
            &error,
        ),
    }
}

fn open_windows_volume_mixer(window: HWND, locale: Locale) {
    let result = unsafe {
        ShellExecuteW(
            Some(window),
            w!("open"),
            w!("ms-settings:apps-volume"),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as usize <= 32 {
        show_operation_error(
            window,
            i18n::text(locale, Text::WindowsMixer),
            i18n::text(locale, Text::MixerOpenFailed),
        );
    }
}

/// Opens the selector without replacing the main popup's content.
///
/// This intentionally small seam is replaced by `selector::open` when the
/// sibling selector module is connected. Keeping the HWND in `WindowState`
/// lets the parent window focus an existing selector instead of opening two.
fn open_selector(window: HWND, state: &mut WindowState, application_index: usize) {
    let mut owner_rect = RECT::default();
    // SAFETY: `window` is the live owner and `owner_rect` is writable.
    if unsafe { GetWindowRect(window, &mut owner_rect) }.is_err() {
        return;
    }

    let row_top = state.scale(
        HEADER_HEIGHT + CONTENT_TOP_PADDING + APPLICATION_ROW_HEIGHT * application_index as i32,
    );
    let row_bottom = row_top + state.scale(APPLICATION_ROW_HEIGHT);
    let anchor = RECT {
        left: owner_rect.left,
        top: owner_rect.top + row_top,
        right: owner_rect.right,
        bottom: owner_rect.top + row_bottom,
    };
    let _ = open_selector_at_anchor(
        window,
        state,
        application_index,
        anchor,
        selector::Placement::Adjacent,
    );
}

fn open_selector_at_anchor(
    window: HWND,
    state: &mut WindowState,
    application_index: usize,
    anchor: RECT,
    placement: selector::Placement,
) -> bool {
    let Some(application) = state.model.applications.get(application_index) else {
        return false;
    };
    let application_name = localized_application_name(state, application).into_owned();
    let application_icon_path = application.icon_path.clone();
    let selected_output_id = application.selected_output_id.clone();
    let volume_percent = application.volume_percent;
    let muted = application.muted;

    if let Some(selector_window) = state.selector_window {
        if state.selector_application != Some(application_index) {
            state.selector_window = None;
            state.selector_application = None;
            // SAFETY: This UI thread owns the selector HWND.
            let _ = unsafe { DestroyWindow(selector_window) };
        } else {
            // SAFETY: The selector module supplies a top-level HWND owned by this UI thread.
            let _ = unsafe { SetForegroundWindow(selector_window) };
            return true;
        }
    }

    let outputs = state
        .model
        .outputs
        .iter()
        .map(|output| (output.id.clone(), output.name.clone()))
        .collect();

    match selector::show(
        window,
        selector::SelectorRequest {
            application_index,
            application_name,
            application_icon_path,
            volume_percent,
            muted,
            locale: state.locale(),
            outputs,
            selected_output_id,
        },
        state.dpi,
        anchor,
        placement,
    ) {
        Ok(selector_window) => {
            state.selector_window = Some(selector_window);
            state.selector_application = Some(application_index);
            invalidate(window);
            true
        }
        Err(error) => {
            show_error(window, state.locale(), &error.to_string());
            false
        }
    }
}

/// Applies one selector row immediately. Choice zero means "System default";
/// every later choice maps to the output at `choice_index - 1`.
fn route_by_choice(
    window: HWND,
    state: &mut WindowState,
    application_index: usize,
    choice_index: usize,
) -> bool {
    let output_id = if choice_index == 0 {
        None
    } else {
        let Some(output) = state.model.outputs.get(choice_index - 1) else {
            return false;
        };
        Some(output.id.clone())
    };
    let Some(request) = state
        .model
        .route_request(application_index, output_id.clone())
    else {
        return false;
    };
    match (state.route)(request) {
        Ok(()) => {
            if let Some(application) = state.model.applications.get_mut(application_index) {
                application.selected_output_id = output_id;
            }
            invalidate(window);
            true
        }
        Err(error) => {
            show_error(window, state.locale(), &error);
            false
        }
    }
}

fn set_volume_by_value(
    window: HWND,
    state: &mut WindowState,
    application_index: usize,
    volume_percent: u8,
    muted: bool,
) -> bool {
    let Some(request) = state
        .model
        .volume_request(application_index, volume_percent, muted)
    else {
        return false;
    };
    match (state.set_volume)(request) {
        Ok(()) => {
            if let Some(application) = state.model.applications.get_mut(application_index) {
                application.volume_percent = volume_percent.min(100);
                application.muted = muted;
            }
            true
        }
        Err(error) => {
            show_error(window, state.locale(), &error);
            false
        }
    }
}

fn selector_closed(state: &mut WindowState, selector_window: HWND) {
    if state.selector_window == Some(selector_window) {
        state.selector_window = None;
        state.selector_application = None;
        state.taskbar_native_menu = None;
    }
}

fn take_native_menu_after_route(native_menu: &mut Option<HWND>, accepted: bool) -> Option<HWND> {
    accepted.then(|| native_menu.take()).flatten()
}

fn dismiss_native_taskbar_menu_after_route(state: &mut WindowState, accepted: bool) {
    let Some(native_menu) = take_native_menu_after_route(&mut state.taskbar_native_menu, accepted)
    else {
        return;
    };
    request_native_menu_close(native_menu);
}

fn request_native_menu_close(native_menu: HWND) {
    // SAFETY: The detected HWND is used only as an asynchronous message target.
    let _ = unsafe { PostMessageW(Some(native_menu), WM_CLOSE, WPARAM(0), LPARAM(0)) };
}

fn close_selector(state: &mut WindowState) {
    if let Some(selector_window) = state.selector_window.take() {
        state.selector_application = None;
        // SAFETY: This UI thread owns the selector HWND.
        let _ = unsafe { DestroyWindow(selector_window) };
    }
}

fn show_settings_view(window: HWND, state: &mut WindowState) {
    close_selector(state);
    stop_locator(state);
    state.startup_enabled = startup::is_enabled().unwrap_or(state.startup_enabled);
    state.view = View::Settings;
    state.focus_index = 0;
    state.hovered = None;
    resize_for_view(window, state);
}

fn show_main_view(window: HWND, state: &mut WindowState) {
    state.view = View::Main;
    state.focus_index = 0;
    state.hovered = None;
    resize_for_view(window, state);
}

fn show_languages_view(window: HWND, state: &mut WindowState) {
    state.view = View::Languages;
    state.focus_index = state.language.choice().list_index() + 1;
    state.hovered = None;
    resize_for_view(window, state);
}

fn select_language(window: HWND, state: &mut WindowState, index: usize) {
    let Some(choice) = LanguageChoice::from_list_index(index) else {
        return;
    };
    if let Err(error) = state.language.set(choice) {
        show_operation_error(
            window,
            i18n::text(state.locale(), Text::LanguageFailed),
            &error,
        );
        return;
    }
    let tooltip = format!("Adufa — {}", i18n::text(state.locale(), Text::AudioRouter));
    if let Some(tray) = state.tray.as_mut() {
        let _ = tray.set_tooltip(&tooltip);
    }
    invalidate(window);
}

fn resize_for_view(window: HWND, state: &WindowState) {
    let mut window_rect = RECT::default();
    // SAFETY: window is live and window_rect is writable for the call.
    if unsafe { GetWindowRect(window, &mut window_rect) }.is_err() {
        invalidate(window);
        return;
    }
    let height = state.scale(state.logical_height());
    let width = state.scale(WINDOW_WIDTH);
    // Keep the popup anchored to the same bottom edge while its subview changes height.
    unsafe {
        let _ = SetWindowPos(
            window,
            None,
            window_rect.left,
            window_rect.bottom - height,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    invalidate(window);
}

fn show_error(window: HWND, locale: Locale, error: &str) {
    let message = format!("{}\n\n{error}", i18n::text(locale, Text::RouteChangeFailed));
    show_message(
        window,
        &message,
        i18n::text(locale, Text::RoutingFailed),
        MB_OK | MB_ICONERROR,
    );
}

fn show_operation_error(window: HWND, caption: &str, error: &str) {
    show_message(window, error, caption, MB_OK | MB_ICONERROR);
}

fn show_message(
    window: HWND,
    message: &str,
    caption: &str,
    style: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
) {
    let message = wide(message);
    let caption = wide(caption);
    // SAFETY: Both strings are null-terminated and remain alive for the modal call.
    unsafe {
        let _ = MessageBoxW(
            Some(window),
            PCWSTR(message.as_ptr()),
            PCWSTR(caption.as_ptr()),
            style,
        );
    }
}

fn apply_dwm_style(window: HWND) {
    let rounded = DWMWCP_ROUND;
    let dark = 1_i32;
    // SAFETY: Values have the exact size/type required by their DWM attributes.
    unsafe {
        let _ = DwmSetWindowAttribute(
            window,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&rounded as *const windows::Win32::Graphics::Dwm::DWM_WINDOW_CORNER_PREFERENCE).cast(),
            size_of_val(&rounded) as u32,
        );
        let _ = DwmSetWindowAttribute(
            window,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const i32).cast(),
            size_of_val(&dark) as u32,
        );
    }
}

fn paint(window: HWND, state: &WindowState) {
    let mut paint = PAINTSTRUCT::default();
    // SAFETY: BeginPaint/EndPaint are paired for this WM_PAINT.
    let dc = unsafe { BeginPaint(window, &mut paint) };
    let mut client = RECT::default();
    // SAFETY: window is live and client points to writable storage.
    if unsafe { GetClientRect(window, &mut client) }.is_err() {
        // SAFETY: Completes the paint cycle begun above.
        let _ = unsafe { EndPaint(window, &paint) };
        return;
    }

    let theme = Theme::DARK;
    fill(dc, client, theme.surface);
    // SAFETY: The DC is valid for this paint cycle.
    unsafe { SetBkMode(dc, TRANSPARENT) };

    match state.view {
        View::Main => {
            draw_header(dc, state, theme);
            draw_content(dc, state, theme);
            draw_actions(dc, state, theme);
        }
        View::Settings => draw_settings(dc, state, theme),
        View::Languages => draw_languages(dc, state, theme),
    }
    draw_panel_border(dc, state, client, theme);

    // SAFETY: Completes and releases this paint DC.
    let _ = unsafe { EndPaint(window, &paint) };
}

fn draw_header(dc: HDC, state: &WindowState, theme: Theme) {
    symbols::draw(
        dc,
        Symbol::Volume,
        logical_rect(state, 10, 14, 38, 50),
        20,
        theme.text,
        state.dpi,
    );
    draw_text(
        dc,
        "Adufa",
        logical_rect(state, 46, 9, 178, 32),
        FontRole::Brand,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::AudioRouter),
        logical_rect(state, 46, 31, 178, 55),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    let locator_active = state.locator.is_some();
    let locator_rect = logical_rect(
        state,
        HEADER_ACTION_LEFT,
        HEADER_ACTION_TOP,
        296,
        HEADER_ACTION_BOTTOM,
    );
    if locator_active {
        fill_rounded(dc, state, locator_rect, 6, theme.accent_soft);
    }
    if state.hovered == Some(Target::HeaderLocator) {
        fill_rounded(dc, state, locator_rect, 6, theme.hover);
    }
    draw_text(
        dc,
        if locator_active {
            i18n::text(state.locale(), Text::Listening)
        } else {
            i18n::text(state.locale(), Text::FindSound)
        },
        logical_rect(state, 214, HEADER_ACTION_TOP, 288, HEADER_ACTION_BOTTOM),
        FontRole::Caption,
        if locator_active {
            theme.accent
        } else {
            theme.secondary_text
        },
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    symbols::draw(
        dc,
        Symbol::SoundLocator,
        logical_rect(state, 188, HEADER_ACTION_TOP, 214, HEADER_ACTION_BOTTOM),
        17,
        if locator_active {
            theme.accent
        } else {
            theme.secondary_text
        },
        state.dpi,
    );
    fill(
        dc,
        logical_rect(state, 16, HEADER_HEIGHT - 1, 288, HEADER_HEIGHT),
        theme.rule,
    );
}

fn draw_content(dc: HDC, state: &WindowState, theme: Theme) {
    if state.model.applications.is_empty() {
        draw_text(
            dc,
            i18n::text(state.locale(), Text::NoApplications),
            logical_rect(
                state,
                16,
                HEADER_HEIGHT + CONTENT_TOP_PADDING,
                288,
                HEADER_HEIGHT + CONTENT_TOP_PADDING + EMPTY_CONTENT_HEIGHT,
            ),
            FontRole::Caption,
            theme.secondary_text,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            state,
        );
        return;
    }

    for (index, application) in state.model.applications.iter().enumerate() {
        let target = Target::Application(index);
        let top = HEADER_HEIGHT + CONTENT_TOP_PADDING + APPLICATION_ROW_HEIGHT * index as i32;
        let row = logical_rect(state, 8, top, 296, top + APPLICATION_ROW_HEIGHT);
        let is_strongest = state
            .locator_state
            .as_ref()
            .is_some_and(|locator| locator.strongest() == Some(index));
        if is_strongest {
            fill_rounded(dc, state, row, 4, theme.accent_soft);
        }
        draw_interaction_surface(dc, state, theme, target, row);
        if is_strongest {
            fill_rounded(
                dc,
                state,
                logical_rect(state, 8, top + 5, 10, top + APPLICATION_ROW_HEIGHT - 5),
                1,
                theme.accent,
            );
        }

        let icon_x = state.scale(16);
        let icon_y = state.scale(top + 10);
        if let Some(Some(icon)) = state.icons.get(index) {
            icon.draw(dc, icon_x, icon_y, state.scale(24));
        } else {
            draw_fallback_app_icon(dc, state, theme, 16, top + 10, 24);
        }

        let (primary_text_top, primary_text_bottom) = application_primary_text_bounds(top);

        let application_name = localized_application_name(state, application);
        draw_text(
            dc,
            &application_name,
            logical_rect(state, 50, primary_text_top, 158, primary_text_bottom),
            FontRole::AppName,
            theme.text,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            state,
        );
        let destination = localized_destination_name(state, index);
        draw_text(
            dc,
            &destination,
            logical_rect(state, 160, primary_text_top, 276, primary_text_bottom),
            FontRole::Caption,
            theme.secondary_text,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            state,
        );
        symbols::draw(
            dc,
            Symbol::ChevronRight,
            logical_rect(state, 278, top + 10, 296, top + 34),
            11,
            theme.secondary_text,
            state.dpi,
        );
        if let Some(locator) = state.locator_state.as_ref() {
            let meter = logical_rect(state, 50, top + 37, 132, top + 41);
            fill_rounded(dc, state, meter, 3, theme.meter_track);
            let level = locator.levels().get(index).copied().unwrap_or_default();
            let meter_right = 50 + (82.0 * level).round() as i32;
            if meter_right > 50 {
                fill_rounded(
                    dc,
                    state,
                    logical_rect(state, 50, top + 37, meter_right, top + 41),
                    3,
                    theme.accent,
                );
            }
        }
    }
}

fn localized_application_name<'a>(
    state: &WindowState,
    application: &'a super::model::ApplicationRow,
) -> Cow<'a, str> {
    if application.application_id == "windows:system-sounds" {
        Cow::Owned(i18n::text(state.locale(), Text::SystemSounds).to_owned())
    } else if application.name == "Application" {
        Cow::Owned(i18n::text(state.locale(), Text::Application).to_owned())
    } else {
        Cow::Borrowed(application.name.as_str())
    }
}

fn localized_destination_name<'a>(state: &'a WindowState, index: usize) -> Cow<'a, str> {
    match state.model.destination_name(index) {
        "System default" => Cow::Owned(i18n::text(state.locale(), Text::SystemDefault).to_owned()),
        "Unavailable output" => {
            Cow::Owned(i18n::text(state.locale(), Text::UnavailableOutput).to_owned())
        }
        name => Cow::Borrowed(name),
    }
}

/// Keeps app and destination labels centered with the 24-DIP icon in every mode.
const fn application_primary_text_bounds(row_top: i32) -> (i32, i32) {
    (row_top, row_top + APPLICATION_ROW_HEIGHT)
}

fn draw_actions(dc: HDC, state: &WindowState, theme: Theme) {
    let top = state.actions_top();
    fill(dc, logical_rect(state, 16, top - 1, 288, top), theme.rule);
    draw_action_row(
        dc,
        state,
        theme,
        Target::Settings,
        top,
        i18n::text(state.locale(), Text::Settings),
    );
    draw_action_row(
        dc,
        state,
        theme,
        Target::Exit,
        top + ACTION_ROW_HEIGHT,
        i18n::text(state.locale(), Text::Exit),
    );
}

fn draw_action_row(
    dc: HDC,
    state: &WindowState,
    theme: Theme,
    target: Target,
    top: i32,
    label: &str,
) {
    let row = logical_rect(state, 8, top, 296, top + ACTION_ROW_HEIGHT);
    draw_interaction_surface(dc, state, theme, target, row);
    match target {
        Target::Settings => symbols::draw(
            dc,
            Symbol::Settings,
            logical_rect(state, 10, top + 7, 34, top + 31),
            17,
            theme.secondary_text,
            state.dpi,
        ),
        Target::Exit => symbols::draw(
            dc,
            Symbol::Power,
            logical_rect(state, 10, top + 7, 34, top + 31),
            17,
            theme.secondary_text,
            state.dpi,
        ),
        _ => {}
    }
    draw_text(
        dc,
        label,
        logical_rect(state, 50, top + 2, 288, top + ACTION_ROW_HEIGHT - 2),
        FontRole::Body,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
}

fn draw_settings(dc: HDC, state: &WindowState, theme: Theme) {
    draw_subview_header(dc, state, theme, i18n::text(state.locale(), Text::Settings));
    draw_text(
        dc,
        i18n::text(state.locale(), Text::System),
        logical_rect(state, 16, 68, 288, 84),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    draw_interaction_surface(
        dc,
        state,
        theme,
        Target::Startup,
        logical_rect(state, 8, SETTINGS_STARTUP_TOP, 296, SETTINGS_STARTUP_BOTTOM),
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::OpenAtLogin),
        logical_rect(state, 16, 90, 244, 116),
        FontRole::BodyStrong,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::StartAtLoginDescription),
        logical_rect(state, 16, 114, 244, 138),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    draw_toggle(dc, state, theme, 252, 106, state.startup_enabled);
    fill(dc, logical_rect(state, 16, 152, 288, 153), theme.rule);
    draw_text(
        dc,
        i18n::text(state.locale(), Text::QuickAccess),
        logical_rect(state, 16, 160, 288, 178),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        state,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::OpenSelectorNearCursor),
        logical_rect(state, 16, 182, 204, 206),
        FontRole::Body,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    draw_keycap(
        dc,
        state,
        theme,
        logical_rect(state, 210, 184, 288, 206),
        "Ctrl + Alt + A",
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::AudibleAppUnderPointer),
        logical_rect(state, 16, 204, 288, 224),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    fill(dc, logical_rect(state, 16, 232, 288, 233), theme.rule);
    draw_interaction_surface(
        dc,
        state,
        theme,
        Target::WindowsMixer,
        logical_rect(state, 8, SETTINGS_MIXER_TOP, 296, SETTINGS_MIXER_BOTTOM),
    );
    symbols::draw(
        dc,
        Symbol::Volume,
        logical_rect(state, 14, 256, 38, 280),
        18,
        theme.secondary_text,
        state.dpi,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::SystemAudioSettings),
        logical_rect(state, 50, 242, 260, 268),
        FontRole::BodyStrong,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::OpenWindowsMixer),
        logical_rect(state, 50, 266, 260, 290),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    symbols::draw(
        dc,
        Symbol::OpenExternal,
        logical_rect(state, 264, 256, 288, 280),
        15,
        theme.secondary_text,
        state.dpi,
    );
    fill(dc, logical_rect(state, 16, 303, 288, 304), theme.rule);
    let language_row = logical_rect(
        state,
        8,
        SETTINGS_LANGUAGE_TOP,
        296,
        SETTINGS_LANGUAGE_BOTTOM,
    );
    draw_interaction_surface(dc, state, theme, Target::Language, language_row);
    draw_text(
        dc,
        i18n::text(state.locale(), Text::Language),
        logical_rect(state, 16, 306, 154, 332),
        FontRole::BodyStrong,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    let language_label = match state.language.choice() {
        LanguageChoice::Automatic => state.language.automatic_label(),
        LanguageChoice::Manual(locale) => locale.display_name().to_owned(),
    };
    draw_text(
        dc,
        &language_label,
        logical_rect(state, 158, 306, 276, 332),
        FontRole::Caption,
        theme.secondary_text,
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    draw_text(
        dc,
        i18n::text(state.locale(), Text::LanguageDescription),
        logical_rect(state, 16, 330, 276, 354),
        FontRole::Caption,
        theme.secondary_text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    symbols::draw(
        dc,
        Symbol::ChevronRight,
        logical_rect(state, 278, 320, 296, 344),
        11,
        theme.secondary_text,
        state.dpi,
    );
}

fn draw_languages(dc: HDC, state: &WindowState, theme: Theme) {
    draw_subview_header(
        dc,
        state,
        theme,
        i18n::text(state.locale(), Text::ChooseLanguage),
    );
    let selected = state.language.choice().list_index();
    for index in 0..LANGUAGE_CHOICE_COUNT {
        let top = LANGUAGE_CONTENT_TOP + LANGUAGE_ROW_HEIGHT * index as i32;
        let target = Target::Locale(index);
        let row = logical_rect(state, 8, top, 296, top + LANGUAGE_ROW_HEIGHT);
        draw_interaction_surface(dc, state, theme, target, row);
        if selected == index {
            symbols::draw(
                dc,
                Symbol::Check,
                logical_rect(state, 10, top + 4, 34, top + 28),
                13,
                theme.accent,
                state.dpi,
            );
        }
        let label = if index == 0 {
            state.language.automatic_label()
        } else {
            LanguageChoice::from_list_index(index)
                .and_then(|choice| match choice {
                    LanguageChoice::Manual(locale) => Some(locale.display_name().to_owned()),
                    LanguageChoice::Automatic => None,
                })
                .unwrap_or_default()
        };
        draw_text(
            dc,
            &label,
            logical_rect(state, 42, top, 288, top + LANGUAGE_ROW_HEIGHT),
            FontRole::Body,
            theme.text,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            state,
        );
    }
}

fn draw_keycap(dc: HDC, state: &WindowState, theme: Theme, rect: RECT, label: &str) {
    let brush = unsafe { CreateSolidBrush(theme.hover) };
    let pen = unsafe { CreatePen(PS_SOLID, 1, theme.rule) };
    let previous_brush = unsafe { SelectObject(dc, HGDIOBJ(brush.0)) };
    let previous_pen = unsafe { SelectObject(dc, HGDIOBJ(pen.0)) };
    unsafe {
        let radius = state.scale(5);
        let _ = RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(HGDIOBJ(pen.0));
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
    draw_text(
        dc,
        label,
        rect,
        FontRole::Caption,
        theme.text,
        windows::Win32::Graphics::Gdi::DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        state,
    );
}

fn draw_toggle(dc: HDC, state: &WindowState, theme: Theme, x: i32, y: i32, enabled: bool) {
    let left = state.scale(x);
    let top = state.scale(y);
    let width = state.scale(36);
    let height = state.scale(20);
    let track_color = if enabled { theme.accent } else { theme.rule };
    let track_brush = unsafe { CreateSolidBrush(track_color) };
    let track_pen = unsafe { CreatePen(PS_SOLID, 1, track_color) };
    let previous_brush = unsafe { SelectObject(dc, HGDIOBJ(track_brush.0)) };
    let previous_pen = unsafe { SelectObject(dc, HGDIOBJ(track_pen.0)) };
    unsafe {
        let _ = RoundRect(dc, left, top, left + width, top + height, height, height);
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(HGDIOBJ(track_pen.0));
        let _ = DeleteObject(HGDIOBJ(track_brush.0));
    }

    let knob_diameter = state.scale(14);
    let knob_left = if enabled {
        left + width - knob_diameter - state.scale(3)
    } else {
        left + state.scale(3)
    };
    let knob_top = top + (height - knob_diameter) / 2;
    let knob_brush = unsafe { CreateSolidBrush(theme.text) };
    let knob_pen = unsafe { CreatePen(PS_SOLID, 1, theme.text) };
    let previous_brush = unsafe { SelectObject(dc, HGDIOBJ(knob_brush.0)) };
    let previous_pen = unsafe { SelectObject(dc, HGDIOBJ(knob_pen.0)) };
    unsafe {
        let _ = Ellipse(
            dc,
            knob_left,
            knob_top,
            knob_left + knob_diameter,
            knob_top + knob_diameter,
        );
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(HGDIOBJ(knob_pen.0));
        let _ = DeleteObject(HGDIOBJ(knob_brush.0));
    }
}

fn draw_subview_header(dc: HDC, state: &WindowState, theme: Theme, title: &str) {
    let back_rect = logical_rect(state, 8, 8, 48, 52);
    draw_interaction_surface(dc, state, theme, Target::Back, back_rect);
    symbols::draw(
        dc,
        Symbol::Back,
        back_rect,
        16,
        theme.secondary_text,
        state.dpi,
    );
    draw_text(
        dc,
        title,
        logical_rect(state, 52, 8, 288, 52),
        FontRole::BodyStrong,
        theme.text,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        state,
    );
    fill(
        dc,
        logical_rect(state, 16, HEADER_HEIGHT - 1, 288, HEADER_HEIGHT),
        theme.rule,
    );
}

fn draw_interaction_surface(
    dc: HDC,
    state: &WindowState,
    theme: Theme,
    target: Target,
    rect: RECT,
) {
    if state.pressed == Some(target) {
        fill_rounded(dc, state, rect, 4, theme.pressed);
    } else if state.hovered == Some(target) {
        fill_rounded(dc, state, rect, 4, theme.hover);
    } else if state.keyboard_focus_visible && state.focused_target() == Some(target) {
        stroke_rounded(dc, state, rect, 4, 2, theme.accent);
    }
}

fn fill_rounded(dc: HDC, state: &WindowState, rect: RECT, logical_radius: i32, color: COLORREF) {
    let brush = unsafe { CreateSolidBrush(color) };
    let pen = unsafe { CreatePen(PS_SOLID, 1, color) };
    let previous_brush = unsafe { SelectObject(dc, HGDIOBJ(brush.0)) };
    let previous_pen = unsafe { SelectObject(dc, HGDIOBJ(pen.0)) };
    let radius = state.scale(logical_radius);
    unsafe {
        let _ = RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(HGDIOBJ(pen.0));
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}

fn stroke_rounded(
    dc: HDC,
    state: &WindowState,
    rect: RECT,
    logical_radius: i32,
    logical_width: i32,
    color: COLORREF,
) {
    let pen = unsafe { CreatePen(PS_SOLID, state.scale(logical_width).max(1), color) };
    let hollow_brush = unsafe { GetStockObject(NULL_BRUSH) };
    let previous_brush = unsafe { SelectObject(dc, hollow_brush) };
    let previous_pen = unsafe { SelectObject(dc, HGDIOBJ(pen.0)) };
    let radius = state.scale(logical_radius);
    unsafe {
        let _ = RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(dc, previous_pen);
        SelectObject(dc, previous_brush);
        let _ = DeleteObject(HGDIOBJ(pen.0));
    }
}

fn draw_fallback_app_icon(dc: HDC, state: &WindowState, theme: Theme, x: i32, y: i32, size: i32) {
    symbols::draw(
        dc,
        Symbol::Application,
        logical_rect(state, x, y, x + size, y + size),
        19,
        theme.secondary_text,
        state.dpi,
    );
}

#[derive(Clone, Copy)]
enum FontRole {
    Brand,
    AppName,
    BodyStrong,
    Body,
    Caption,
}

fn draw_text(
    dc: HDC,
    value: &str,
    mut rect: RECT,
    role: FontRole,
    color: COLORREF,
    format: windows::Win32::Graphics::Gdi::DRAW_TEXT_FORMAT,
    state: &WindowState,
) {
    let (size, weight, face) = match role {
        FontRole::Brand => (15, FW_SEMIBOLD.0 as i32, w!("Segoe UI Variable Text")),
        FontRole::AppName => (14, FW_MEDIUM.0 as i32, w!("Segoe UI Variable Text")),
        FontRole::BodyStrong => (13, FW_SEMIBOLD.0 as i32, w!("Segoe UI Variable Text")),
        FontRole::Body => (13, FW_NORMAL.0 as i32, w!("Segoe UI Variable Text")),
        FontRole::Caption => (12, FW_NORMAL.0 as i32, w!("Segoe UI Variable Text")),
    };
    // A negative height requests character height rather than the cell height.
    let font = unsafe {
        CreateFontW(
            -state.scale(size),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            face,
        )
    };
    // SAFETY: The font and UTF-16 buffer live through this synchronous draw.
    let previous = unsafe { SelectObject(dc, HGDIOBJ(font.0)) };
    unsafe { SetTextColor(dc, color) };
    let mut utf16: Vec<u16> = value.encode_utf16().collect();
    unsafe {
        DrawTextW(dc, &mut utf16, &mut rect, format);
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(font.0));
    }
}

fn fill(dc: HDC, rect: RECT, color: COLORREF) {
    // SAFETY: One owned brush is used synchronously and then released.
    let brush: HBRUSH = unsafe { CreateSolidBrush(color) };
    unsafe {
        FillRect(dc, &rect, brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}

fn draw_panel_border(dc: HDC, state: &WindowState, client: RECT, theme: Theme) {
    let line = state.scale(1).max(1);
    fill(
        dc,
        RECT {
            bottom: line,
            ..client
        },
        theme.rule,
    );
    fill(
        dc,
        RECT {
            top: client.bottom - line,
            ..client
        },
        theme.rule,
    );
    fill(
        dc,
        RECT {
            right: line,
            ..client
        },
        theme.rule,
    );
    fill(
        dc,
        RECT {
            left: client.right - line,
            ..client
        },
        theme.rule,
    );
}

fn logical_rect(state: &WindowState, left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left: state.scale(left),
        top: state.scale(top),
        right: state.scale(right),
        bottom: state.scale(bottom),
    }
}

fn hit_test_view_logical(x: i32, y: i32, state: &WindowState) -> Option<Target> {
    match state.view {
        View::Main => hit_test_logical(x, y, state.model.applications.len()),
        View::Settings => {
            if (8..48).contains(&x) && (8..52).contains(&y) {
                Some(Target::Back)
            } else if (8..296).contains(&x)
                && (SETTINGS_STARTUP_TOP..SETTINGS_STARTUP_BOTTOM).contains(&y)
            {
                Some(Target::Startup)
            } else if (8..296).contains(&x)
                && (SETTINGS_MIXER_TOP..SETTINGS_MIXER_BOTTOM).contains(&y)
            {
                Some(Target::WindowsMixer)
            } else if (8..296).contains(&x)
                && (SETTINGS_LANGUAGE_TOP..SETTINGS_LANGUAGE_BOTTOM).contains(&y)
            {
                Some(Target::Language)
            } else {
                None
            }
        }
        View::Languages => {
            if (8..48).contains(&x) && (8..52).contains(&y) {
                Some(Target::Back)
            } else if (8..296).contains(&x)
                && (LANGUAGE_CONTENT_TOP
                    ..LANGUAGE_CONTENT_TOP + LANGUAGE_ROW_HEIGHT * LANGUAGE_CHOICE_COUNT as i32)
                    .contains(&y)
            {
                Some(Target::Locale(
                    ((y - LANGUAGE_CONTENT_TOP) / LANGUAGE_ROW_HEIGHT) as usize,
                ))
            } else {
                None
            }
        }
    }
}

fn hit_test_logical(x: i32, y: i32, application_count: usize) -> Option<Target> {
    if (HEADER_ACTION_LEFT..296).contains(&x)
        && (HEADER_ACTION_TOP..HEADER_ACTION_BOTTOM).contains(&y)
    {
        return Some(Target::HeaderLocator);
    }
    if !(8..296).contains(&x) {
        return None;
    }
    let content_height = if application_count == 0 {
        EMPTY_CONTENT_HEIGHT
    } else {
        APPLICATION_ROW_HEIGHT * application_count as i32
    };
    let content_top = HEADER_HEIGHT + CONTENT_TOP_PADDING;
    if application_count > 0 && (content_top..content_top + content_height).contains(&y) {
        return Some(Target::Application(
            ((y - content_top) / APPLICATION_ROW_HEIGHT) as usize,
        ));
    }
    let actions_top = content_top + content_height + 1;
    if (actions_top..actions_top + ACTION_ROW_HEIGHT).contains(&y) {
        return Some(Target::Settings);
    }
    if (actions_top + ACTION_ROW_HEIGHT..actions_top + ACTION_ROW_HEIGHT * 2).contains(&y) {
        return Some(Target::Exit);
    }
    None
}

fn target_for_navigation_index(index: usize, application_count: usize) -> Target {
    if index == 0 {
        Target::HeaderLocator
    } else if index <= application_count {
        Target::Application(index - 1)
    } else if index == application_count + 1 {
        Target::Settings
    } else {
        Target::Exit
    }
}

fn target_for_view_navigation_index(index: usize, state: &WindowState) -> Option<Target> {
    match state.view {
        View::Main => Some(target_for_navigation_index(
            index,
            state.model.applications.len(),
        )),
        View::Settings => match index {
            0 => Some(Target::Back),
            1 => Some(Target::Startup),
            2 => Some(Target::WindowsMixer),
            3 => Some(Target::Language),
            _ => None,
        },
        View::Languages => {
            if index == 0 {
                Some(Target::Back)
            } else {
                (index <= LANGUAGE_CHOICE_COUNT).then_some(Target::Locale(index - 1))
            }
        }
    }
}

fn navigation_index_for_target(target: Target, state: &WindowState) -> Option<usize> {
    match (state.view, target) {
        (View::Main, Target::HeaderLocator) => Some(0),
        (View::Main, Target::Application(index)) => Some(index + 1),
        (View::Main, Target::Settings) => Some(state.model.applications.len() + 1),
        (View::Main, Target::Exit) => Some(state.model.applications.len() + 2),
        (View::Settings, Target::Back) => Some(0),
        (View::Settings, Target::Startup) => Some(1),
        (View::Settings, Target::WindowsMixer) => Some(2),
        (View::Settings, Target::Language) => Some(3),
        (View::Languages, Target::Back) => Some(0),
        (View::Languages, Target::Locale(index)) if index < LANGUAGE_CHOICE_COUNT => {
            Some(index + 1)
        }
        _ => None,
    }
}

fn point_x(lparam: LPARAM) -> i32 {
    (lparam.0 as u16 as i16) as i32
}

fn point_y(lparam: LPARAM) -> i32 {
    ((lparam.0 >> 16) as u16 as i16) as i32
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

fn invalidate(window: HWND) {
    // SAFETY: Invalidating a live HWND retains no borrowed pointer.
    let _ = unsafe { InvalidateRect(Some(window), None, false) };
}

fn invalidate_header(window: HWND, state: &WindowState) {
    let rect = logical_rect(state, 0, 0, WINDOW_WIDTH, HEADER_HEIGHT);
    // SAFETY: The rectangle is valid only for this synchronous invalidation call.
    let _ = unsafe { InvalidateRect(Some(window), Some(&raw const rect), false) };
}

fn invalidate_application_row(window: HWND, state: &WindowState, index: usize) {
    if index >= state.model.applications.len() {
        return;
    }
    let top = HEADER_HEIGHT + CONTENT_TOP_PADDING + APPLICATION_ROW_HEIGHT * index as i32;
    let rect = logical_rect(state, 8, top, 296, top + APPLICATION_ROW_HEIGHT);
    // SAFETY: The rectangle is valid only for this synchronous invalidation call.
    let _ = unsafe { InvalidateRect(Some(window), Some(&raw const rect), false) };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::super::model::{ApplicationRow, OutputChoice};
    use super::*;

    fn routing_model() -> PopupModel {
        PopupModel {
            applications: vec![ApplicationRow {
                application_id: "player".to_owned(),
                name: "Player".to_owned(),
                icon_path: None,
                process_ids: vec![42],
                selected_output_id: None,
                volume_percent: 50,
                muted: false,
            }],
            outputs: vec![OutputChoice {
                id: "headset".to_owned(),
                name: "Headset".to_owned(),
                is_system_default: false,
            }],
        }
    }

    #[test]
    fn application_rows_have_distinct_hit_targets() {
        assert_eq!(
            hit_test_logical(20, HEADER_HEIGHT + CONTENT_TOP_PADDING + 3, 3),
            Some(Target::Application(0))
        );
        assert_eq!(
            hit_test_logical(
                20,
                HEADER_HEIGHT + CONTENT_TOP_PADDING + APPLICATION_ROW_HEIGHT + 3,
                3,
            ),
            Some(Target::Application(1))
        );
    }

    #[test]
    fn application_list_keeps_eight_pixels_below_the_header_rule() {
        assert_eq!(
            hit_test_logical(20, HEADER_HEIGHT + CONTENT_TOP_PADDING - 1, 1),
            None
        );
        assert_eq!(
            hit_test_logical(20, HEADER_HEIGHT + CONTENT_TOP_PADDING, 1),
            Some(Target::Application(0))
        );
    }

    #[test]
    fn row_text_stays_centered_with_its_application_icon_in_every_mode() {
        let row_top = 80;
        let (text_top, text_bottom) = application_primary_text_bounds(row_top);
        let text_center = (text_top + text_bottom) / 2;
        let icon_center = row_top + 10 + 12;

        assert_eq!(text_center, icon_center);
        assert_eq!(APPLICATION_ROW_HEIGHT, 44);
    }

    #[test]
    fn settings_and_exit_are_clickable_even_when_no_apps_exist() {
        let top = HEADER_HEIGHT + CONTENT_TOP_PADDING + EMPTY_CONTENT_HEIGHT + 1;
        assert_eq!(hit_test_logical(20, top + 2, 0), Some(Target::Settings));
        assert_eq!(
            hit_test_logical(20, top + ACTION_ROW_HEIGHT + 2, 0),
            Some(Target::Exit)
        );
    }

    #[test]
    fn settings_view_exposes_back_and_startup_as_separate_targets() {
        let mut state = WindowState::new(
            routing_model(),
            Box::new(|_| Ok(())),
            Box::new(|_| Ok(())),
            Box::new(|| Err("unused".to_owned())),
            96,
        );
        state.view = View::Settings;

        assert_eq!(hit_test_view_logical(20, 20, &state), Some(Target::Back));
        assert_eq!(hit_test_view_logical(20, 84, &state), None);
        assert_eq!(
            hit_test_view_logical(20, 100, &state),
            Some(Target::Startup)
        );
    }

    #[test]
    fn sound_locator_hit_area_matches_its_header_surface() {
        assert_eq!(
            hit_test_logical(HEADER_ACTION_LEFT, HEADER_ACTION_TOP, 1),
            Some(Target::HeaderLocator)
        );
        assert_eq!(hit_test_logical(276, 22, 1), Some(Target::HeaderLocator));
        assert_eq!(hit_test_logical(296, 22, 1), None);
        assert_eq!(hit_test_logical(276, HEADER_ACTION_BOTTOM, 1), None);
    }

    #[test]
    fn brand_icon_is_centered_on_the_two_line_identity_block() {
        let icon_center = (14 + 50) / 2;
        let identity_center = (9 + 55) / 2;
        assert_eq!(icon_center, identity_center);
    }

    #[test]
    fn two_line_brand_header_has_a_full_spacing_step_before_content() {
        assert_eq!(HEADER_HEIGHT, 64);
        assert_eq!(CONTENT_TOP_PADDING, 8);
    }

    #[test]
    fn taskbar_companion_probes_inside_the_native_menu_above_a_bottom_taskbar() {
        let button = RECT {
            left: 1194,
            top: 1032,
            right: 1238,
            bottom: 1080,
        };
        let work = RECT {
            left: 0,
            top: 0,
            right: 2560,
            bottom: 1032,
        };

        assert_eq!(taskbar_menu_probe(button, work), POINT { x: 1216, y: 1031 });
    }

    #[test]
    fn only_known_native_menu_window_classes_are_used_as_companion_anchors() {
        assert!(is_native_taskbar_menu_class("Windows.UI.Core.CoreWindow"));
        assert!(is_native_taskbar_menu_class("Xaml_WindowedPopupClass"));
        assert!(is_native_taskbar_menu_class("#32768"));
        assert!(!is_native_taskbar_menu_class("CabinetWClass"));
    }

    #[test]
    fn taskbar_companion_uses_the_visible_dwm_frame_instead_of_the_shadow_bounds() {
        let shadow_bounds = RECT {
            left: 80,
            top: 66,
            right: 384,
            bottom: 402,
        };
        let visible_bounds = RECT {
            left: 82,
            top: 79,
            right: 382,
            bottom: 400,
        };

        assert_eq!(
            preferred_native_menu_bounds(Some(visible_bounds), shadow_bounds),
            visible_bounds
        );
    }

    #[test]
    fn xaml_jump_list_transparent_inset_is_removed_from_the_companion_anchor() {
        let host_bounds = RECT {
            left: 42,
            top: 23,
            right: 360,
            bottom: 375,
        };

        assert_eq!(
            content_bounds_for_native_menu("Windows.UI.Core.CoreWindow", host_bounds, 96),
            RECT {
                left: 54,
                top: 35,
                right: 348,
                bottom: 363,
            }
        );
        assert_eq!(
            content_bounds_for_native_menu("Xaml_WindowedPopupClass", host_bounds, 144),
            RECT {
                left: 60,
                top: 41,
                right: 342,
                bottom: 357,
            }
        );
        assert_eq!(
            content_bounds_for_native_menu("#32768", host_bounds, 96),
            host_bounds
        );
    }

    #[test]
    fn native_menu_is_taken_only_after_an_output_route_is_accepted() {
        let native_menu = HWND(123_isize as *mut core::ffi::c_void);
        let mut pending = Some(native_menu);

        assert_eq!(take_native_menu_after_route(&mut pending, false), None);
        assert_eq!(pending, Some(native_menu));
        assert_eq!(
            take_native_menu_after_route(&mut pending, true),
            Some(native_menu)
        );
        assert_eq!(pending, None);
    }

    #[test]
    fn accepted_route_posts_a_close_message_to_the_native_companion() {
        let instance = unsafe { GetModuleHandleW(None) }.expect("module handle");
        let native_menu = unsafe {
            CreateWindowExW(
                Default::default(),
                windows::core::w!("STATIC"),
                windows::core::w!("Native menu test"),
                WS_POPUP,
                0,
                0,
                100,
                100,
                None,
                None,
                Some(instance.into()),
                None,
            )
        }
        .expect("test window");

        request_native_menu_close(native_menu);

        let mut message = MSG::default();
        assert!(unsafe {
            windows::Win32::UI::WindowsAndMessaging::PeekMessageW(
                &mut message,
                Some(native_menu),
                WM_CLOSE,
                WM_CLOSE,
                windows::Win32::UI::WindowsAndMessaging::PM_REMOVE,
            )
            .as_bool()
        });
        unsafe { DispatchMessageW(&message) };
        assert!(!unsafe {
            windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(native_menu)).as_bool()
        });
    }

    #[test]
    fn settings_section_label_does_not_touch_the_startup_control() {
        let mut state = WindowState::new(
            routing_model(),
            Box::new(|_| Ok(())),
            Box::new(|_| Ok(())),
            Box::new(|| Err("unused".to_owned())),
            96,
        );
        state.view = View::Settings;

        assert_eq!(hit_test_view_logical(20, 84, &state), None);
        assert_eq!(hit_test_view_logical(20, 88, &state), Some(Target::Startup));
    }

    #[test]
    fn settings_language_row_opens_a_dedicated_choice_view() {
        let mut state = WindowState::new(
            routing_model(),
            Box::new(|_| Ok(())),
            Box::new(|_| Ok(())),
            Box::new(|| Err("unused".to_owned())),
            96,
        );
        state.view = View::Settings;

        assert_eq!(
            hit_test_view_logical(20, SETTINGS_LANGUAGE_TOP, &state),
            Some(Target::Language)
        );
        assert_eq!(
            target_for_view_navigation_index(3, &state),
            Some(Target::Language)
        );

        state.view = View::Languages;
        assert_eq!(
            hit_test_view_logical(20, LANGUAGE_CONTENT_TOP, &state),
            Some(Target::Locale(0))
        );
        assert_eq!(
            hit_test_view_logical(20, LANGUAGE_CONTENT_TOP + LANGUAGE_ROW_HEIGHT * 8, &state,),
            Some(Target::Locale(8))
        );
        assert_eq!(
            target_for_view_navigation_index(9, &state),
            Some(Target::Locale(8))
        );
    }

    #[test]
    fn quick_popup_flips_inside_the_monitor_work_area() {
        let work_area = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };

        assert_eq!(
            clamp_popup_to_work_area(POINT { x: 100, y: 100 }, 304, 260, 12, work_area),
            (112, 112)
        );
        assert_eq!(
            clamp_popup_to_work_area(POINT { x: 1900, y: 1020 }, 304, 260, 12, work_area),
            (1584, 748)
        );
    }

    #[test]
    fn keyboard_order_starts_with_locator_and_ends_with_actions() {
        assert_eq!(target_for_navigation_index(0, 2), Target::HeaderLocator);
        assert_eq!(target_for_navigation_index(1, 2), Target::Application(0));
        assert_eq!(target_for_navigation_index(3, 2), Target::Settings);
        assert_eq!(target_for_navigation_index(4, 2), Target::Exit);
    }

    #[test]
    fn selector_choice_updates_ui_only_after_routing_succeeds() {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let captured = Rc::clone(&requests);
        let mut state = WindowState::new(
            routing_model(),
            Box::new(move |request| {
                captured.borrow_mut().push(request);
                Ok(())
            }),
            Box::new(|_| Ok(())),
            Box::new(|| Err("unused".to_owned())),
            96,
        );

        assert!(route_by_choice(HWND::default(), &mut state, 0, 1));
        assert_eq!(requests.borrow()[0].output_id.as_deref(), Some("headset"));
        assert_eq!(
            state.model.applications[0].selected_output_id.as_deref(),
            Some("headset")
        );
        assert!(!route_by_choice(HWND::default(), &mut state, 0, 2));
    }

    #[test]
    fn volume_control_updates_the_model_only_after_windows_accepts_it() {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let captured = Rc::clone(&requests);
        let mut state = WindowState::new(
            routing_model(),
            Box::new(|_| Ok(())),
            Box::new(move |request| {
                captured.borrow_mut().push(request);
                Ok(())
            }),
            Box::new(|| Err("unused".to_owned())),
            96,
        );

        assert!(set_volume_by_value(
            HWND::default(),
            &mut state,
            0,
            35,
            true
        ));
        assert_eq!(requests.borrow()[0].volume_percent, 35);
        assert!(requests.borrow()[0].muted);
        assert_eq!(state.model.applications[0].volume_percent, 35);
        assert!(state.model.applications[0].muted);
    }

    #[test]
    fn repeated_audio_discovery_failure_is_presented_once_per_episode() {
        let mut active_error = None;

        assert!(should_present_discovery_error(
            &mut active_error,
            "COM apartment mismatch"
        ));
        assert!(!should_present_discovery_error(
            &mut active_error,
            "COM apartment mismatch"
        ));
        assert!(should_present_discovery_error(
            &mut active_error,
            "Audio service unavailable"
        ));
    }
}
