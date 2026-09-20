//! Windows sign-in startup preference.
//!
//! The UI owns only the user's preference. Windows remains responsible for
//! launching the executable through the current user's `Run` registry key.

use std::mem::size_of;
use std::path::Path;

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::w;

const VALUE_NAME: windows::core::PCWSTR = w!("Adufa");
const RUN_KEY: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

pub fn is_enabled() -> Result<bool, String> {
    let key = open_run_key(KEY_READ)?;
    let result = unsafe { RegQueryValueExW(key.0, VALUE_NAME, None, None, None, None) };
    if result == ERROR_SUCCESS {
        Ok(true)
    } else if result == ERROR_FILE_NOT_FOUND {
        Ok(false)
    } else {
        Err(format!(
            "Windows could not read the startup preference ({result:?})."
        ))
    }
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let key = open_run_key(KEY_SET_VALUE)?;
    let result = if enabled {
        let executable = std::env::current_exe()
            .map_err(|error| format!("Windows could not locate Adufa: {error}"))?;
        let value = startup_command(&executable);
        let utf16: Vec<u16> = value.encode_utf16().chain([0]).collect();
        let bytes = unsafe {
            std::slice::from_raw_parts(utf16.as_ptr().cast::<u8>(), utf16.len() * size_of::<u16>())
        };
        unsafe { RegSetValueExW(key.0, VALUE_NAME, None, REG_SZ, Some(bytes)) }
    } else {
        unsafe { RegDeleteValueW(key.0, VALUE_NAME) }
    };

    if result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND) {
        Ok(())
    } else {
        Err(format!(
            "Windows could not update the startup preference ({result:?})."
        ))
    }
}

fn open_run_key(
    access: windows::Win32::System::Registry::REG_SAM_FLAGS,
) -> Result<RegistryKey, String> {
    let mut key = HKEY::default();
    let result = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, access, &mut key) };
    if result == ERROR_SUCCESS {
        Ok(RegistryKey(key))
    } else {
        Err(format!(
            "Windows could not open the user startup settings ({result:?})."
        ))
    }
}

fn startup_command(executable: &Path) -> String {
    format!("\"{}\"", executable.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_command_quotes_paths_with_spaces() {
        assert_eq!(
            startup_command(Path::new(r"C:\Program Files\Adufa\Adufa.exe")),
            r#""C:\Program Files\Adufa\Adufa.exe""#
        );
    }
}
