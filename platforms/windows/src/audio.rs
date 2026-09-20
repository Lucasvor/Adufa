use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;

use router_engine::{ApplicationId, Capabilities, CapabilityAvailability, Observation, OutputId};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Media::Audio::{
    AudioSessionStateActive, DEVICE_STATE_ACTIVE, IAudioSessionControl2, IAudioSessionManager2,
    IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
    CoUninitialize, STGM_READ,
};
use windows::core::{BSTR, Interface, PWSTR};

use crate::process_identity;

type BoxError = Box<dyn Error + Send + Sync>;

const SYSTEM_SOUNDS_ID: &str = "windows:system-sounds";

pub struct WindowsObservation {
    pub observation: Observation,
    pub stats: ScanStats,
    /// Windows-only presentation metadata for every output in the observation.
    pub outputs: Vec<WindowsOutput>,
}

/// Owned Windows output metadata kept outside the cross-platform domain model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowsOutput {
    pub id: OutputId,
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationVolume {
    pub percent: u8,
    pub muted: bool,
}

#[derive(Default)]
pub struct ScanStats {
    pub active_sessions: usize,
    pub skipped_sessions: usize,
    pub endpoint_failures: usize,
    pub inaccessible_processes: usize,
    groups: BTreeMap<String, GroupStats>,
}

impl ScanStats {
    /// Returns the unique process IDs that currently contribute sessions to an
    /// application group. The sorted order makes UI updates deterministic.
    pub fn processes_for(&self, application: &str) -> Vec<u32> {
        self.groups
            .get(application)
            .map_or_else(Vec::new, |group| group.processes.iter().copied().collect())
    }

    /// Returns an executable path suitable for asking the Windows shell for the
    /// application's icon. Packaged identities may legitimately have no path.
    pub fn executable_path_for(&self, application: &str) -> Option<&str> {
        self.groups
            .get(application)
            .and_then(|group| group.executable_path.as_deref())
    }

    pub fn volume_for(&self, application: &str) -> Option<ApplicationVolume> {
        let group = self.groups.get(application)?;
        Some(ApplicationVolume {
            percent: (group.max_volume? * 100.0).round().clamp(0.0, 100.0) as u8,
            muted: group.all_muted,
        })
    }
}

#[derive(Clone, Default)]
struct GroupStats {
    sessions: usize,
    processes: BTreeSet<u32>,
    executable_path: Option<String>,
    max_volume: Option<f32>,
    all_muted: bool,
}

struct ComApartment {
    // COM initialization and teardown must remain on the same thread.
    _not_send: PhantomData<Rc<()>>,
}

impl ComApartment {
    fn initialize_mta() -> windows::core::Result<Self> {
        // SAFETY: The reserved pointer is null as required by CoInitializeEx. This
        // executable initializes COM once on its current thread and the guard calls
        // CoUninitialize on that same thread after all COM interface values are gone.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self {
            _not_send: PhantomData,
        })
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: A successful CoInitializeEx call is paired exactly once with this
        // call, and PhantomData<Rc<()>> prevents moving the guard to another thread.
        unsafe { CoUninitialize() };
    }
}

struct CoTaskMemWide(PWSTR);

impl CoTaskMemWide {
    fn to_string(&self) -> Result<String, BoxError> {
        // SAFETY: IMMDevice::GetId returns a valid null-terminated UTF-16 string
        // allocated with CoTaskMemAlloc. The allocation stays alive through this
        // conversion and is released by this wrapper's Drop implementation.
        Ok(unsafe { self.0.to_string()? })
    }
}

impl Drop for CoTaskMemWide {
    fn drop(&mut self) {
        // SAFETY: This is the original pointer returned by IMMDevice::GetId. No
        // aliases outlive the wrapper and CoTaskMemFree accepts a null pointer.
        unsafe { CoTaskMemFree(Some(self.0.0.cast())) };
    }
}

pub fn observe() -> Result<WindowsObservation, BoxError> {
    let _apartment = ComApartment::initialize_mta()?;

    // SAFETY: COM is initialized as MTA on this thread, aggregation is not used,
    // and windows-rs owns and releases the returned interface reference.
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let default_output = default_output_id(&enumerator);

    // SAFETY: The enumerator is a valid COM interface owned on this initialized
    // thread. The returned collection is reference-counted by windows-rs.
    let collection = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
    // SAFETY: The collection remains alive for this call and owns its item list.
    let count = unsafe { collection.GetCount()? };

    let mut outputs = Vec::with_capacity(count as usize);
    let mut output_metadata = Vec::with_capacity(count as usize);
    let mut groups: BTreeMap<ApplicationId, GroupStats> = BTreeMap::new();
    let mut process_cache: BTreeMap<u32, Option<process_identity::ProcessIdentity>> =
        BTreeMap::new();
    let mut stats = ScanStats::default();

    for index in 0..count {
        // SAFETY: index is less than the count returned by this still-live
        // collection. windows-rs owns the returned interface reference.
        let device = match unsafe { collection.Item(index) } {
            Ok(device) => device,
            Err(_) => {
                stats.endpoint_failures += 1;
                continue;
            }
        };

        let Ok(id) = device_id(&device) else {
            stats.endpoint_failures += 1;
            continue;
        };
        let Ok(id) = OutputId::new(id) else {
            stats.endpoint_failures += 1;
            continue;
        };
        output_metadata.push(build_output_metadata(
            id.clone(),
            device_friendly_name(&device).ok(),
            default_output.as_ref(),
        ));
        outputs.push(id);

        if scan_device_sessions(&device, &mut groups, &mut process_cache, &mut stats).is_err() {
            // A disappearing endpoint cannot invalidate the other endpoints in a
            // complete observation; the next OS event will trigger another scan.
            stats.endpoint_failures += 1;
        }
    }

    stats.groups = groups
        .iter()
        .map(|(id, group)| (id.as_str().to_owned(), group.clone()))
        .collect();

    Ok(WindowsObservation {
        observation: Observation {
            capabilities: Capabilities {
                application_routing: CapabilityAvailability::Available,
            },
            applications: groups.into_keys().collect(),
            outputs,
            system_default: default_output,
        },
        stats,
        outputs: output_metadata,
    })
}

fn default_output_id(enumerator: &IMMDeviceEnumerator) -> Option<OutputId> {
    // SAFETY: The enumerator is valid for this call. Failure is expected when no
    // default render endpoint exists and is represented as None.
    let device = unsafe {
        enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .ok()?
    };
    device_id(&device)
        .ok()
        .and_then(|value| OutputId::new(value).ok())
}

fn device_id(device: &IMMDevice) -> Result<String, BoxError> {
    // SAFETY: device is a live COM interface. GetId returns a CoTaskMem-owned
    // null-terminated string that is immediately placed in an owning wrapper.
    let value = CoTaskMemWide(unsafe { device.GetId()? });
    value.to_string()
}

fn device_friendly_name(device: &IMMDevice) -> Result<String, BoxError> {
    // SAFETY: device is a live COM interface and STGM_READ requests a read-only
    // property store. windows-rs owns the returned interface and PROPVARIANT.
    let store = unsafe { device.OpenPropertyStore(STGM_READ)? };
    // SAFETY: PKEY_Device_FriendlyName is a process-lifetime constant and store is
    // valid for this call. The returned PROPVARIANT clears itself on drop.
    let value = unsafe { store.GetValue(&PKEY_Device_FriendlyName)? };
    let name = BSTR::try_from(&value)?.to_string();
    if name.trim().is_empty() {
        return Err("audio output has an empty friendly name".into());
    }
    Ok(name)
}

fn build_output_metadata(
    id: OutputId,
    friendly_name: Option<String>,
    default_output: Option<&OutputId>,
) -> WindowsOutput {
    let name = friendly_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| id.as_str().to_owned());
    let is_default = default_output == Some(&id);
    WindowsOutput {
        id,
        name,
        is_default,
    }
}

fn scan_device_sessions(
    device: &IMMDevice,
    groups: &mut BTreeMap<ApplicationId, GroupStats>,
    process_cache: &mut BTreeMap<u32, Option<process_identity::ProcessIdentity>>,
    stats: &mut ScanStats,
) -> Result<(), BoxError> {
    // SAFETY: device is live, COM is initialized on this thread, no activation
    // parameters are required, and windows-rs owns the returned manager reference.
    let manager: IAudioSessionManager2 = unsafe { device.Activate(CLSCTX_ALL, None)? };
    // SAFETY: manager is live for the call and windows-rs owns the enumerator.
    let sessions = unsafe { manager.GetSessionEnumerator()? };
    // SAFETY: sessions remains live while its current item count is read.
    let count = unsafe { sessions.GetCount()? };

    for index in 0..count {
        // Each session is independently racy. Failure means that session ended
        // during enumeration, not that the endpoint observation failed.
        // SAFETY: index is within the count returned by this live enumerator.
        let Ok(control) = (unsafe { sessions.GetSession(index) }) else {
            stats.skipped_sessions += 1;
            continue;
        };
        let Ok(control) = control.cast::<IAudioSessionControl2>() else {
            stats.skipped_sessions += 1;
            continue;
        };
        // SAFETY: control is a live IAudioSessionControl2 reference.
        let Ok(state) = (unsafe { control.GetState() }) else {
            stats.skipped_sessions += 1;
            continue;
        };
        if state != AudioSessionStateActive {
            continue;
        }
        stats.active_sessions += 1;

        // SAFETY: control is live; S_OK specifically identifies the system-sounds
        // session while S_FALSE means an ordinary process-backed session.
        let is_system_sounds = unsafe { control.IsSystemSoundsSession() } == S_OK;
        // SAFETY: control remains live and GetProcessId writes into windows-rs-owned
        // result storage. Failure is handled as a disappearing session.
        let pid = match unsafe { control.GetProcessId() } {
            Ok(pid) => pid,
            Err(_) => {
                stats.skipped_sessions += 1;
                continue;
            }
        };

        let identity = if is_system_sounds {
            ApplicationId::new(SYSTEM_SOUNDS_ID)
                .ok()
                .map(|application_id| process_identity::ProcessIdentity {
                    application_id,
                    executable_path: None,
                })
        } else if let Some(cached) = process_cache.get(&pid) {
            cached.clone()
        } else {
            let resolved = process_identity::resolve(pid);
            if resolved.is_none() {
                stats.inaccessible_processes += 1;
            }
            process_cache.insert(pid, resolved.clone());
            resolved
        };

        let Some(identity) = identity else {
            stats.skipped_sessions += 1;
            continue;
        };
        let session_volume = control
            .cast::<ISimpleAudioVolume>()
            .ok()
            .and_then(|volume| {
                let level = unsafe { volume.GetMasterVolume() }.ok()?;
                let muted = unsafe { volume.GetMute() }.ok()?.as_bool();
                Some((level, muted))
            });
        record_session(groups, identity, pid, session_volume);
    }

    Ok(())
}

fn record_session(
    groups: &mut BTreeMap<ApplicationId, GroupStats>,
    identity: process_identity::ProcessIdentity,
    pid: u32,
    volume: Option<(f32, bool)>,
) {
    let group = groups.entry(identity.application_id).or_default();
    group.sessions += 1;
    group.processes.insert(pid);
    if group.executable_path.is_none() {
        group.executable_path = identity.executable_path;
    }
    if let Some((level, muted)) = volume {
        group.all_muted = if group.max_volume.is_none() {
            muted
        } else {
            group.all_muted && muted
        };
        group.max_volume = Some(group.max_volume.map_or(level, |current| current.max(level)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_with_one_identity_form_one_application_group() {
        let id = ApplicationId::new("windows:path:c:\\apps\\browser.exe").unwrap();
        let mut groups = BTreeMap::<ApplicationId, GroupStats>::new();

        for pid in [100, 100, 200] {
            record_session(
                &mut groups,
                process_identity::ProcessIdentity {
                    application_id: id.clone(),
                    executable_path: Some(r"C:\Apps\browser.exe".to_owned()),
                },
                pid,
                None,
            );
        }

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[&id].sessions, 3);
        assert_eq!(groups[&id].processes.len(), 2);
        assert_eq!(
            groups[&id].executable_path.as_deref(),
            Some(r"C:\Apps\browser.exe")
        );
    }

    #[test]
    fn scan_stats_exposes_owned_metadata_for_one_application_group() {
        let id = ApplicationId::new("windows:path:c:\\apps\\player.exe").unwrap();
        let mut groups = BTreeMap::<ApplicationId, GroupStats>::new();
        for pid in [42, 7, 42] {
            record_session(
                &mut groups,
                process_identity::ProcessIdentity {
                    application_id: id.clone(),
                    executable_path: Some(r"C:\Apps\player.exe".to_owned()),
                },
                pid,
                Some((0.42, pid == 7)),
            );
        }
        let stats = ScanStats {
            groups: groups
                .into_iter()
                .map(|(application, group)| (application.as_str().to_owned(), group))
                .collect(),
            ..ScanStats::default()
        };

        assert_eq!(stats.processes_for(id.as_str()), vec![7, 42]);
        assert_eq!(
            stats.executable_path_for(id.as_str()),
            Some(r"C:\Apps\player.exe")
        );
        assert_eq!(
            stats.volume_for(id.as_str()),
            Some(ApplicationVolume {
                percent: 42,
                muted: false,
            })
        );
    }

    #[test]
    fn output_metadata_uses_friendly_name_and_marks_exact_default() {
        let id = OutputId::new("endpoint-a").unwrap();
        let output =
            build_output_metadata(id.clone(), Some("Studio Speakers".to_owned()), Some(&id));

        assert_eq!(output.name, "Studio Speakers");
        assert!(output.is_default);
    }

    #[test]
    fn output_metadata_falls_back_to_stable_id_when_name_is_empty() {
        let id = OutputId::new("endpoint-b").unwrap();
        let output = build_output_metadata(id, Some("   ".to_owned()), None);

        assert_eq!(output.name, "endpoint-b");
        assert!(!output.is_default);
    }
}
