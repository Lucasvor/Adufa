use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use router_engine::ApplicationId;
use windows::Win32::Foundation::{
    APPMODEL_ERROR_NO_APPLICATION, CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HANDLE,
    HWND, LPARAM, PROPERTYKEY,
};
use windows::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId};
use windows::core::{BOOL, BSTR, GUID, PWSTR};

const PKEY_APP_USER_MODEL_ID: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
    pid: 5,
};

/// Presentation-neutral process metadata captured while the process handle is
/// available. The executable path is kept out of the shared domain model, but
/// lets the Windows shell resolve the application's real icon.
#[derive(Clone, Debug)]
pub struct ProcessIdentity {
    pub application_id: ApplicationId,
    pub executable_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RunningApplication {
    pub application_id: ApplicationId,
    pub executable_path: Option<String>,
    pub process_ids: Vec<u32>,
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: OpenProcess returned this owned handle and it is closed exactly
        // once here after all queries that borrow it have completed.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// Resolves the strongest stable identity available for a process.
///
/// A packaged application ID takes precedence. The canonical executable path is
/// still queried when accessible and is the fallback for unpackaged applications.
/// Failure is expected for protected or already-terminated processes.
pub fn resolve(pid: u32) -> Option<ProcessIdentity> {
    // SAFETY: Only query rights are requested, handle inheritance is disabled, and
    // the returned owned handle is closed by OwnedHandle on every return path.
    let process =
        OwnedHandle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? });
    let application_user_model_id = application_user_model_id(process.0);
    let executable_path = executable_path(process.0);
    let application_id = build_application_id(
        application_user_model_id.as_deref(),
        executable_path.as_deref(),
    )?;
    Some(ProcessIdentity {
        application_id,
        executable_path,
    })
}

pub fn find_running_application(display_name: &str) -> Option<RunningApplication> {
    let display_name = normalize_label(display_name);
    if display_name.is_empty() {
        return None;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()? };
    let _snapshot = OwnedHandle(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_err() {
        return None;
    }

    let mut matches = BTreeMap::<ApplicationId, RunningApplication>::new();
    loop {
        let executable = String::from_utf16_lossy(&entry.szExeFile)
            .trim_end_matches('\0')
            .to_owned();
        let stem = Path::new(&executable)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let executable_matches = labels_match(&display_name, stem);
        let identity = (executable_matches || may_have_branded_install_path(stem))
            .then(|| resolve(entry.th32ProcessID))
            .flatten();
        let path_matches = identity
            .as_ref()
            .and_then(|identity| identity.executable_path.as_deref())
            .is_some_and(|path| labels_match_path(&display_name, path));
        if (executable_matches || path_matches)
            && let Some(identity) = identity
        {
            let application = matches
                .entry(identity.application_id.clone())
                .or_insert_with(|| RunningApplication {
                    application_id: identity.application_id.clone(),
                    executable_path: identity.executable_path.clone(),
                    process_ids: Vec::new(),
                });
            application.process_ids.push(entry.th32ProcessID);
        }

        if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
            break;
        }
    }

    matches.into_values().next()
}

pub fn find_running_application_by_aumid(value: &str) -> Option<RunningApplication> {
    let app_user_model_id = value.strip_prefix("Appid:").map_or(value, str::trim).trim();
    if app_user_model_id.is_empty() {
        return None;
    }

    let _apartment = ComApartment::initialize().ok()?;
    let mut search = AppUserModelIdSearch {
        app_user_model_id,
        process_ids: BTreeSet::new(),
    };
    unsafe {
        EnumWindows(
            Some(find_app_user_model_id_window),
            LPARAM((&mut search as *mut AppUserModelIdSearch).cast::<()>() as isize),
        )
        .ok()?;
    }

    let mut matches = BTreeMap::<ApplicationId, RunningApplication>::new();
    for process_id in search.process_ids {
        let Some(identity) = resolve(process_id) else {
            continue;
        };
        let application = matches
            .entry(identity.application_id.clone())
            .or_insert_with(|| RunningApplication {
                application_id: identity.application_id.clone(),
                executable_path: identity.executable_path.clone(),
                process_ids: Vec::new(),
            });
        application.process_ids.push(process_id);
    }
    matches.into_values().next()
}

struct AppUserModelIdSearch<'a> {
    app_user_model_id: &'a str,
    process_ids: BTreeSet<u32>,
}

unsafe extern "system" fn find_app_user_model_id_window(window: HWND, parameter: LPARAM) -> BOOL {
    let search = unsafe { &mut *(parameter.0 as *mut AppUserModelIdSearch<'_>) };
    let Ok(store) = (unsafe { SHGetPropertyStoreForWindow::<IPropertyStore>(window) }) else {
        return BOOL(1);
    };
    let Ok(value) = (unsafe { store.GetValue(&PKEY_APP_USER_MODEL_ID) }) else {
        return BOOL(1);
    };
    let Ok(value) = BSTR::try_from(&value) else {
        return BOOL(1);
    };
    if value != search.app_user_model_id {
        return BOOL(1);
    }

    let mut process_id = 0;
    if unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) } != 0 && process_id != 0 {
        search.process_ids.insert(process_id);
    }
    BOOL(1)
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> windows::core::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn normalize_label(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn labels_match(display_name: &str, executable_stem: &str) -> bool {
    let executable_stem = normalize_label(executable_stem);
    if executable_stem.is_empty() {
        return false;
    }

    display_name == executable_stem
        || display_name.contains(&executable_stem)
        || executable_stem.contains(display_name)
        || match executable_stem.as_str() {
            "chrome" => display_name.contains("googlechrome"),
            "msedge" => display_name.contains("microsoftedge"),
            "code" => display_name.contains("visualstudiocode") || display_name.contains("vscode"),
            "explorer" => display_name.contains("fileexplorer"),
            _ => false,
        }
}

fn labels_match_path(display_name: &str, executable_path: &str) -> bool {
    normalize_label(executable_path).contains(display_name)
}

fn may_have_branded_install_path(executable_stem: &str) -> bool {
    matches!(
        normalize_label(executable_stem).as_str(),
        "applicationframehost" | "chrome" | "msedge"
    )
}

fn application_user_model_id(process: HANDLE) -> Option<String> {
    let mut length = 0;
    // SAFETY: A null output buffer is required for the size query; length points to
    // writable storage and process remains open throughout both calls.
    let first = unsafe { GetApplicationUserModelId(process, &mut length, None) };
    if first == APPMODEL_ERROR_NO_APPLICATION {
        return None;
    }
    if first != ERROR_INSUFFICIENT_BUFFER || length == 0 {
        return None;
    }

    let mut buffer = vec![0_u16; length as usize];
    // SAFETY: buffer contains length writable UTF-16 elements, length is passed by
    // address as required, and process remains valid for this call.
    let result = unsafe {
        GetApplicationUserModelId(process, &mut length, Some(PWSTR(buffer.as_mut_ptr())))
    };
    if result != ERROR_SUCCESS || length == 0 {
        return None;
    }
    let content_length = (length as usize).saturating_sub(1);
    String::from_utf16(&buffer[..content_length]).ok()
}

fn executable_path(process: HANDLE) -> Option<String> {
    // QueryFullProcessImageNameW documents 32,767 UTF-16 code units as the maximum
    // Windows path length, so one bounded allocation covers the native contract.
    let mut buffer = vec![0_u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: process has query rights, buffer is writable for length elements, and
    // the mutable length pointer remains valid for the duration of the call.
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
        .ok()?;
    }
    String::from_utf16(&buffer[..length as usize]).ok()
}

fn build_application_id(
    aumid: Option<&str>,
    executable_path: Option<&str>,
) -> Option<ApplicationId> {
    if let Some(aumid) = aumid.filter(|value| !value.is_empty()) {
        return ApplicationId::new(format!("windows:aumid:{aumid}")).ok();
    }
    let path = executable_path.filter(|value| !value.is_empty())?;
    // QueryFullProcessImageNameW already returns the operating system's canonical
    // spelling. Preserve case because Windows supports case-sensitive directories.
    ApplicationId::new(format!("windows:path:{}", path.replace('/', "\\"))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_identity_wins_over_executable_path() {
        let id = build_application_id(
            Some("Publisher.App_123!Player"),
            Some(r"C:\Program Files\App\player.exe"),
        )
        .unwrap();

        assert_eq!(id.as_str(), "windows:aumid:Publisher.App_123!Player");
    }

    #[test]
    fn executable_path_fallback_normalizes_separator_without_losing_case() {
        let first = build_application_id(None, Some(r"C:\Apps\Player.EXE")).unwrap();
        let second = build_application_id(None, Some("C:/Apps/Player.EXE")).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.as_str(), r"windows:path:C:\Apps\Player.EXE");
    }

    #[test]
    fn taskbar_label_matches_chrome_without_active_audio() {
        assert!(labels_match(&normalize_label("Google Chrome"), "chrome"));
    }

    #[test]
    fn taskbar_label_matches_edge_alias() {
        assert!(labels_match(&normalize_label("Microsoft Edge"), "msedge"));
    }

    #[test]
    fn taskbar_label_matches_a_chromium_browser_from_its_install_path() {
        assert!(labels_match_path(
            &normalize_label("Helium"),
            r"C:\Users\Lucas\AppData\Local\imput\Helium\Application\chrome.exe"
        ));
    }

    #[test]
    fn taskbar_app_id_strips_the_ui_automation_prefix() {
        assert_eq!(
            "Appid: Chrome._crx_hnpfjngllnfapefoaidbinmjnm"
                .strip_prefix("Appid:")
                .map_or("", str::trim),
            "Chrome._crx_hnpfjngllnfapefoaidbinmjnm"
        );
    }
}
