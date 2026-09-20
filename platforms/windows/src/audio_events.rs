//! Event-driven bridge from Windows Core Audio changes to the UI thread.
//!
//! The resident worker blocks on a condition variable while audio topology is
//! stable. Native COM callbacks wake it only when a device or session changes,
//! so Adufa does not spend CPU polling while it sits in the tray.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, PROPERTYKEY, WPARAM};
use windows::Win32::Media::Audio::{
    AudioSessionDisconnectReason, AudioSessionState, DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow,
    ERole, IAudioSessionControl, IAudioSessionEvents, IAudioSessionEvents_Impl,
    IAudioSessionManager2, IAudioSessionNotification, IAudioSessionNotification_Impl,
    IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl, MMDeviceEnumerator,
    eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
use windows::core::{BOOL, GUID, PCWSTR, Ref, implement};

/// Private UI message indicating that a fresh audio observation is required.
pub const CHANGE_MESSAGE: u32 = 0x8000 + 38;

const EVENT_SETTLE_DELAY: Duration = Duration::from_millis(150);

type WakeSignal = Arc<(Mutex<bool>, Condvar)>;

/// Owns the callback worker for the lifetime of the application.
pub struct AudioChangeMonitor {
    stop: Arc<AtomicBool>,
    wake: WakeSignal,
    worker: Option<JoinHandle<()>>,
}

impl AudioChangeMonitor {
    pub fn start(window: HWND) -> Result<Self, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let wake = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_stop = Arc::clone(&stop);
        let worker_wake = Arc::clone(&wake);
        let window_address = window.0 as isize;
        let worker = thread::Builder::new()
            .name("adufa-audio-events".to_owned())
            .spawn(move || {
                let window = HWND(window_address as *mut core::ffi::c_void);
                if let Err(error) = watch(window, &worker_stop, &worker_wake) {
                    eprintln!("Audio event monitor stopped: {error}");
                }
            })
            .map_err(|error| format!("Could not start audio event monitoring: {error}"))?;
        Ok(Self {
            stop,
            wake,
            worker: Some(worker),
        })
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        signal(&self.wake);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for AudioChangeMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> windows::core::Result<Self> {
        // SAFETY: One balanced MTA initialization is owned by this worker.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: Paired on the same thread with the successful initialization.
        unsafe { CoUninitialize() };
    }
}

#[implement(IAudioSessionNotification)]
struct SessionChangeSink {
    wake: WakeSignal,
}

impl IAudioSessionNotification_Impl for SessionChangeSink_Impl {
    fn OnSessionCreated(&self, _session: Ref<IAudioSessionControl>) -> windows::core::Result<()> {
        signal(&self.wake);
        Ok(())
    }
}

#[implement(IAudioSessionEvents)]
struct SessionStateSink {
    wake: WakeSignal,
}

impl IAudioSessionEvents_Impl for SessionStateSink_Impl {
    fn OnDisplayNameChanged(
        &self,
        _display_name: &PCWSTR,
        _event_context: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnIconPathChanged(
        &self,
        _icon_path: &PCWSTR,
        _event_context: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        _volume: f32,
        _muted: BOOL,
        event_context: *const GUID,
    ) -> windows::core::Result<()> {
        let event_context = if event_context.is_null() {
            None
        } else {
            Some(unsafe { *event_context })
        };
        if should_signal_volume_change(event_context) {
            signal(&self.wake);
        }
        Ok(())
    }

    fn OnChannelVolumeChanged(
        &self,
        _channel_count: u32,
        _new_channel_volumes: *const f32,
        _changed_channel: u32,
        _event_context: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnGroupingParamChanged(
        &self,
        _grouping: *const GUID,
        _event_context: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnStateChanged(&self, _new_state: AudioSessionState) -> windows::core::Result<()> {
        signal(&self.wake);
        Ok(())
    }

    fn OnSessionDisconnected(
        &self,
        _reason: AudioSessionDisconnectReason,
    ) -> windows::core::Result<()> {
        signal(&self.wake);
        Ok(())
    }
}

fn should_signal_volume_change(event_context: Option<GUID>) -> bool {
    event_context != Some(crate::volume::EVENT_CONTEXT)
}

#[implement(IMMNotificationClient)]
struct DeviceChangeSink {
    wake: WakeSignal,
}

impl DeviceChangeSink_Impl {
    fn changed(&self) -> windows::core::Result<()> {
        signal(&self.wake);
        Ok(())
    }
}

impl IMMNotificationClient_Impl for DeviceChangeSink_Impl {
    fn OnDeviceStateChanged(
        &self,
        _device_id: &PCWSTR,
        _new_state: DEVICE_STATE,
    ) -> windows::core::Result<()> {
        self.changed()
    }

    fn OnDeviceAdded(&self, _device_id: &PCWSTR) -> windows::core::Result<()> {
        self.changed()
    }

    fn OnDeviceRemoved(&self, _device_id: &PCWSTR) -> windows::core::Result<()> {
        self.changed()
    }

    fn OnDefaultDeviceChanged(
        &self,
        _flow: EDataFlow,
        _role: ERole,
        _default_device_id: &PCWSTR,
    ) -> windows::core::Result<()> {
        self.changed()
    }

    fn OnPropertyValueChanged(
        &self,
        _device_id: &PCWSTR,
        _key: &PROPERTYKEY,
    ) -> windows::core::Result<()> {
        self.changed()
    }
}

fn signal(wake: &WakeSignal) {
    let (changed, condition) = &**wake;
    if let Ok(mut changed) = changed.lock() {
        *changed = true;
        condition.notify_one();
    }
}

fn watch(
    window: HWND,
    stop: &AtomicBool,
    wake: &WakeSignal,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _apartment = ComApartment::initialize()?;
    // SAFETY: COM is initialized on this worker and aggregation is not used.
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let device_sink: IMMNotificationClient = DeviceChangeSink {
        wake: Arc::clone(wake),
    }
    .into();
    // SAFETY: The enumerator and callback remain alive through explicit teardown.
    unsafe { enumerator.RegisterEndpointNotificationCallback(&device_sink)? };

    let session_sink: IAudioSessionNotification = SessionChangeSink {
        wake: Arc::clone(wake),
    }
    .into();
    let state_sink: IAudioSessionEvents = SessionStateSink {
        wake: Arc::clone(wake),
    }
    .into();
    let mut registrations = register_audio_notifications(&enumerator, &session_sink, &state_sink);

    let (changed, condition) = &**wake;
    while !stop.load(Ordering::Acquire) {
        let mut pending = changed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*pending && !stop.load(Ordering::Acquire) {
            pending = condition
                .wait(pending)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if stop.load(Ordering::Acquire) {
            break;
        }
        *pending = false;
        drop(pending);

        // Session creation and activation are often separate notifications.
        // This short one-shot delay coalesces the burst without periodic polling.
        thread::sleep(EVENT_SETTLE_DELAY);
        if stop.load(Ordering::Acquire) {
            break;
        }
        registrations.unregister(&session_sink, &state_sink);
        registrations = register_audio_notifications(&enumerator, &session_sink, &state_sink);
        // SAFETY: The private message carries no pointer or borrowed data.
        let _ = unsafe { PostMessageW(Some(window), CHANGE_MESSAGE, WPARAM(0), LPARAM(0)) };
    }

    registrations.unregister(&session_sink, &state_sink);
    // SAFETY: This exactly reverses the registration above while both are live.
    let _ = unsafe { enumerator.UnregisterEndpointNotificationCallback(&device_sink) };
    Ok(())
}

struct AudioRegistrations {
    managers: Vec<IAudioSessionManager2>,
    sessions: Vec<IAudioSessionControl>,
}

impl AudioRegistrations {
    fn unregister(
        &self,
        session_sink: &IAudioSessionNotification,
        state_sink: &IAudioSessionEvents,
    ) {
        for session in &self.sessions {
            // SAFETY: Every retained session successfully registered this sink.
            let _ = unsafe { session.UnregisterAudioSessionNotification(state_sink) };
        }
        for manager in &self.managers {
            // SAFETY: Every retained manager successfully registered this sink.
            let _ = unsafe { manager.UnregisterSessionNotification(session_sink) };
        }
    }
}

fn register_audio_notifications(
    enumerator: &IMMDeviceEnumerator,
    session_sink: &IAudioSessionNotification,
    state_sink: &IAudioSessionEvents,
) -> AudioRegistrations {
    let Ok(devices) = (unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) })
    else {
        return AudioRegistrations {
            managers: Vec::new(),
            sessions: Vec::new(),
        };
    };
    let Ok(count) = (unsafe { devices.GetCount() }) else {
        return AudioRegistrations {
            managers: Vec::new(),
            sessions: Vec::new(),
        };
    };
    let mut managers = Vec::new();
    let mut registered_sessions = Vec::new();
    for index in 0..count {
        let Ok(device) = (unsafe { devices.Item(index) }) else {
            continue;
        };
        let Ok(manager) = (unsafe { device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) })
        else {
            continue;
        };
        // Microsoft requires priming the manager's enumerator before callback registration.
        if let Ok(sessions) = unsafe { manager.GetSessionEnumerator() } {
            if let Ok(session_count) = unsafe { sessions.GetCount() } {
                for session_index in 0..session_count {
                    let Ok(session) = (unsafe { sessions.GetSession(session_index) }) else {
                        continue;
                    };
                    if unsafe { session.RegisterAudioSessionNotification(state_sink) }.is_ok() {
                        registered_sessions.push(session);
                    }
                }
            }
        }
        if unsafe { manager.RegisterSessionNotification(session_sink) }.is_ok() {
            managers.push(manager);
        }
    }
    AudioRegistrations {
        managers,
        sessions: registered_sessions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adufa_volume_events_do_not_trigger_a_full_model_refresh() {
        assert!(!should_signal_volume_change(Some(
            crate::volume::EVENT_CONTEXT
        )));
        assert!(should_signal_volume_change(None));
        assert!(should_signal_volume_change(Some(GUID::from_u128(
            0x8b8c65ec_794d_4be5_9012_59af48e32412
        ))));
    }

    #[test]
    fn signal_marks_pending_work_without_polling() {
        let wake = Arc::new((Mutex::new(false), Condvar::new()));
        signal(&wake);
        assert!(*wake.0.lock().unwrap());
    }
}
