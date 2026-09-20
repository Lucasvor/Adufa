//! Process-wide keyboard shortcuts backed by the native Windows hotkey API.
//!
//! A registration is tied to the window that receives `WM_HOTKEY`. Keeping the
//! handle in [`GlobalHotKey`] makes that ownership explicit and guarantees a
//! best-effort `UnregisterHotKey` call when the guard is dropped.

use std::fmt;

use windows::Win32::Foundation::{ERROR_HOTKEY_ALREADY_REGISTERED, HWND, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey, VK_A,
};
use windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY;
use windows_core::{Error as WindowsError, HRESULT};

/// Identifier carried in `WM_HOTKEY` for the quick popup shortcut.
///
/// Application-defined hotkey identifiers must be in the `0x0000..=0xBFFF`
/// range. This value is intentionally stable so message dispatch can remain a
/// small equality check.
pub const QUICK_POPUP_HOTKEY_ID: i32 = 0x0ADF;

/// Virtual-key code for the `A` in the default `Ctrl+Alt+A` shortcut.
pub const QUICK_POPUP_VIRTUAL_KEY: u32 = VK_A.0 as u32;

/// Modifiers for `Ctrl+Alt+A`, including suppression of key-repeat messages.
pub const QUICK_POPUP_MODIFIERS: HOT_KEY_MODIFIERS =
    HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0 | MOD_NOREPEAT.0);

/// Human-readable form used in settings and actionable error messages.
pub const QUICK_POPUP_LABEL: &str = "Ctrl+Alt+A";

/// Returns whether a window message activates Adufa's quick popup shortcut.
pub const fn is_quick_popup_hotkey(message: u32, wparam: WPARAM) -> bool {
    message == WM_HOTKEY && wparam.0 == QUICK_POPUP_HOTKEY_ID as usize
}

/// Owns one native global-hotkey registration.
///
/// Drop performs best-effort cleanup because destructors cannot report an
/// `UnregisterHotKey` failure. Call [`Self::unregister`] when the caller needs
/// explicit shutdown error reporting.
#[derive(Debug)]
pub struct GlobalHotKey {
    window: HWND,
    id: i32,
    registered: bool,
}

impl GlobalHotKey {
    /// Registers the default `Ctrl+Alt+A` shortcut for `window`.
    pub fn register_quick_popup(window: HWND) -> Result<Self, HotKeyError> {
        Self::register(
            window,
            QUICK_POPUP_HOTKEY_ID,
            QUICK_POPUP_MODIFIERS,
            QUICK_POPUP_VIRTUAL_KEY,
        )
    }

    /// Unregisters the shortcut and reports any Windows shutdown failure.
    pub fn unregister(mut self) -> Result<(), HotKeyError> {
        self.unregister_inner()
    }

    fn register(
        window: HWND,
        id: i32,
        modifiers: HOT_KEY_MODIFIERS,
        virtual_key: u32,
    ) -> Result<Self, HotKeyError> {
        // SAFETY: `window` is borrowed only as an opaque handle. Windows copies
        // the scalar id, modifiers, and virtual-key values during this call.
        unsafe { RegisterHotKey(Some(window), id, modifiers, virtual_key) }
            .map_err(HotKeyError::registration)?;

        Ok(Self {
            window,
            id,
            registered: true,
        })
    }

    fn unregister_inner(&mut self) -> Result<(), HotKeyError> {
        if !self.registered {
            return Ok(());
        }

        // SAFETY: the pair matches the successful RegisterHotKey call owned by
        // this guard and remains valid until the owning window is destroyed.
        unsafe { UnregisterHotKey(Some(self.window), self.id) }
            .map_err(HotKeyError::UnregistrationFailed)?;
        self.registered = false;
        Ok(())
    }
}

impl Drop for GlobalHotKey {
    fn drop(&mut self) {
        let _ = self.unregister_inner();
    }
}

/// Errors from acquiring or explicitly releasing the quick popup shortcut.
#[derive(Debug)]
pub enum HotKeyError {
    /// Another application has already claimed `Ctrl+Alt+A`.
    AlreadyRegistered,
    /// Windows rejected registration for another reason.
    RegistrationFailed(WindowsError),
    /// Windows rejected an explicit unregister request.
    UnregistrationFailed(WindowsError),
}

impl HotKeyError {
    fn registration(error: WindowsError) -> Self {
        let already_registered = HRESULT::from_win32(ERROR_HOTKEY_ALREADY_REGISTERED.0);
        if error.code() == already_registered {
            Self::AlreadyRegistered
        } else {
            Self::RegistrationFailed(error)
        }
    }
}

impl fmt::Display for HotKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered => write!(
                formatter,
                "{QUICK_POPUP_LABEL} is already used by another application"
            ),
            Self::RegistrationFailed(error) => write!(
                formatter,
                "Windows could not register {QUICK_POPUP_LABEL}: {error}"
            ),
            Self::UnregistrationFailed(error) => write!(
                formatter,
                "Windows could not unregister {QUICK_POPUP_LABEL}: {error}"
            ),
        }
    }
}

impl std::error::Error for HotKeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AlreadyRegistered => None,
            Self::RegistrationFailed(error) | Self::UnregistrationFailed(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_popup_message_requires_both_message_and_identifier() {
        assert!(is_quick_popup_hotkey(
            WM_HOTKEY,
            WPARAM(QUICK_POPUP_HOTKEY_ID as usize)
        ));
        assert!(!is_quick_popup_hotkey(
            WM_HOTKEY,
            WPARAM((QUICK_POPUP_HOTKEY_ID + 1) as usize)
        ));
        assert!(!is_quick_popup_hotkey(
            WM_HOTKEY + 1,
            WPARAM(QUICK_POPUP_HOTKEY_ID as usize)
        ));
    }

    #[test]
    fn default_shortcut_includes_no_repeat() {
        assert_ne!(QUICK_POPUP_MODIFIERS.0 & MOD_CONTROL.0, 0);
        assert_ne!(QUICK_POPUP_MODIFIERS.0 & MOD_ALT.0, 0);
        assert_ne!(QUICK_POPUP_MODIFIERS.0 & MOD_NOREPEAT.0, 0);
        assert_eq!(QUICK_POPUP_VIRTUAL_KEY, u32::from(b'A'));
    }

    #[test]
    fn collision_error_is_actionable() {
        assert_eq!(
            HotKeyError::AlreadyRegistered.to_string(),
            "Ctrl+Alt+A is already used by another application"
        );
    }
}
