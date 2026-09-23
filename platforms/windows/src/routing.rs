//! Windows per-process output routing.
//!
//! Windows does not expose per-application output selection through the public
//! Core Audio API. The system Settings application uses the internal
//! `Windows.Media.Internal.AudioPolicyConfig` WinRT activation factory instead.
//! This module deliberately keeps that compatibility boundary small: callers
//! provide process IDs and either one raw MMDevice ID or `None` for the system
//! default. No internal Windows ABI details escape into the engine or UI.

use std::ffi::c_void;
use std::fmt;

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::WinRT::{
    RO_INIT_MULTITHREADED, RoGetActivationFactory, RoInitialize, RoUninitialize,
};
use windows::core::{HRESULT, HSTRING, Interface};

const AUDIO_POLICY_RUNTIME_CLASS: &str = "Windows.Media.Internal.AudioPolicyConfig";
const RENDER_DEVICE_INTERFACE_CLASS: &str = "{e6327cad-dcec-4949-ae8a-991e976a79d2}";

// These integer values are part of the stable Core Audio EDataFlow/ERole ABI.
const E_RENDER: i32 = 0;
const E_CONSOLE: i32 = 0;
const E_MULTIMEDIA: i32 = 1;
const E_INVALIDARG: HRESULT = HRESULT(0x8007_0057_u32 as i32);

/// A failure while applying a persisted Windows per-process output selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingError {
    /// The caller supplied no process to route.
    NoProcesses,
    /// Process ID zero is not a routable application process.
    InvalidProcessId,
    /// A non-default output selection contained an empty MMDevice ID.
    InvalidOutputDeviceId,
    /// The Windows Runtime could not be initialized on the calling thread.
    RuntimeInitialization { code: HRESULT },
    /// Neither known version of the private activation factory was available.
    ActivationFactoryUnavailable {
        current_code: HRESULT,
        downlevel_code: HRESULT,
    },
    /// Windows rejected one process/role assignment.
    AssignmentFailed {
        process_id: u32,
        role: AudioRole,
        code: HRESULT,
    },
    /// Windows could not return a persisted per-process route.
    ReadFailed {
        process_id: u32,
        role: AudioRole,
        code: HRESULT,
    },
    /// Different processes or audio roles reported different explicit outputs.
    InconsistentAssignments,
}

impl fmt::Display for RoutingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProcesses => formatter.write_str("no application processes were supplied"),
            Self::InvalidProcessId => formatter.write_str("process ID zero cannot be routed"),
            Self::InvalidOutputDeviceId => {
                formatter.write_str("the selected output device ID is empty")
            }
            Self::RuntimeInitialization { code } => {
                write!(
                    formatter,
                    "Windows Runtime initialization failed ({code:?})"
                )
            }
            Self::ActivationFactoryUnavailable {
                current_code,
                downlevel_code,
            } => write!(
                formatter,
                "Windows audio policy activation factory is unavailable (current: {current_code:?}, downlevel: {downlevel_code:?})"
            ),
            Self::AssignmentFailed {
                process_id,
                role,
                code,
            } => write!(
                formatter,
                "Windows rejected output routing for process {process_id} ({role}, {code:?})"
            ),
            Self::ReadFailed {
                process_id,
                role,
                code,
            } => write!(
                formatter,
                "Windows could not read output routing for process {process_id} ({role}, {code:?})"
            ),
            Self::InconsistentAssignments => {
                formatter.write_str("Windows reported inconsistent outputs for one application")
            }
        }
    }
}

impl std::error::Error for RoutingError {}

/// The Windows audio roles updated for each routed process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRole {
    Multimedia,
    Console,
}

impl AudioRole {
    const fn as_abi(self) -> i32 {
        match self {
            Self::Multimedia => E_MULTIMEDIA,
            Self::Console => E_CONSOLE,
        }
    }
}

impl fmt::Display for AudioRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Multimedia => formatter.write_str("multimedia"),
            Self::Console => formatter.write_str("console"),
        }
    }
}

/// Persist the selected render endpoint for every supplied application process.
///
/// `output_device_id` is the raw ID returned by `IMMDevice::GetId`. Passing
/// `None` removes the application's explicit choice and returns it to the
/// system default. Windows stores the selection for future streams; whether an
/// already-running stream moves immediately remains application-dependent.
///
/// This function updates both the multimedia and console roles because desktop
/// applications may create either kind of stream. If Windows rejects an
/// assignment after earlier assignments succeeded, the function reports the
/// exact process and role; Windows does not offer a transactional batch API.
pub fn route_processes(
    process_ids: &[u32],
    output_device_id: Option<&str>,
) -> Result<(), RoutingError> {
    validate_request(process_ids, output_device_id)?;

    // WinRT initialization is thread-affine. Keep the matching uninitialize
    // guard alive through factory acquisition and all ABI calls.
    let _runtime = RuntimeApartment::initialize()?;
    let factory = AudioPolicyFactory::activate()?;
    let device_path = output_device_id.map(render_device_path).map(HSTRING::from);

    for &process_id in process_ids {
        for role in [AudioRole::Multimedia, AudioRole::Console] {
            match factory.set_persisted_default(process_id, role, device_path.as_ref()) {
                Ok(()) => {}
                Err(code) if is_ignorable_assignment_error(code) => {}
                Err(code) => {
                    return Err(RoutingError::AssignmentFailed {
                        process_id,
                        role,
                        code,
                    });
                }
            }
        }
    }

    Ok(())
}

fn is_ignorable_assignment_error(code: HRESULT) -> bool {
    code == E_INVALIDARG
}

/// Reads the explicit output that Windows has persisted for an application.
///
/// This private Windows interface is not reliable enough to be Adufa's only
/// store. It is used to import routes created by Windows Settings or another
/// mixer. `None` means Windows reports no explicit output for any supplied PID.
pub fn persisted_output_for_processes(process_ids: &[u32]) -> Result<Option<String>, RoutingError> {
    validate_processes(process_ids)?;
    let _runtime = RuntimeApartment::initialize()?;
    let factory = AudioPolicyFactory::activate()?;
    let mut selected: Option<String> = None;

    for &process_id in process_ids {
        for role in [AudioRole::Multimedia, AudioRole::Console] {
            let Some(output) = factory
                .get_persisted_default(process_id, role)
                .map_err(|code| RoutingError::ReadFailed {
                    process_id,
                    role,
                    code,
                })?
            else {
                continue;
            };

            if selected.as_ref().is_some_and(|known| known != &output) {
                return Err(RoutingError::InconsistentAssignments);
            }
            selected = Some(output);
        }
    }

    Ok(selected)
}

fn validate_request(
    process_ids: &[u32],
    output_device_id: Option<&str>,
) -> Result<(), RoutingError> {
    validate_processes(process_ids)?;
    if output_device_id.is_some_and(str::is_empty) {
        return Err(RoutingError::InvalidOutputDeviceId);
    }
    Ok(())
}

fn validate_processes(process_ids: &[u32]) -> Result<(), RoutingError> {
    if process_ids.is_empty() {
        return Err(RoutingError::NoProcesses);
    }
    if process_ids.contains(&0) {
        return Err(RoutingError::InvalidProcessId);
    }
    Ok(())
}

fn render_device_path(raw_device_id: &str) -> String {
    format!(r"\\?\SWD#MMDEVAPI#{raw_device_id}#{RENDER_DEVICE_INTERFACE_CLASS}")
}

struct RuntimeApartment {
    uninitialize: bool,
}

impl RuntimeApartment {
    fn initialize() -> Result<Self, RoutingError> {
        // SAFETY: RoInitialize is called on this thread and the successful call
        // is balanced by this guard's Drop implementation on the same thread.
        match unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
            Ok(()) => Ok(Self { uninitialize: true }),
            Err(error) if error.code() == RPC_E_CHANGED_MODE => {
                // The host may already own an STA (for example, the Win32 UI
                // thread). WinRT is initialized in that case; we must neither
                // change nor uninitialize the host's apartment.
                Ok(Self {
                    uninitialize: false,
                })
            }
            Err(error) => Err(RoutingError::RuntimeInitialization { code: error.code() }),
        }
    }
}

impl Drop for RuntimeApartment {
    fn drop(&mut self) {
        if self.uninitialize {
            // SAFETY: This balances the successful RoInitialize call made by
            // RuntimeApartment::initialize on the same thread.
            unsafe { RoUninitialize() };
        }
    }
}

enum AudioPolicyFactory {
    Current(IAudioPolicyConfigFactoryCurrent),
    Downlevel(IAudioPolicyConfigFactoryDownlevel),
}

impl AudioPolicyFactory {
    fn activate() -> Result<Self, RoutingError> {
        let runtime_class = HSTRING::from(AUDIO_POLICY_RUNTIME_CLASS);

        // Capability probing is more robust than OS-version checks. Microsoft
        // changed the factory IID in Windows 10 21H2 while retaining the same
        // method layout used here.
        let current =
            unsafe { RoGetActivationFactory::<IAudioPolicyConfigFactoryCurrent>(&runtime_class) };
        match current {
            Ok(factory) => Ok(Self::Current(factory)),
            Err(current_error) => {
                let downlevel = unsafe {
                    RoGetActivationFactory::<IAudioPolicyConfigFactoryDownlevel>(&runtime_class)
                };
                match downlevel {
                    Ok(factory) => Ok(Self::Downlevel(factory)),
                    Err(downlevel_error) => Err(RoutingError::ActivationFactoryUnavailable {
                        current_code: current_error.code(),
                        downlevel_code: downlevel_error.code(),
                    }),
                }
            }
        }
    }

    fn set_persisted_default(
        &self,
        process_id: u32,
        role: AudioRole,
        device_path: Option<&HSTRING>,
    ) -> Result<(), HRESULT> {
        match self {
            Self::Current(factory) => set_persisted_default(factory, process_id, role, device_path),
            Self::Downlevel(factory) => {
                set_persisted_default(factory, process_id, role, device_path)
            }
        }
    }

    fn get_persisted_default(
        &self,
        process_id: u32,
        role: AudioRole,
    ) -> Result<Option<String>, HRESULT> {
        match self {
            Self::Current(factory) => get_persisted_default(factory, process_id, role),
            Self::Downlevel(factory) => get_persisted_default(factory, process_id, role),
        }
    }
}

trait AudioPolicyInterface: Interface<Vtable = IAudioPolicyConfigFactory_Vtbl> {}

impl AudioPolicyInterface for IAudioPolicyConfigFactoryCurrent {}
impl AudioPolicyInterface for IAudioPolicyConfigFactoryDownlevel {}

fn set_persisted_default<T: AudioPolicyInterface>(
    factory: &T,
    process_id: u32,
    role: AudioRole,
    device_path: Option<&HSTRING>,
) -> Result<(), HRESULT> {
    // The WinRT ABI represents an HSTRING as an opaque pointer. A null handle
    // has the documented AudioPolicyConfig meaning "follow system default".
    let device_path_abi = device_path.map_or(std::ptr::null_mut(), |path| {
        // SAFETY: HSTRING is a transparent owning wrapper around the WinRT
        // handle. We copy only its handle value and keep `path` alive for the
        // duration of the synchronous COM call.
        unsafe { std::mem::transmute_copy::<HSTRING, *mut c_void>(path) }
    });

    // SAFETY: Both interface declarations use the verified AudioPolicyConfig
    // IInspectable layout. The 19 reserved slots place this method at ABI slot
    // 25 (IUnknown 3 + IInspectable 3 + 19), and all parameters match the
    // Windows ABI. The COM object and optional HSTRING outlive the call.
    let result = unsafe {
        (Interface::vtable(factory).set_persisted_default_audio_endpoint)(
            Interface::as_raw(factory),
            process_id,
            E_RENDER,
            role.as_abi(),
            device_path_abi,
        )
    };

    result.ok().map_err(|error| error.code())
}

fn get_persisted_default<T: AudioPolicyInterface>(
    factory: &T,
    process_id: u32,
    role: AudioRole,
) -> Result<Option<String>, HRESULT> {
    let mut device_path_abi = std::ptr::null_mut();
    let result = unsafe {
        (Interface::vtable(factory).get_persisted_default_audio_endpoint)(
            Interface::as_raw(factory),
            process_id,
            E_RENDER,
            role.as_abi(),
            &mut device_path_abi,
        )
    };
    result.ok().map_err(|error| error.code())?;
    if device_path_abi.is_null() {
        return Ok(None);
    }

    // SAFETY: A successful WinRT out-HSTRING call transfers one owned handle to
    // the caller. HSTRING takes that ownership and releases it exactly once.
    let device_path = unsafe { std::mem::transmute::<*mut c_void, HSTRING>(device_path_abi) };
    let device_path = device_path.to_string_lossy();
    if device_path.is_empty() {
        Ok(None)
    } else {
        Ok(raw_device_id(&device_path).map(str::to_owned))
    }
}

fn raw_device_id(device_path: &str) -> Option<&str> {
    const PREFIX: &str = r"\\?\SWD#MMDEVAPI#";
    const SUFFIX: &str = "#{e6327cad-dcec-4949-ae8a-991e976a79d2}";
    if device_path.len() <= PREFIX.len() + SUFFIX.len()
        || !device_path[..PREFIX.len()].eq_ignore_ascii_case(PREFIX)
        || !device_path[device_path.len() - SUFFIX.len()..].eq_ignore_ascii_case(SUFFIX)
    {
        return None;
    }
    Some(&device_path[PREFIX.len()..device_path.len() - SUFFIX.len()])
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct IAudioPolicyConfigFactoryCurrent(windows::core::IUnknown);

// SAFETY: The IID and vtable layout below are the exact ABI requested from
// RoGetActivationFactory. The transparent IUnknown field supplies correct COM
// reference counting and pointer representation.
unsafe impl Interface for IAudioPolicyConfigFactoryCurrent {
    type Vtable = IAudioPolicyConfigFactory_Vtbl;
    const IID: windows::core::GUID =
        windows::core::GUID::from_u128(0xab3d4648_e242_459f_b02f_541c70306324);
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct IAudioPolicyConfigFactoryDownlevel(windows::core::IUnknown);

// SAFETY: See the current-interface implementation above. Windows versions
// before 21H2 expose the same method layout under this earlier IID.
unsafe impl Interface for IAudioPolicyConfigFactoryDownlevel {
    type Vtable = IAudioPolicyConfigFactory_Vtbl;
    const IID: windows::core::GUID =
        windows::core::GUID::from_u128(0x2a59116d_6c4f_45e0_a74f_707e3fef9258);
}

/// Minimal ABI projection for the internal AudioPolicyConfig factory.
///
/// We intentionally model the preceding methods as opaque pointer-sized slots:
/// they are never invoked, and only their positions are relevant. If Windows
/// changes this private layout, activation/calls will fail at this one boundary
/// instead of contaminating the rest of the application architecture.
#[repr(C)]
struct IAudioPolicyConfigFactory_Vtbl {
    pub base__: windows::core::IInspectable_Vtbl,
    reserved: [usize; 19],
    set_persisted_default_audio_endpoint: unsafe extern "system" fn(
        this: *mut c_void,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: *mut c_void,
    ) -> HRESULT,
    get_persisted_default_audio_endpoint: unsafe extern "system" fn(
        this: *mut c_void,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: *mut *mut c_void,
    ) -> HRESULT,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_render_interface_path_without_changing_raw_device_id() {
        let raw_id = "{0.0.0.00000000}.{01234567-89ab-cdef-0123-456789abcdef}";

        assert_eq!(
            render_device_path(raw_id),
            r"\\?\SWD#MMDEVAPI#{0.0.0.00000000}.{01234567-89ab-cdef-0123-456789abcdef}#{e6327cad-dcec-4949-ae8a-991e976a79d2}"
        );
    }

    #[test]
    fn extracts_raw_device_id_from_windows_policy_path() {
        let path = r"\\?\SWD#MMDEVAPI#{0.0.0.00000000}.{01234567-89ab-cdef-0123-456789abcdef}#{E6327CAD-DCEC-4949-AE8A-991E976A79D2}";
        assert_eq!(
            raw_device_id(path),
            Some("{0.0.0.00000000}.{01234567-89ab-cdef-0123-456789abcdef}")
        );
    }

    #[test]
    fn rejects_unrelated_policy_paths() {
        assert_eq!(raw_device_id("not-an-mmdevice-path"), None);
    }

    #[test]
    fn rejects_requests_that_cannot_target_an_application() {
        assert_eq!(validate_request(&[], None), Err(RoutingError::NoProcesses));
        assert_eq!(
            validate_request(&[0], None),
            Err(RoutingError::InvalidProcessId)
        );
        assert_eq!(
            validate_request(&[42], Some("")),
            Err(RoutingError::InvalidOutputDeviceId)
        );
    }

    #[test]
    fn accepts_system_default_and_explicit_outputs() {
        assert_eq!(validate_request(&[42, 43], None), Ok(()));
        assert_eq!(validate_request(&[42], Some("raw-mmdevice-id")), Ok(()));
    }

    #[test]
    fn invalid_argument_is_ignorable_for_processes_without_audio_sessions() {
        assert!(is_ignorable_assignment_error(HRESULT(
            0x8007_0057_u32 as i32
        )));
        assert!(!is_ignorable_assignment_error(HRESULT(
            0x8007_0005_u32 as i32
        )));
    }
}
