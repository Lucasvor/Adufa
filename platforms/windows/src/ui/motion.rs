//! Shared motion policy for native Windows popup surfaces.
//!
//! Motion is event-driven, short, and optional. It consumes no timer or CPU
//! while Adufa is idle and follows the user's Windows animation preference.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    AW_BLEND, AnimateWindow, IsWindowVisible, SPI_GETCLIENTAREAANIMATION, SW_SHOW,
    SW_SHOWNOACTIVATE, SetForegroundWindow, ShowWindow, SystemParametersInfoW,
};
use windows::core::BOOL;

const OPEN_ANIMATION_MS: u32 = 180;

/// Reveals a popup with a restrained native fade when Windows permits motion.
///
/// The preference is checked for every reveal because it can change while
/// Adufa is resident. Any API failure falls back to an immediate show.
pub fn show_popup(window: HWND) {
    let already_visible = unsafe { IsWindowVisible(window) }.as_bool();
    let mut animations_enabled = BOOL::default();
    let preference_available = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut animations_enabled as *mut BOOL).cast()),
            Default::default(),
        )
    }
    .is_ok();

    if !already_visible && preference_available && animations_enabled.as_bool() {
        // SAFETY: The live top-level HWND is owned by the UI thread. AW_BLEND
        // changes only visibility/opacity and retains no caller-owned pointer.
        if unsafe { AnimateWindow(window, OPEN_ANIMATION_MS, AW_BLEND) }.is_err() {
            let _ = unsafe { ShowWindow(window, SW_SHOW) };
        }
    } else {
        let _ = unsafe { ShowWindow(window, SW_SHOW) };
    }
    let _ = unsafe { SetForegroundWindow(window) };
}

/// Shows a latency-sensitive contextual surface without the synchronous
/// `AnimateWindow` call used by larger popup views.
pub fn show_context_menu(window: HWND) {
    let _ = unsafe { ShowWindow(window, SW_SHOW) };
    let _ = unsafe { SetForegroundWindow(window) };
}

/// Reveals a taskbar companion without taking activation away from Explorer's
/// native menu.
pub fn show_context_menu_no_activate(window: HWND) {
    let _ = unsafe { ShowWindow(window, SW_SHOWNOACTIVATE) };
}
