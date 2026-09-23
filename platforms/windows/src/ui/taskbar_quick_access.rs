//! Direct, out-of-process quick access for Windows taskbar buttons.
//!
//! Explorer remains untouched. A low-level mouse hook observes right-clicks,
//! inspects the visible taskbar button outside the hook callback, and notifies
//! the UI with a running application target when it can resolve one. The
//! original gesture always continues to Explorer so its native menu is kept.

use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetMonitorInfoW, HGDIOBJ,
    MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DI_NORMAL, DrawIconEx, GA_ROOT, GetAncestor, GetClassNameW, HC_ACTION, HHOOK,
    HICON, LLMHF_INJECTED, MSLLHOOKSTRUCT, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    WH_MOUSE_LL, WM_RBUTTONDOWN, WindowFromPoint,
};

use crate::process_identity;

/// Notifies the UI thread that a known taskbar application was right-clicked.
/// The owned target is retrieved through [`take_pending_target`].
pub const RIGHT_CLICK_MESSAGE: u32 = 0x8000 + 60;

static OWNER_WINDOW: AtomicIsize = AtomicIsize::new(0);
static PENDING_POINT: Mutex<Option<POINT>> = Mutex::new(None);

const TASKBAR_ICON_DIP: i32 = 24;
const TASKBAR_BUTTON_DIP: i32 = 44;
const MATCH_SEARCH_DIP: i32 = 22;
const MAX_MATCH_SCORE: u64 = 4_000;
const MIN_MATCH_MARGIN_PERCENT: u64 = 112;

#[derive(Clone, Debug)]
pub struct TaskbarButtonTarget {
    pub application_index: usize,
    pub bounds: RECT,
    pub application: Option<TaskbarApplication>,
}

#[derive(Clone, Debug)]
pub struct TaskbarApplication {
    pub application_id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub process_ids: Vec<u32>,
}

/// Owns the process-wide mouse hook.
pub struct TaskbarQuickAccessMonitor {
    mouse_hook: HHOOK,
    owner: isize,
}

impl TaskbarQuickAccessMonitor {
    pub fn start(owner: HWND) -> Result<Self, String> {
        let instance: HINSTANCE = unsafe { GetModuleHandleW(None) }
            .map_err(|error| error.to_string())?
            .into();
        let owner_value = owner.0 as isize;
        OWNER_WINDOW.store(owner_value, Ordering::Release);
        let mouse_hook = match unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_callback), Some(instance), 0)
        } {
            Ok(hook) => hook,
            Err(error) => {
                OWNER_WINDOW.store(0, Ordering::Release);
                return Err(error.to_string());
            }
        };

        Ok(Self {
            mouse_hook,
            owner: owner_value,
        })
    }
}

impl Drop for TaskbarQuickAccessMonitor {
    fn drop(&mut self) {
        let _ = OWNER_WINDOW.compare_exchange(self.owner, 0, Ordering::AcqRel, Ordering::Acquire);
        let _ = unsafe { UnhookWindowsHookEx(self.mouse_hook) };
    }
}

/// Moves the most recent observed taskbar target to the UI thread.
pub fn take_pending_target(
    icons: impl IntoIterator<Item = (usize, HICON)>,
) -> Option<TaskbarButtonTarget> {
    let point = PENDING_POINT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()?;
    let taskbar = taskbar_rect_at(point)?;
    if let Some((name, automation_id, bounds)) = inspect_taskbar_button(point) {
        let application = automation_id
            .as_deref()
            .and_then(process_identity::find_running_application_by_aumid)
            .or_else(|| process_identity::find_running_application(&name));
        if let Some(application) = application {
            return Some(TaskbarButtonTarget {
                application_index: usize::MAX,
                bounds,
                application: Some(TaskbarApplication {
                    application_id: application.application_id.as_str().to_owned(),
                    name,
                    icon_path: application.executable_path,
                    process_ids: application.process_ids,
                }),
            });
        }
    }

    let icon_size = taskbar_icon_size(taskbar);
    let search = scale_for_taskbar(MATCH_SEARCH_DIP, taskbar);
    let capture_bounds = RECT {
        left: point.x - search - icon_size,
        top: taskbar.top,
        right: point.x + search + icon_size + 1,
        bottom: taskbar.bottom,
    };
    let capture = capture_screen_region(capture_bounds)?;

    let mut candidates = Vec::new();
    for (application_index, icon) in icons {
        let pixels = render_icon(icon, icon_size)?;
        if let Some(icon_match) = best_icon_match(&capture, &pixels, icon_size, search) {
            candidates.push((application_index, icon_match));
        }
    }
    let (application_index, icon_match) = choose_unique_icon_match(&mut candidates)?;
    let icon_center_x = capture_bounds.left + icon_match.x + icon_size / 2;
    let button_width = scale_for_taskbar(TASKBAR_BUTTON_DIP, taskbar);
    Some(TaskbarButtonTarget {
        application_index,
        bounds: RECT {
            left: icon_center_x - button_width / 2,
            top: taskbar.top,
            right: icon_center_x + (button_width + 1) / 2,
            bottom: taskbar.bottom,
        },
        application: None,
    })
}

struct UiAutomationApartment;

impl UiAutomationApartment {
    fn initialize() -> windows::core::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for UiAutomationApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn inspect_taskbar_button(point: POINT) -> Option<(String, Option<String>, RECT)> {
    let _apartment = UiAutomationApartment::initialize().ok()?;
    let automation: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL).ok()? };
    let element = unsafe { automation.ElementFromPoint(point).ok()? };
    let name = unsafe { element.CurrentName().ok()? }.to_string();
    let automation_id = unsafe { element.CurrentAutomationId().ok() }.map(|id| id.to_string());
    let bounds = unsafe { element.CurrentBoundingRectangle().ok()? };
    if name.trim().is_empty()
        || bounds.right <= bounds.left
        || bounds.bottom <= bounds.top
        || !point_in_rect(point, bounds)
    {
        return None;
    }
    Some((name, automation_id, bounds))
}

unsafe extern "system" fn mouse_hook_callback(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let message = wparam.0 as u32;
    // SAFETY: WH_MOUSE_LL supplies an MSLLHOOKSTRUCT for HC_ACTION.
    let event = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    if event.flags & LLMHF_INJECTED != 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    if message == WM_RBUTTONDOWN && is_taskbar_point(event.pt) {
        *PENDING_POINT
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(event.pt);
        let owner_value = OWNER_WINDOW.load(Ordering::Acquire);
        if owner_value != 0 {
            let owner = HWND(owner_value as *mut core::ffi::c_void);
            let _ = unsafe { PostMessageW(Some(owner), RIGHT_CLICK_MESSAGE, WPARAM(0), LPARAM(0)) };
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[derive(Debug)]
struct PixelBuffer {
    width: i32,
    height: i32,
    pixels: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IconMatch {
    score: u64,
    x: i32,
    y: i32,
}

fn taskbar_rect_at(point: POINT) -> Option<RECT> {
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return None;
    }
    let monitor = info.rcMonitor;
    let work = info.rcWork;
    if !is_taskbar_band(point, monitor, work) {
        return None;
    }

    if work.bottom < monitor.bottom {
        Some(RECT {
            top: work.bottom,
            ..monitor
        })
    } else if work.top > monitor.top {
        Some(RECT {
            bottom: work.top,
            ..monitor
        })
    } else if work.left > monitor.left {
        Some(RECT {
            right: work.left,
            ..monitor
        })
    } else if work.right < monitor.right {
        Some(RECT {
            left: work.right,
            ..monitor
        })
    } else {
        None
    }
}

fn is_taskbar_point(point: POINT) -> bool {
    if taskbar_rect_at(point).is_some() {
        return true;
    }

    // Auto-hidden taskbars can temporarily occupy work-area coordinates. The
    // native class remains a useful fallback for that case.
    let under_cursor = unsafe { WindowFromPoint(point) };
    let root = unsafe { GetAncestor(under_cursor, GA_ROOT) };
    if root.0.is_null() {
        return false;
    }
    let mut class_name = [0_u16; 64];
    let length = unsafe { GetClassNameW(root, &mut class_name) };
    length > 0
        && matches!(
            String::from_utf16_lossy(&class_name[..length as usize]).as_str(),
            "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
        )
}

fn point_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn is_taskbar_band(point: POINT, monitor: RECT, work: RECT) -> bool {
    point_in_rect(point, monitor) && !point_in_rect(point, work)
}

fn scale_for_taskbar(value: i32, taskbar: RECT) -> i32 {
    let thickness = (taskbar.bottom - taskbar.top)
        .min(taskbar.right - taskbar.left)
        .max(1);
    (value * thickness / 48).max(1)
}

fn taskbar_icon_size(taskbar: RECT) -> i32 {
    scale_for_taskbar(TASKBAR_ICON_DIP, taskbar)
}

fn capture_screen_region(rect: RECT) -> Option<PixelBuffer> {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    let (dc, bitmap, previous, bits) = create_dib(width, height)?;
    let screen = unsafe { GetDC(None) };
    if screen.0.is_null() {
        unsafe {
            SelectObject(dc, previous);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(dc);
        }
        return None;
    }
    let copied = unsafe {
        BitBlt(
            dc,
            0,
            0,
            width,
            height,
            Some(screen),
            rect.left,
            rect.top,
            SRCCOPY,
        )
    };
    unsafe { ReleaseDC(None, screen) };
    let pixels = copied.is_ok().then(|| {
        unsafe { std::slice::from_raw_parts(bits.cast::<u32>(), (width * height) as usize) }
            .to_vec()
    });
    unsafe {
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
    }
    pixels.map(|pixels| PixelBuffer {
        width,
        height,
        pixels,
    })
}

fn render_icon(icon: HICON, size: i32) -> Option<PixelBuffer> {
    let (dc, bitmap, previous, bits) = create_dib(size, size)?;
    let drawn = unsafe { DrawIconEx(dc, 0, 0, icon, size, size, 0, None, DI_NORMAL) }.is_ok();
    let pixels = drawn.then(|| unsafe {
        std::slice::from_raw_parts(bits.cast::<u32>(), (size * size) as usize).to_vec()
    });
    unsafe {
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
    }
    pixels.map(|pixels| PixelBuffer {
        width: size,
        height: size,
        pixels,
    })
}

fn create_dib(
    width: i32,
    height: i32,
) -> Option<(
    windows::Win32::Graphics::Gdi::HDC,
    windows::Win32::Graphics::Gdi::HBITMAP,
    HGDIOBJ,
    *mut core::ffi::c_void,
)> {
    if width <= 0 || height <= 0 {
        return None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()? };
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.0.is_null() || bitmap.0.is_null() || bits.is_null() {
        if !bitmap.0.is_null() {
            let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        }
        if !dc.0.is_null() {
            let _ = unsafe { DeleteDC(dc) };
        }
        return None;
    }
    let previous = unsafe { SelectObject(dc, HGDIOBJ(bitmap.0)) };
    Some((dc, bitmap, previous, bits))
}

fn best_icon_match(
    screen: &PixelBuffer,
    icon: &PixelBuffer,
    icon_size: i32,
    search: i32,
) -> Option<IconMatch> {
    if icon.width != icon_size || icon.height != icon_size || screen.height < icon_size {
        return None;
    }
    let nominal_x = (screen.width - icon_size) / 2;
    let nominal_y = (screen.height - icon_size) / 2;
    let mut best: Option<IconMatch> = None;
    for x in (nominal_x - search).max(0)..=(nominal_x + search).min(screen.width - icon_size) {
        for y in (nominal_y - 3).max(0)..=(nominal_y + 3).min(screen.height - icon_size) {
            if let Some(score) = icon_score_at(screen, icon, x, y) {
                let candidate = IconMatch { score, x, y };
                if best.is_none_or(|current| candidate.score < current.score) {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

fn icon_score_at(screen: &PixelBuffer, icon: &PixelBuffer, x: i32, y: i32) -> Option<u64> {
    let background = taskbar_background(screen);
    let mut error = 0_u64;
    let mut samples = 0_u64;
    for row in 0..icon.height {
        for column in 0..icon.width {
            let icon_pixel = icon.pixels[(row * icon.width + column) as usize];
            let alpha = ((icon_pixel >> 24) & 0xff) as u64;
            let effective_alpha = if alpha == 0 && icon_pixel & 0x00ff_ffff != 0 {
                255
            } else {
                alpha
            };
            if effective_alpha < 40 {
                continue;
            }
            let screen_pixel = screen.pixels[((y + row) * screen.width + x + column) as usize];
            for shift in [0, 8, 16] {
                let foreground = ((icon_pixel >> shift) & 0xff) as i64;
                let base = ((background >> shift) & 0xff) as i64;
                // DrawIconEx writes premultiplied BGRA into a 32-bit DIB.
                // The foreground channel already contains its alpha factor.
                let expected = if alpha == 0 {
                    foreground
                } else {
                    foreground + base * (255 - effective_alpha) as i64 / 255
                };
                let actual = ((screen_pixel >> shift) & 0xff) as i64;
                error += (expected - actual).unsigned_abs().pow(2);
            }
            samples += 3;
        }
    }
    (samples >= 24).then_some(error / samples)
}

fn taskbar_background(screen: &PixelBuffer) -> u32 {
    let corners = [
        screen.pixels[0],
        screen.pixels[(screen.width - 1) as usize],
        screen.pixels[((screen.height - 1) * screen.width) as usize],
        screen.pixels[(screen.height * screen.width - 1) as usize],
    ];
    [0, 8, 16].into_iter().fold(0_u32, |color, shift| {
        let channel = corners
            .iter()
            .map(|pixel| (pixel >> shift) & 0xff)
            .sum::<u32>()
            / corners.len() as u32;
        color | channel << shift
    })
}

fn choose_unique_icon_match(candidates: &mut [(usize, IconMatch)]) -> Option<(usize, IconMatch)> {
    candidates.sort_unstable_by_key(|candidate| candidate.1.score);
    let best = *candidates.first()?;
    if best.1.score > MAX_MATCH_SCORE {
        return None;
    }
    if let Some((_, second)) = candidates.get(1)
        && best.1.score.saturating_mul(MIN_MATCH_MARGIN_PERCENT) >= second.score.saturating_mul(100)
    {
        return None;
    }
    Some(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_match_requires_a_clear_winner() {
        let at = |score| IconMatch { score, x: 4, y: 6 };
        let mut clear = [(3, at(120)), (1, at(500)), (8, at(800))];
        assert_eq!(choose_unique_icon_match(&mut clear), Some((3, at(120))));

        let mut ambiguous = [(3, at(120)), (1, at(130))];
        assert_eq!(choose_unique_icon_match(&mut ambiguous), None);

        let mut poor = [(3, at(MAX_MATCH_SCORE + 1))];
        assert_eq!(choose_unique_icon_match(&mut poor), None);
    }

    #[test]
    fn visual_match_returns_the_actual_icon_position_not_the_click_center() {
        let background = 0x0014_1414;
        let icon_pixel = 0xffff_ffff;
        let icon = PixelBuffer {
            width: 3,
            height: 3,
            pixels: vec![icon_pixel; 9],
        };
        let mut screen_pixels = vec![background; 35];
        for row in 0..3 {
            for column in 0..3 {
                screen_pixels[((row + 1) * 7 + column + 3) as usize] = icon_pixel;
            }
        }
        let screen = PixelBuffer {
            width: 7,
            height: 5,
            pixels: screen_pixels,
        };

        assert_eq!(
            best_icon_match(&screen, &icon, 3, 2),
            Some(IconMatch {
                score: 0,
                x: 3,
                y: 1,
            })
        );
    }

    #[test]
    fn icon_score_composites_premultiplied_pixels_only_once() {
        let background = 0x0014_1414;
        let premultiplied_green = 0x8000_6400;
        let composed_green = 0x0009_6d09;
        let icon = PixelBuffer {
            width: 3,
            height: 3,
            pixels: vec![premultiplied_green; 9],
        };
        let mut screen_pixels = vec![background; 25];
        for row in 0..3 {
            for column in 0..3 {
                screen_pixels[((row + 1) * 5 + column + 1) as usize] = composed_green;
            }
        }
        let screen = PixelBuffer {
            width: 5,
            height: 5,
            pixels: screen_pixels,
        };

        assert_eq!(icon_score_at(&screen, &icon, 1, 1), Some(0));
    }

    #[test]
    fn taskbar_band_is_detected_without_relying_on_explorer_window_classes() {
        let monitor = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let work = RECT {
            bottom: 1040,
            ..monitor
        };

        assert!(is_taskbar_band(POINT { x: 900, y: 1060 }, monitor, work));
        assert!(!is_taskbar_band(POINT { x: 900, y: 900 }, monitor, work));
    }
}
