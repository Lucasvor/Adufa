use router_engine::ApplicationId;
use windows::Win32::Foundation::{
    APPMODEL_ERROR_NO_APPLICATION, CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HANDLE,
};
use windows::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::PWSTR;

/// Presentation-neutral process metadata captured while the process handle is
/// available. The executable path is kept out of the shared domain model, but
/// lets the Windows shell resolve the application's real icon.
#[derive(Clone, Debug)]
pub struct ProcessIdentity {
    pub application_id: ApplicationId,
    pub executable_path: Option<String>,
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
}
