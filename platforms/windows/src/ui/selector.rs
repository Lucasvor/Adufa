//! Adjacent native output selector for the compact Windows popup.
//!
//! The selector is intentionally a separate owned popup. This keeps the main
//! application list stable while presenting output choices beside the row that
//! opened it.

use std::mem::{size_of, size_of_val};
use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{
    ERROR_CLASS_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Dwm::{
    DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateCompatibleBitmap,
    CreateCompatibleDC, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH,
    DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW,
    Ellipse, EndPaint, FW_NORMAL, FW_SEMIBOLD, FillRect, GetMonitorInfoW, HBRUSH, HDC, HGDIOBJ,
    InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect, OUT_DEFAULT_PRECIS,
    PAINTSTRUCT, SRCCOPY, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_DOWN, VK_ESCAPE,
    VK_LEFT, VK_RETURN, VK_RIGHT, VK_SPACE, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CallNextHookEx, CreateWindowExW, DefWindowProcW,
    DestroyWindow, GWLP_USERDATA, GetClientRect, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowRect, HC_ACTION, HHOOK, IDC_ARROW, KillTimer, LoadCursorW, MSLLHOOKSTRUCT,
    PostMessageW, RegisterClassW, SPI_GETWORKAREA, SendMessageW, SetTimer, SetWindowLongPtrW,
    SetWindowsHookExW, SystemParametersInfoW, UnhookWindowsHookEx, WA_INACTIVE, WH_MOUSE_LL,
    WM_ACTIVATE, WM_CAPTURECHANGED, WM_CLOSE, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MOUSEMOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONDOWN, WM_TIMER, WM_XBUTTONDOWN, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{Error as WindowsError, PCWSTR, w};

use super::i18n::{self, Locale, Text};
use super::icons::OwnedIcon;
use super::motion;
use super::symbols::{self, Symbol};
use super::theme::Theme;

const WINDOW_CLASS: PCWSTR = w!("AdufaOutputSelector");
const WM_MOUSELEAVE: u32 = 0x02A3;

const WINDOW_WIDTH: i32 = 232;
const HEADER_HEIGHT: i32 = 44;
const VOLUME_TOP: i32 = HEADER_HEIGHT + 8;
const VOLUME_HEIGHT: i32 = 40;
const VOLUME_BOTTOM: i32 = VOLUME_TOP + VOLUME_HEIGHT;
const OUTPUT_RULE_TOP: i32 = VOLUME_BOTTOM + 8;
const OUTPUT_LABEL_TOP: i32 = OUTPUT_RULE_TOP + 8;
const SECTION_LABEL_HEIGHT: i32 = 24;
const CHOICES_TOP: i32 = OUTPUT_LABEL_TOP + SECTION_LABEL_HEIGHT;
const ROW_HEIGHT: i32 = 32;
const EDGE_PADDING: i32 = 4;
const ADJACENT_GAP: i32 = 8;
const MUTE_LEFT: i32 = 8;
const MUTE_RIGHT: i32 = 38;
const SLIDER_LEFT: i32 = 44;
const SLIDER_RIGHT: i32 = 180;
const DISMISS_TIMER_ID: usize = 1;
const DISMISS_DELAY_MS: u32 = 75;

static ACTIVE_SELECTOR: AtomicIsize = AtomicIsize::new(0);
static OUTSIDE_POINTER_HOOK: AtomicIsize = AtomicIsize::new(0);

/// Sent synchronously to the owner when the user activates a choice.
///
/// `wParam` is the application index and `lParam` is the choice index. Choice
/// zero means System default; choices `1..` map to the outputs passed to
/// [`show`] in order. The selector closes only when the owner returns
/// `LRESULT(1)`.
pub const ROUTE_MESSAGE: u32 = 0x8000 + 43;

/// Sent synchronously when the application volume or mute state changes.
/// `wParam` is the application index. `lParam` packs percentage and mute state.
pub const VOLUME_MESSAGE: u32 = 0x8000 + 45;

/// Sent synchronously to the owner while the selector is being destroyed.
///
/// `wParam` is the application index and `lParam` is the selector HWND value.
pub const CLOSED_MESSAGE: u32 = 0x8000 + 44;

/// Owned presentation payload for one selector instance.
///
/// Grouping these values keeps the window factory stable as the selector gains
/// application identity without leaking `PopupModel` into this UI module.
pub struct SelectorRequest {
    pub application_index: usize,
    pub application_name: String,
    pub application_icon_path: Option<String>,
    pub volume_percent: u8,
    pub muted: bool,
    pub locale: Locale,
    pub outputs: Vec<(String, String)>,
    pub selected_output_id: Option<String>,
}

/// Controls whether the selector extends beside an Adufa row or accompanies
/// the native taskbar menu.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    Adjacent,
    TaskbarCompanion,
}

#[derive(Debug)]
struct OutputChoice {
    id: String,
    name: String,
}

struct SelectorState {
    owner: HWND,
    application_index: usize,
    application_name: String,
    application_icon: Option<OwnedIcon>,
    outputs: Vec<OutputChoice>,
    selected_choice: Option<usize>,
    focus_choice: usize,
    volume_percent: u8,
    muted: bool,
    volume_focused: bool,
    volume_dragging: bool,
    drag_start_volume: u8,
    hovered_choice: Option<usize>,
    tracking_mouse: bool,
    dpi: u32,
    locale: Locale,
}

impl SelectorState {
    fn scale(&self, value: i32) -> i32 {
        scale(value, self.dpi)
    }

    fn choice_count(&self) -> usize {
        self.outputs.len() + 1
    }

    fn choice_at(&self, x: i32, y: i32) -> Option<usize> {
        let width = self.scale(WINDOW_WIDTH);
        let edge = self.scale(EDGE_PADDING);
        let height = self.scale(CHOICES_TOP + ROW_HEIGHT * self.choice_count() as i32) + edge;
        let choices_top = self.scale(CHOICES_TOP);
        if x < edge || x >= width - edge || y < choices_top || y >= height - edge {
            return None;
        }

        let choice = ((y - choices_top) / self.scale(ROW_HEIGHT).max(1)) as usize;
        (choice < self.choice_count()).then_some(choice)
    }

    fn label_for_choice(&self, choice: usize) -> Option<&str> {
        if choice == 0 {
            Some(i18n::text(self.locale, Text::SystemDefault))
        } else {
            self.outputs
                .get(choice - 1)
                .map(|output| output.name.as_str())
        }
    }

    fn volume_row_at(&self, y: i32) -> bool {
        y >= self.scale(VOLUME_TOP) && y < self.scale(VOLUME_BOTTOM)
    }

    fn mute_at(&self, x: i32, y: i32) -> bool {
        self.volume_row_at(y) && x >= self.scale(MUTE_LEFT) && x < self.scale(MUTE_RIGHT)
    }

    fn slider_at(&self, x: i32, y: i32) -> bool {
        self.volume_row_at(y)
            && x >= self.scale(SLIDER_LEFT - 4)
            && x <= self.scale(SLIDER_RIGHT + 4)
    }

    fn volume_from_x(&self, x: i32) -> u8 {
        volume_percent_from_x(x, self.scale(SLIDER_LEFT), self.scale(SLIDER_RIGHT))
    }
}

/// Opens a dark application-audio popup beside `anchor` and returns its HWND.
///
/// `outputs` contains `(id, display_name)` pairs. The window owns all copied
/// state until `WM_NCDESTROY`; callers handle [`ROUTE_MESSAGE`],
/// [`VOLUME_MESSAGE`], and [`CLOSED_MESSAGE`] in the owner window procedure.
pub fn show(
    owner: HWND,
    request: SelectorRequest,
    dpi: u32,
    anchor: RECT,
    placement: Placement,
) -> windows::core::Result<HWND> {
    let dpi = dpi.max(96);
    let outputs: Vec<OutputChoice> = request
        .outputs
        .into_iter()
        .map(|(id, name)| OutputChoice { id, name })
        .collect();
    let selected_choice = choice_for_selected_output(
        request.selected_output_id.as_deref(),
        outputs.iter().map(|output| output.id.as_str()),
    );
    let state = Box::new(SelectorState {
        owner,
        application_index: request.application_index,
        application_name: request.application_name,
        application_icon: request
            .application_icon_path
            .as_deref()
            .and_then(|path| OwnedIcon::from_path_at_dpi(path, 24, dpi)),
        focus_choice: selected_choice.unwrap_or(0),
        selected_choice,
        volume_percent: request.volume_percent.min(100),
        muted: request.muted,
        volume_focused: false,
        volume_dragging: false,
        drag_start_volume: request.volume_percent.min(100),
        outputs,
        hovered_choice: None,
        tracking_mouse: false,
        dpi,
        locale: request.locale,
    });

    let instance = unsafe { GetModuleHandleW(None)? };
    register_window_class(instance.into())?;

    let width = scale(WINDOW_WIDTH, dpi);
    let height = scale(CHOICES_TOP + ROW_HEIGHT * state.choice_count() as i32, dpi)
        + scale(EDGE_PADDING, dpi);
    let work_area = monitor_work_area(anchor);
    let popup = match placement {
        Placement::Adjacent => {
            adjacent_popup_rect(anchor, work_area, width, height, scale(ADJACENT_GAP, dpi))
        }
        Placement::TaskbarCompanion => {
            adjacent_popup_rect(anchor, work_area, width, height, scale(ADJACENT_GAP, dpi))
        }
    };
    let is_taskbar_companion = placement == Placement::TaskbarCompanion;
    let extended_style = if is_taskbar_companion {
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE
    } else {
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST
    };
    let window_title = wide(i18n::text(request.locale, Text::AudioOutput));
    let raw_state = Box::into_raw(state);

    // SAFETY: `raw_state` is stable heap storage. WM_NCCREATE installs it as
    // window userdata, and WM_NCDESTROY recovers it exactly once.
    let window = match unsafe {
        CreateWindowExW(
            extended_style,
            WINDOW_CLASS,
            PCWSTR(window_title.as_ptr()),
            WS_POPUP,
            popup.left,
            popup.top,
            popup.right - popup.left,
            popup.bottom - popup.top,
            Some(owner),
            None,
            Some(instance.into()),
            Some(raw_state.cast()),
        )
    } {
        Ok(window) => window,
        Err(error) => {
            // WM_NCCREATE is the first point at which ownership transfers. Our
            // procedure accepts it unconditionally, matching the main popup's
            // lifetime contract.
            drop(unsafe { Box::from_raw(raw_state) });
            return Err(error);
        }
    };

    apply_dwm_style(window);
    if is_taskbar_companion {
        motion::show_context_menu_no_activate(window);
    } else {
        motion::show_context_menu(window);
    }
    if let Err(error) = install_outside_pointer_hook(window, instance.into()) {
        let _ = unsafe { DestroyWindow(window) };
        return Err(error);
    }
    Ok(window)
}

fn install_outside_pointer_hook(
    window: HWND,
    instance: windows::Win32::Foundation::HINSTANCE,
) -> windows::core::Result<()> {
    ACTIVE_SELECTOR.store(window.0 as isize, Ordering::Release);
    let hook =
        unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(outside_pointer_hook), Some(instance), 0) };
    match hook {
        Ok(hook) => {
            OUTSIDE_POINTER_HOOK.store(hook.0 as isize, Ordering::Release);
            Ok(())
        }
        Err(error) => {
            ACTIVE_SELECTOR.store(0, Ordering::Release);
            Err(error)
        }
    }
}

unsafe extern "system" fn outside_pointer_hook(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32
        && matches!(
            wparam.0 as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
        )
    {
        let window_value = ACTIVE_SELECTOR.load(Ordering::Acquire);
        if window_value != 0 {
            let window = HWND(window_value as *mut core::ffi::c_void);
            let mut bounds = RECT::default();
            if unsafe { GetWindowRect(window, &mut bounds) }.is_ok() {
                // SAFETY: WH_MOUSE_LL supplies an MSLLHOOKSTRUCT for HC_ACTION.
                let event = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
                if should_dismiss_for_pointer(event.pt, bounds) {
                    let _ = unsafe { PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
                }
            }
        }
    }
    // Never consume input; Adufa only observes it to dismiss its own popup.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn should_dismiss_for_pointer(point: POINT, bounds: RECT) -> bool {
    point.x < bounds.left
        || point.x >= bounds.right
        || point.y < bounds.top
        || point.y >= bounds.bottom
}

fn remove_outside_pointer_hook(window: HWND) {
    if ACTIVE_SELECTOR
        .compare_exchange(window.0 as isize, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        let hook_value = OUTSIDE_POINTER_HOOK.swap(0, Ordering::AcqRel);
        if hook_value != 0 {
            let _ = unsafe { UnhookWindowsHookEx(HHOOK(hook_value as *mut core::ffi::c_void)) };
        }
    }
}

fn register_window_class(
    instance: windows::Win32::Foundation::HINSTANCE,
) -> windows::core::Result<()> {
    // SAFETY: The shared cursor remains owned by Windows.
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: cursor,
        lpszClassName: WINDOW_CLASS,
        ..Default::default()
    };

    // SAFETY: The callback and static class name remain valid for the process.
    if unsafe { RegisterClassW(&class) } != 0 {
        return Ok(());
    }
    // Registering the same class again is expected when the selector reopens.
    if unsafe { GetLastError() } == ERROR_CLASS_ALREADY_EXISTS {
        Ok(())
    } else {
        Err(WindowsError::from_thread())
    }
}

fn monitor_work_area(anchor: RECT) -> RECT {
    let monitor = unsafe { MonitorFromRect(&anchor, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is initialized with the required size and writable.
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        info.rcWork
    } else {
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
            anchor
        }
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: WM_NCCREATE provides a valid CREATESTRUCTW for this call.
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        let state = create.lpCreateParams.cast::<SelectorState>();
        // SAFETY: The Box address remains stable through WM_NCDESTROY.
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, state as isize) };
    }

    // SAFETY: Userdata is zero before WM_NCCREATE and otherwise our Box.
    let state_ptr = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) } as *mut SelectorState;

    match message {
        WM_PAINT if !state_ptr.is_null() => {
            paint(window, unsafe { &*state_ptr });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_ACTIVATE if !state_ptr.is_null() => {
            if (wparam.0 as u32 & 0xffff) == WA_INACTIVE {
                let _ = unsafe { SetTimer(Some(window), DISMISS_TIMER_ID, DISMISS_DELAY_MS, None) };
            } else {
                let _ = unsafe { KillTimer(Some(window), DISMISS_TIMER_ID) };
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            if !state.tracking_mouse {
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: window,
                    dwHoverTime: 0,
                };
                if unsafe { TrackMouseEvent(&mut tracking) }.is_ok() {
                    state.tracking_mouse = true;
                }
            }
            let x = point_x(lparam);
            let y = point_y(lparam);
            if state.volume_dragging {
                let volume = state.volume_from_x(x);
                if volume != state.volume_percent {
                    state.volume_percent = volume;
                    invalidate(window);
                }
            }
            let hovered = state.choice_at(x, y);
            if hovered != state.hovered_choice {
                state.hovered_choice = hovered;
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            state.tracking_mouse = false;
            state.hovered_choice = None;
            invalidate(window);
            LRESULT(0)
        }
        WM_LBUTTONDOWN if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            let x = point_x(lparam);
            let y = point_y(lparam);
            if state.slider_at(x, y) {
                state.volume_focused = true;
                state.volume_dragging = true;
                state.drag_start_volume = state.volume_percent;
                state.volume_percent = state.volume_from_x(x);
                let _ = unsafe { SetCapture(window) };
                invalidate(window);
            } else if state.mute_at(x, y) {
                state.volume_focused = true;
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            let x = point_x(lparam);
            let y = point_y(lparam);
            if state.volume_dragging {
                state.volume_percent = state.volume_from_x(x);
                state.volume_dragging = false;
                let _ = unsafe { ReleaseCapture() };
                commit_volume(window, state, state.drag_start_volume, state.muted);
            } else if state.mute_at(x, y) {
                let previous_muted = state.muted;
                commit_volume(window, state, state.volume_percent, !previous_muted);
            } else if let Some(choice) = state.choice_at(x, y) {
                state.volume_focused = false;
                state.focus_choice = choice;
                activate_choice(window, state, choice);
            }
            LRESULT(0)
        }
        WM_CAPTURECHANGED if !state_ptr.is_null() => {
            let state = unsafe { &mut *state_ptr };
            if state.volume_dragging {
                state.volume_dragging = false;
                state.volume_percent = state.drag_start_volume;
                invalidate(window);
            }
            LRESULT(0)
        }
        WM_KEYDOWN if !state_ptr.is_null() => {
            handle_key(window, unsafe { &mut *state_ptr }, wparam);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == DISMISS_TIMER_ID => {
            let _ = unsafe { KillTimer(Some(window), DISMISS_TIMER_ID) };
            if unsafe { GetForegroundWindow() } != window {
                let _ = unsafe { DestroyWindow(window) };
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = unsafe { DestroyWindow(window) };
            LRESULT(0)
        }
        WM_NCDESTROY if !state_ptr.is_null() => {
            remove_outside_pointer_hook(window);
            let state = unsafe { &*state_ptr };
            // Synchronous notification makes it safe for the owner to clear its
            // selector HWND before another one can be opened.
            unsafe {
                SendMessageW(
                    state.owner,
                    CLOSED_MESSAGE,
                    Some(WPARAM(state.application_index)),
                    Some(LPARAM(window.0 as isize)),
                );
                SetWindowLongPtrW(window, GWLP_USERDATA, 0);
            }
            drop(unsafe { Box::from_raw(state_ptr) });
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn handle_key(window: HWND, state: &mut SelectorState, key: WPARAM) {
    match key.0 as u16 {
        value if value == VK_DOWN.0 => {
            if state.volume_focused {
                state.volume_focused = false;
                state.focus_choice = 0;
            } else {
                state.focus_choice = (state.focus_choice + 1) % state.choice_count();
            }
            invalidate(window);
        }
        value if value == VK_UP.0 => {
            if state.volume_focused {
                state.focus_choice = state.choice_count() - 1;
                state.volume_focused = false;
            } else if state.focus_choice == 0 {
                state.volume_focused = true;
            } else {
                state.focus_choice -= 1;
            }
            invalidate(window);
        }
        value if value == VK_RETURN.0 || value == VK_SPACE.0 => {
            if state.volume_focused {
                commit_volume(window, state, state.volume_percent, !state.muted);
            } else {
                activate_choice(window, state, state.focus_choice);
            }
        }
        value if value == VK_RIGHT.0 && state.volume_focused => {
            let previous = state.volume_percent;
            let next = state.volume_percent.saturating_add(5).min(100);
            state.volume_percent = next;
            commit_volume(window, state, previous, state.muted);
        }
        value if value == VK_LEFT.0 && state.volume_focused => {
            let previous = state.volume_percent;
            state.volume_percent = state.volume_percent.saturating_sub(5);
            commit_volume(window, state, previous, state.muted);
        }
        value if value == VK_ESCAPE.0 || value == VK_LEFT.0 => {
            let _ = unsafe { DestroyWindow(window) };
        }
        _ => {}
    }
}

fn commit_volume(
    window: HWND,
    state: &mut SelectorState,
    previous_volume: u8,
    requested_muted: bool,
) {
    let previous_muted = state.muted;
    state.muted = requested_muted;
    let payload = encode_volume_message(state.volume_percent, state.muted);
    let accepted = unsafe {
        SendMessageW(
            state.owner,
            VOLUME_MESSAGE,
            Some(WPARAM(state.application_index)),
            Some(LPARAM(payload)),
        )
    };
    if accepted != LRESULT(1) {
        state.volume_percent = previous_volume;
        state.muted = previous_muted;
    }
    invalidate(window);
}

fn activate_choice(window: HWND, state: &SelectorState, choice: usize) {
    if choice >= state.choice_count() {
        return;
    }
    // SAFETY: The owner HWND supplied to `show` receives only integer payloads.
    let accepted = unsafe {
        SendMessageW(
            state.owner,
            ROUTE_MESSAGE,
            Some(WPARAM(state.application_index)),
            Some(LPARAM(choice as isize)),
        )
    };
    if accepted == LRESULT(1) {
        let _ = unsafe { DestroyWindow(window) };
    }
}

fn apply_dwm_style(window: HWND) {
    let rounded = DWMWCP_ROUND;
    let dark = 1_i32;
    // SAFETY: Each value matches the exact type and size required by DWM.
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

fn paint(window: HWND, state: &SelectorState) {
    let mut paint = PAINTSTRUCT::default();
    // SAFETY: BeginPaint and EndPaint are paired within this WM_PAINT.
    let dc = unsafe { BeginPaint(window, &mut paint) };
    let mut client = RECT::default();
    if unsafe { GetClientRect(window, &mut client) }.is_err() {
        let _ = unsafe { EndPaint(window, &paint) };
        return;
    }

    let buffer_dc = unsafe { CreateCompatibleDC(Some(dc)) };
    let bitmap = unsafe {
        CreateCompatibleBitmap(dc, client.right - client.left, client.bottom - client.top)
    };
    if buffer_dc.0.is_null() || bitmap.0.is_null() {
        if !buffer_dc.0.is_null() {
            let _ = unsafe { DeleteDC(buffer_dc) };
        }
        if !bitmap.0.is_null() {
            let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        }
        let _ = unsafe { EndPaint(window, &paint) };
        return;
    }
    let previous_bitmap = unsafe { SelectObject(buffer_dc, HGDIOBJ(bitmap.0)) };

    render(buffer_dc, client, state);
    let _ = unsafe {
        BitBlt(
            dc,
            0,
            0,
            client.right - client.left,
            client.bottom - client.top,
            Some(buffer_dc),
            0,
            0,
            SRCCOPY,
        )
    };
    unsafe {
        SelectObject(buffer_dc, previous_bitmap);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(buffer_dc);
    }
    let _ = unsafe { EndPaint(window, &paint) };
}

fn render(dc: HDC, client: RECT, state: &SelectorState) {
    let theme = Theme::DARK;
    fill(dc, client, theme.surface);
    unsafe { SetBkMode(dc, TRANSPARENT) };

    draw_header(dc, state, theme);
    draw_volume_control(dc, state, theme);
    for choice in 0..state.choice_count() {
        draw_choice(dc, state, theme, choice);
    }
    draw_panel_border(dc, state, client, theme);
}

fn draw_header(dc: HDC, state: &SelectorState, theme: Theme) {
    let icon_size = state.scale(24);
    if let Some(icon) = &state.application_icon {
        icon.draw(dc, state.scale(10), state.scale(10), icon_size);
    } else {
        symbols::draw(
            dc,
            Symbol::Application,
            RECT {
                left: state.scale(8),
                top: state.scale(6),
                right: state.scale(40),
                bottom: state.scale(42),
            },
            19,
            theme.secondary_text,
            state.dpi,
        );
    }
    draw_text_with_style(
        dc,
        &state.application_name,
        RECT {
            left: state.scale(42),
            top: state.scale(4),
            right: state.scale(WINDOW_WIDTH - 10),
            bottom: state.scale(40),
        },
        theme.text,
        13,
        FW_SEMIBOLD.0 as i32,
        state,
    );
    fill(
        dc,
        RECT {
            left: state.scale(10),
            top: state.scale(HEADER_HEIGHT - 1),
            right: state.scale(WINDOW_WIDTH - 10),
            bottom: state.scale(HEADER_HEIGHT),
        },
        theme.rule,
    );
}

fn draw_volume_control(dc: HDC, state: &SelectorState, theme: Theme) {
    symbols::draw(
        dc,
        if state.muted {
            Symbol::Mute
        } else {
            Symbol::Volume
        },
        RECT {
            left: state.scale(MUTE_LEFT),
            top: state.scale(VOLUME_TOP + 4),
            right: state.scale(MUTE_RIGHT),
            bottom: state.scale(VOLUME_BOTTOM - 4),
        },
        16,
        if state.muted {
            theme.secondary_text
        } else {
            theme.text
        },
        state.dpi,
    );

    let slider_left = state.scale(SLIDER_LEFT);
    let slider_right = state.scale(SLIDER_RIGHT);
    let track_top = state.scale(VOLUME_TOP + 18);
    let track_height = state.scale(4).max(2);
    let thumb_x =
        slider_left + (slider_right - slider_left) * i32::from(state.volume_percent) / 100;
    fill(
        dc,
        RECT {
            left: slider_left,
            top: track_top,
            right: slider_right,
            bottom: track_top + track_height,
        },
        theme.meter_track,
    );
    fill(
        dc,
        RECT {
            left: slider_left,
            top: track_top,
            right: thumb_x.max(slider_left),
            bottom: track_top + track_height,
        },
        if state.muted {
            theme.secondary_text
        } else {
            theme.accent
        },
    );
    let radius = state.scale(6).max(4);
    let center_y = track_top + track_height / 2;
    let brush = unsafe {
        CreateSolidBrush(if state.muted {
            theme.secondary_text
        } else {
            theme.accent
        })
    };
    let previous = unsafe { SelectObject(dc, HGDIOBJ(brush.0)) };
    unsafe {
        let _ = Ellipse(
            dc,
            thumb_x - radius,
            center_y - radius,
            thumb_x + radius,
            center_y + radius,
        );
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }

    let volume_label = if state.muted {
        i18n::text(state.locale, Text::Muted).to_owned()
    } else {
        format!("{}%", state.volume_percent)
    };
    draw_text_with_style(
        dc,
        &volume_label,
        RECT {
            left: state.scale(188),
            top: state.scale(VOLUME_TOP),
            right: state.scale(WINDOW_WIDTH - 8),
            bottom: state.scale(VOLUME_BOTTOM),
        },
        theme.secondary_text,
        11,
        FW_NORMAL.0 as i32,
        state,
    );

    fill(
        dc,
        RECT {
            left: state.scale(10),
            top: state.scale(OUTPUT_RULE_TOP),
            right: state.scale(WINDOW_WIDTH - 10),
            bottom: state.scale(OUTPUT_RULE_TOP + 1),
        },
        theme.rule,
    );

    draw_text_with_style(
        dc,
        i18n::text(state.locale, Text::AudioOutput),
        RECT {
            left: state.scale(12),
            top: state.scale(OUTPUT_LABEL_TOP),
            right: state.scale(WINDOW_WIDTH - 10),
            bottom: state.scale(CHOICES_TOP),
        },
        theme.secondary_text,
        11,
        FW_NORMAL.0 as i32,
        state,
    );
}

fn draw_choice(dc: HDC, state: &SelectorState, theme: Theme, choice: usize) {
    let top = state.scale(CHOICES_TOP + ROW_HEIGHT * choice as i32);
    let edge = state.scale(EDGE_PADDING);
    let row = RECT {
        left: edge,
        top,
        right: state.scale(WINDOW_WIDTH) - edge,
        bottom: top + state.scale(ROW_HEIGHT),
    };

    if state.hovered_choice == Some(choice)
        || (!state.volume_focused && state.focus_choice == choice)
    {
        fill(dc, row, theme.hover);
    }
    if state.selected_choice == Some(choice) {
        symbols::draw(
            dc,
            Symbol::Check,
            RECT {
                left: state.scale(6),
                top,
                right: state.scale(30),
                bottom: top + state.scale(ROW_HEIGHT),
            },
            13,
            theme.accent,
            state.dpi,
        );
    }

    if let Some(label) = state.label_for_choice(choice) {
        draw_text(
            dc,
            label,
            RECT {
                left: state.scale(32),
                top,
                right: state.scale(WINDOW_WIDTH - 10),
                bottom: top + state.scale(ROW_HEIGHT),
            },
            theme.text,
            state,
        );
    }
}

fn draw_text(
    dc: HDC,
    value: &str,
    rect: RECT,
    color: windows::Win32::Foundation::COLORREF,
    state: &SelectorState,
) {
    draw_text_with_style(dc, value, rect, color, 13, FW_NORMAL.0 as i32, state);
}

fn draw_text_with_style(
    dc: HDC,
    value: &str,
    mut rect: RECT,
    color: windows::Win32::Foundation::COLORREF,
    logical_size: i32,
    weight: i32,
    state: &SelectorState,
) {
    let font = unsafe {
        CreateFontW(
            -state.scale(logical_size),
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
            w!("Segoe UI Variable Text"),
        )
    };
    let previous = unsafe { SelectObject(dc, HGDIOBJ(font.0)) };
    unsafe { SetTextColor(dc, color) };
    let mut utf16: Vec<u16> = value.encode_utf16().collect();
    unsafe {
        DrawTextW(
            dc,
            &mut utf16,
            &mut rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(font.0));
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

fn fill(dc: HDC, rect: RECT, color: windows::Win32::Foundation::COLORREF) {
    let brush: HBRUSH = unsafe { CreateSolidBrush(color) };
    unsafe {
        FillRect(dc, &rect, brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}

fn draw_panel_border(dc: HDC, state: &SelectorState, client: RECT, theme: Theme) {
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

fn choice_for_selected_output<'a>(
    selected_output_id: Option<&str>,
    output_ids: impl IntoIterator<Item = &'a str>,
) -> Option<usize> {
    match selected_output_id {
        None => Some(0),
        Some(selected) => output_ids
            .into_iter()
            .position(|output_id| output_id == selected)
            .map(|index| index + 1),
    }
}

fn volume_percent_from_x(x: i32, left: i32, right: i32) -> u8 {
    let width = (right - left).max(1);
    let clamped = x.clamp(left, right) - left;
    ((clamped * 100 + width / 2) / width) as u8
}

fn encode_volume_message(volume_percent: u8, muted: bool) -> isize {
    isize::from(volume_percent.min(100)) | (isize::from(muted) << 8)
}

pub fn decode_volume_message(payload: isize) -> (u8, bool) {
    ((payload as u8).min(100), payload & (1 << 8) != 0)
}

fn adjacent_popup_rect(anchor: RECT, work: RECT, width: i32, height: i32, gap: i32) -> RECT {
    let right_x = anchor.right + gap;
    let left_x = anchor.left - gap - width;
    let left = if right_x + width <= work.right {
        right_x
    } else if left_x >= work.left {
        left_x
    } else {
        clamp_origin(right_x, width, work.left, work.right)
    };
    let top = clamp_origin(anchor.top, height, work.top, work.bottom);
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

fn clamp_origin(preferred: i32, size: i32, minimum: i32, maximum: i32) -> i32 {
    let latest = maximum.saturating_sub(size);
    if latest < minimum {
        minimum
    } else {
        preferred.clamp(minimum, latest)
    }
}

fn scale(value: i32, dpi: u32) -> i32 {
    value * dpi as i32 / 96
}

fn point_x(lparam: LPARAM) -> i32 {
    (lparam.0 as u16 as i16) as i32
}

fn point_y(lparam: LPARAM) -> i32 {
    ((lparam.0 >> 16) as u16 as i16) as i32
}

fn invalidate(window: HWND) {
    let _ = unsafe { InvalidateRect(Some(window), None, false) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_output_maps_default_and_real_outputs_to_choice_indices() {
        let outputs = ["speakers", "headset"];
        assert_eq!(choice_for_selected_output(None, outputs), Some(0));
        assert_eq!(
            choice_for_selected_output(Some("headset"), outputs),
            Some(2)
        );
        assert_eq!(choice_for_selected_output(Some("missing"), outputs), None);
    }

    #[test]
    fn volume_slider_maps_and_clamps_pointer_positions() {
        assert_eq!(volume_percent_from_x(40, 40, 180), 0);
        assert_eq!(volume_percent_from_x(110, 40, 180), 50);
        assert_eq!(volume_percent_from_x(180, 40, 180), 100);
        assert_eq!(volume_percent_from_x(4, 40, 180), 0);
        assert_eq!(volume_percent_from_x(240, 40, 180), 100);
    }

    #[test]
    fn volume_message_round_trips_percent_and_mute_state() {
        for (percent, muted) in [(0, false), (37, true), (100, false)] {
            assert_eq!(
                decode_volume_message(encode_volume_message(percent, muted)),
                (percent, muted)
            );
        }
    }

    #[test]
    fn volume_section_has_explicit_spacing_and_dividers() {
        assert_eq!(VOLUME_TOP - HEADER_HEIGHT, 8);
        assert_eq!(OUTPUT_RULE_TOP - VOLUME_BOTTOM, 8);
        assert_eq!(OUTPUT_LABEL_TOP - OUTPUT_RULE_TOP, 8);
        assert_eq!(CHOICES_TOP - OUTPUT_LABEL_TOP, SECTION_LABEL_HEIGHT);
    }

    #[test]
    fn first_choice_respects_the_same_edge_padding_as_the_other_sides() {
        let state = SelectorState {
            owner: HWND::default(),
            application_index: 0,
            application_name: "Player".to_owned(),
            application_icon: None,
            outputs: vec![OutputChoice {
                id: "speakers".to_owned(),
                name: "Speakers".to_owned(),
            }],
            selected_choice: Some(0),
            focus_choice: 0,
            volume_percent: 50,
            muted: false,
            volume_focused: false,
            volume_dragging: false,
            drag_start_volume: 50,
            hovered_choice: None,
            tracking_mouse: false,
            dpi: 96,
            locale: Locale::En,
        };

        assert_eq!(state.choice_at(16, CHOICES_TOP - 1), None);
        assert_eq!(state.choice_at(16, CHOICES_TOP), Some(0));
        assert_eq!(state.choice_at(16, CHOICES_TOP + ROW_HEIGHT), Some(1));
    }

    #[test]
    fn geometry_prefers_the_right_side_when_it_fits() {
        let anchor = RECT {
            left: 100,
            top: 80,
            right: 200,
            bottom: 120,
        };
        let work = RECT {
            left: 0,
            top: 0,
            right: 800,
            bottom: 600,
        };

        assert_eq!(
            adjacent_popup_rect(anchor, work, WINDOW_WIDTH, 200, ADJACENT_GAP),
            RECT {
                left: 208,
                top: 80,
                right: 440,
                bottom: 280,
            }
        );
    }

    #[test]
    fn geometry_falls_back_left_and_clamps_to_the_work_area() {
        let work = RECT {
            left: 0,
            top: 0,
            right: 800,
            bottom: 600,
        };
        let near_right_bottom = RECT {
            left: 650,
            top: 540,
            right: 790,
            bottom: 580,
        };

        assert_eq!(
            adjacent_popup_rect(near_right_bottom, work, WINDOW_WIDTH, 200, ADJACENT_GAP,),
            RECT {
                left: 410,
                top: 400,
                right: 642,
                bottom: 600,
            }
        );
    }

    #[test]
    fn geometry_clamps_when_neither_adjacent_side_fits() {
        let work = RECT {
            left: 100,
            top: 100,
            right: 500,
            bottom: 500,
        };
        let anchor = RECT {
            left: 180,
            top: 40,
            right: 420,
            bottom: 80,
        };

        assert_eq!(
            adjacent_popup_rect(anchor, work, 300, 120, 6),
            RECT {
                left: 200,
                top: 100,
                right: 500,
                bottom: 220,
            }
        );
    }

    #[test]
    fn pointer_dismissal_only_accepts_clicks_outside_the_selector() {
        let selector = RECT {
            left: 100,
            top: 100,
            right: 332,
            bottom: 420,
        };

        assert!(!should_dismiss_for_pointer(
            POINT { x: 100, y: 100 },
            selector
        ));
        assert!(!should_dismiss_for_pointer(
            POINT { x: 331, y: 419 },
            selector
        ));
        assert!(should_dismiss_for_pointer(
            POINT { x: 99, y: 100 },
            selector
        ));
        assert!(should_dismiss_for_pointer(
            POINT { x: 332, y: 420 },
            selector
        ));
    }
}
