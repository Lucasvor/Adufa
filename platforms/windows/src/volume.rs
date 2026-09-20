//! Windows per-application session volume adapter.
//!
//! The interface accepts process IDs and one normalized value. Endpoint and
//! session enumeration stay private so callers do not retain COM interfaces or
//! need to understand how one application can span several audio sessions.

use std::collections::BTreeSet;

use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
    ISimpleAudioVolume, MMDeviceEnumerator, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::GUID;
use windows::core::Interface;

/// Identifies changes initiated by Adufa so its Core Audio observer can avoid
/// rebuilding the model for state the selector already updated optimistically.
pub static EVENT_CONTEXT: GUID = GUID::from_u128(0x545f5928_f2c3_4ed0_b350_695684f45baa);

struct ComApartment;

impl ComApartment {
    fn initialize() -> windows::core::Result<Self> {
        // SAFETY: This synchronous adapter balances COM on the calling UI thread.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: Paired with the successful initialization on this same thread.
        unsafe { CoUninitialize() };
    }
}

pub fn set_for_processes(
    process_ids: &[u32],
    volume_percent: u8,
    muted: bool,
) -> Result<(), String> {
    let targets: BTreeSet<_> = process_ids.iter().copied().collect();
    if targets.is_empty() {
        return Err("The application has no active audio sessions".to_owned());
    }

    let _apartment = ComApartment::initialize().map_err(|error| error.to_string())?;
    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|error| error.to_string())?
    };
    let endpoints = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|error| error.to_string())?
    };
    let endpoint_count = unsafe { endpoints.GetCount().map_err(|error| error.to_string())? };
    let level = normalized_level(volume_percent);
    let mut updated = 0_usize;
    let mut last_error = None;

    for endpoint_index in 0..endpoint_count {
        let result = (|| -> windows::core::Result<()> {
            let endpoint = unsafe { endpoints.Item(endpoint_index)? };
            let manager: IAudioSessionManager2 = unsafe { endpoint.Activate(CLSCTX_ALL, None)? };
            let sessions = unsafe { manager.GetSessionEnumerator()? };
            let session_count = unsafe { sessions.GetCount()? };

            for session_index in 0..session_count {
                let control = match unsafe { sessions.GetSession(session_index) } {
                    Ok(control) => control,
                    Err(error) => {
                        last_error = Some(error);
                        continue;
                    }
                };
                let control = match control.cast::<IAudioSessionControl2>() {
                    Ok(control) => control,
                    Err(error) => {
                        last_error = Some(error);
                        continue;
                    }
                };
                let process_id = match unsafe { control.GetProcessId() } {
                    Ok(process_id) => process_id,
                    Err(error) => {
                        last_error = Some(error);
                        continue;
                    }
                };
                if !targets.contains(&process_id) {
                    continue;
                }

                let volume = match control.cast::<ISimpleAudioVolume>() {
                    Ok(volume) => volume,
                    Err(error) => {
                        last_error = Some(error);
                        continue;
                    }
                };
                let changed = unsafe {
                    volume
                        .SetMasterVolume(level, &raw const EVENT_CONTEXT)
                        .and_then(|()| volume.SetMute(muted, &raw const EVENT_CONTEXT))
                };
                match changed {
                    Ok(()) => updated += 1,
                    Err(error) => last_error = Some(error),
                }
            }
            Ok(())
        })();

        if let Err(error) = result {
            last_error = Some(error);
        }
    }

    if updated > 0 {
        Ok(())
    } else if let Some(error) = last_error {
        Err(format!(
            "Windows could not update the audio session: {error}"
        ))
    } else {
        Err("The application no longer has an active audio session".to_owned())
    }
}

fn normalized_level(volume_percent: u8) -> f32 {
    f32::from(volume_percent.min(100)) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_percentage_is_normalized_and_clamped() {
        assert_eq!(normalized_level(0), 0.0);
        assert_eq!(normalized_level(37), 0.37);
        assert_eq!(normalized_level(100), 1.0);
        assert_eq!(normalized_level(255), 1.0);
    }

    #[test]
    #[ignore = "requires a live Windows audio session"]
    fn live_session_accepts_its_current_volume_and_mute_state() {
        let before = crate::audio::observe().expect("audio observation");
        let (application_id, current, process_ids) = before
            .observation
            .applications
            .iter()
            .find_map(|application| {
                let application_id = application.as_str();
                Some((
                    application_id.to_owned(),
                    before.stats.volume_for(application_id)?,
                    before.stats.processes_for(application_id),
                ))
            })
            .expect("an active application volume");

        set_for_processes(&process_ids, current.percent, current.muted)
            .expect("round-trip volume update");

        let after = crate::audio::observe().expect("audio observation after update");
        let observed = after
            .stats
            .volume_for(&application_id)
            .expect("application remains active");
        assert!(observed.percent.abs_diff(current.percent) <= 1);
        assert_eq!(observed.muted, current.muted);
    }
}
