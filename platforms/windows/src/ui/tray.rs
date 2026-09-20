//! Windows notification-area integration.
//!
//! The tray boundary owns only the Shell notification record. Visibility and
//! routing remain window concerns, so the shell can be replaced without leaking
//! Windows messages into the domain engine.

use std::mem::size_of;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{HICON, WM_LBUTTONUP, WM_RBUTTONUP};
use windows::core::Error as WindowsError;

pub const CALLBACK_MESSAGE: u32 = 0x8000 + 42;
const ICON_ID: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayEvent {
    Activate,
}

/// A notification-area icon removed automatically during window teardown.
pub struct TrayIcon {
    data: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn add(window: HWND, icon: HICON, tooltip: &str) -> windows::core::Result<Self> {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: window,
            uID: ICON_ID,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: CALLBACK_MESSAGE,
            hIcon: icon,
            ..Default::default()
        };
        write_utf16(&mut data.szTip, tooltip);

        // SAFETY: `data` is fully initialized and remains alive for the
        // synchronous Shell_NotifyIconW call. The shell copies the record.
        if !unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.as_bool() {
            return Err(WindowsError::from_thread());
        }
        // Version 4 gives consistent activation semantics on current Windows.
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        if !unsafe { Shell_NotifyIconW(NIM_SETVERSION, &data) }.as_bool() {
            let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
            return Err(WindowsError::from_thread());
        }
        Ok(Self { data })
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> windows::core::Result<()> {
        write_utf16(&mut self.data.szTip, tooltip);
        self.data.uFlags = NIF_TIP | NIF_SHOWTIP;
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.data) }.as_bool() {
            Ok(())
        } else {
            Err(WindowsError::from_thread())
        }
    }

    pub fn event(lparam: LPARAM) -> Option<TrayEvent> {
        let message = lparam.0 as u32 & 0xFFFF;
        matches!(message, WM_LBUTTONUP | WM_RBUTTONUP).then_some(TrayEvent::Activate)
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // SAFETY: This is the exact owner/id pair registered in `add`. Deletion
        // is idempotent from the application's perspective during shutdown.
        let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &self.data) };
    }
}

fn write_utf16<const N: usize>(target: &mut [u16; N], value: &str) {
    target.fill(0);
    for (destination, source) in target
        .iter_mut()
        .take(N.saturating_sub(1))
        .zip(value.encode_utf16())
    {
        *destination = source;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_primary_and_secondary_tray_activation() {
        assert_eq!(
            TrayIcon::event(LPARAM(WM_LBUTTONUP as isize)),
            Some(TrayEvent::Activate)
        );
        assert_eq!(
            TrayIcon::event(LPARAM(WM_RBUTTONUP as isize)),
            Some(TrayEvent::Activate)
        );
        assert_eq!(
            TrayIcon::event(LPARAM(((ICON_ID << 16) | WM_LBUTTONUP) as isize)),
            Some(TrayEvent::Activate)
        );
        assert_eq!(TrayIcon::event(LPARAM(0)), None);
    }

    #[test]
    fn tooltip_copy_is_null_terminated_and_bounded() {
        let mut buffer = [u16::MAX; 8];
        write_utf16(&mut buffer, "123456789");
        assert_eq!(&buffer[..7], &"1234567".encode_utf16().collect::<Vec<_>>());
        assert_eq!(buffer[7], 0);
    }
}
