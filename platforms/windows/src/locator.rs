//! Windows Core Audio peak monitoring for the temporary Sound Locator mode.
//!
//! The worker owns every COM interface it uses. The UI receives owned peak
//! snapshots through a channel and is only woken by a lightweight window
//! message, which keeps COM and rendering concerns on their respective threads.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, S_OK, WPARAM};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IAudioSessionControl, IAudioSessionControl2, IAudioSessionManager2,
    IAudioSessionNotification, IAudioSessionNotification_Impl, IMMDeviceEnumerator,
    MMDeviceEnumerator, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
use windows::core::{Interface, Ref, implement};

/// Private message posted whenever a fresh peak snapshot is available.
pub const UPDATE_MESSAGE: u32 = 0x8000 + 37;

const SAMPLE_INTERVAL: Duration = Duration::from_millis(50);
const SOURCE_SWITCH_DELAY: Duration = Duration::from_millis(300);
const SILENCE_HOLD: Duration = Duration::from_secs(1);
const SILENCE_THRESHOLD: f32 = 0.003_162_277_6; // -50 dBFS as a linear amplitude.

/// Stable application metadata copied into the monitoring worker.
#[derive(Clone, Debug)]
pub struct MonitoredApplication {
    pub is_system_sounds: bool,
    pub process_ids: Vec<u32>,
}

/// One owned sample containing a display-ready level for each application row.
#[derive(Clone, Debug)]
pub struct PeakSnapshot {
    pub levels: Vec<f32>,
}

/// Background Core Audio monitor. Dropping it guarantees sampling has stopped.
pub struct PeakMonitor {
    receiver: mpsc::Receiver<PeakSnapshot>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl PeakMonitor {
    pub fn start(window: HWND, applications: Vec<MonitoredApplication>) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let window_address = window.0 as isize;
        let worker = thread::Builder::new()
            .name("adufa-sound-locator".to_owned())
            .spawn(move || {
                let window = HWND(window_address as *mut core::ffi::c_void);
                if let Err(error) = monitor(window, applications, sender, &worker_stop) {
                    eprintln!("Sound Locator stopped: {error}");
                }
            })
            .map_err(|error| format!("Could not start Sound Locator: {error}"))?;

        Ok(Self {
            receiver,
            stop,
            worker: Some(worker),
        })
    }

    /// Returns only the newest queued sample; stale frames are intentionally
    /// discarded when the UI thread was busy routing or painting.
    pub fn take_latest(&self) -> Option<PeakSnapshot> {
        let mut latest = None;
        while let Ok(snapshot) = self.receiver.try_recv() {
            latest = Some(snapshot);
        }
        latest
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for PeakMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Pure presentation state for thresholding, source stability, and silence hold.
pub struct LocatorState {
    levels: Vec<f32>,
    strongest: Option<usize>,
    candidate: Option<(usize, Instant)>,
    last_audible: Option<Instant>,
    has_audible_source: bool,
}

impl LocatorState {
    pub fn new(application_count: usize) -> Self {
        Self {
            levels: vec![0.0; application_count],
            strongest: None,
            candidate: None,
            last_audible: None,
            has_audible_source: false,
        }
    }

    pub fn levels(&self) -> &[f32] {
        &self.levels
    }

    pub fn strongest(&self) -> Option<usize> {
        self.strongest
    }

    pub fn is_listening(&self) -> bool {
        !self.has_audible_source
    }

    /// Applies one raw linear-amplitude sample and returns rows whose visible
    /// level changed enough to require repainting.
    pub fn update(&mut self, raw_levels: &[f32], now: Instant) -> Vec<usize> {
        let mut changed = Vec::new();
        for index in 0..self.levels.len() {
            let raw = raw_levels.get(index).copied().unwrap_or_default();
            let visible = display_level(raw);
            if (self.levels[index] - visible).abs() >= 0.01 {
                self.levels[index] = visible;
                changed.push(index);
            }
        }

        let loudest = raw_levels
            .iter()
            .take(self.levels.len())
            .enumerate()
            .filter(|(_, level)| **level >= SILENCE_THRESHOLD)
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index);
        self.has_audible_source = loudest.is_some();

        match (self.strongest, loudest) {
            (None, Some(index)) => self.select(index, now),
            (Some(current), Some(index)) if current == index => {
                self.candidate = None;
                self.last_audible = Some(now);
            }
            (Some(_), Some(index)) => match self.candidate {
                Some((candidate, since))
                    if candidate == index && now.duration_since(since) >= SOURCE_SWITCH_DELAY =>
                {
                    self.select(index, now);
                }
                Some((candidate, _)) if candidate == index => {}
                _ => self.candidate = Some((index, now)),
            },
            (Some(_), None) => {
                self.candidate = None;
                if self
                    .last_audible
                    .is_some_and(|last| now.duration_since(last) >= SILENCE_HOLD)
                {
                    self.strongest = None;
                }
            }
            (None, None) => self.candidate = None,
        }

        changed
    }

    fn select(&mut self, index: usize, now: Instant) {
        self.strongest = Some(index);
        self.candidate = None;
        self.last_audible = Some(now);
    }
}

fn display_level(linear: f32) -> f32 {
    if linear < SILENCE_THRESHOLD {
        return 0.0;
    }
    let decibels = 20.0 * linear.clamp(f32::MIN_POSITIVE, 1.0).log10();
    ((decibels + 50.0) / 50.0).clamp(0.0, 1.0)
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> windows::core::Result<Self> {
        // SAFETY: The worker owns one balanced MTA initialization for its entire life.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: Paired with the successful initialization on this same worker.
        unsafe { CoUninitialize() };
    }
}

#[implement(IAudioSessionNotification)]
struct SessionNotification {
    sessions_changed: Arc<AtomicBool>,
}

impl IAudioSessionNotification_Impl for SessionNotification_Impl {
    fn OnSessionCreated(
        &self,
        _new_session: Ref<IAudioSessionControl>,
    ) -> windows::core::Result<()> {
        self.sessions_changed.store(true, Ordering::Release);
        Ok(())
    }
}

struct SessionMeter {
    application_index: usize,
    meter: IAudioMeterInformation,
}

fn monitor(
    window: HWND,
    applications: Vec<MonitoredApplication>,
    sender: mpsc::Sender<PeakSnapshot>,
    stop: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _apartment = ComApartment::initialize()?;
    // SAFETY: COM is initialized on this worker and aggregation is not used.
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    // SAFETY: The enumerator is live and the returned collection is COM-owned.
    let devices = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
    // SAFETY: The collection remains live through enumeration.
    let device_count = unsafe { devices.GetCount()? };
    let mut managers = Vec::new();
    for index in 0..device_count {
        // A disappearing endpoint is isolated from the remaining endpoints.
        let Ok(device) = (unsafe { devices.Item(index) }) else {
            continue;
        };
        let Ok(manager) = (unsafe { device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) })
        else {
            continue;
        };
        managers.push(manager);
    }

    let process_map = process_index(&applications);
    // Priming each session enumerator before registration follows the Core
    // Audio notification contract and also gives the first frame immediately.
    let mut meters = collect_meters(&managers, &applications, &process_map);
    let sessions_changed = Arc::new(AtomicBool::new(false));
    let notification: IAudioSessionNotification = SessionNotification {
        sessions_changed: Arc::clone(&sessions_changed),
    }
    .into();
    for manager in &managers {
        // SAFETY: Both interfaces remain alive until they are explicitly unregistered below.
        let _ = unsafe { manager.RegisterSessionNotification(&notification) };
    }

    while !stop.load(Ordering::Acquire) {
        if sessions_changed.swap(false, Ordering::AcqRel) {
            meters = collect_meters(&managers, &applications, &process_map);
        }

        let mut levels = vec![0.0_f32; applications.len()];
        meters.retain(|session| {
            // SAFETY: The meter is owned by this worker. A failed read usually
            // means the session ended, so it is removed without affecting peers.
            match unsafe { session.meter.GetPeakValue() } {
                Ok(level) => {
                    levels[session.application_index] =
                        levels[session.application_index].max(level.clamp(0.0, 1.0));
                    true
                }
                Err(_) => false,
            }
        });
        if sender.send(PeakSnapshot { levels }).is_err() {
            break;
        }
        // SAFETY: Posting an integer-only private message retains no Rust references.
        let _ = unsafe { PostMessageW(Some(window), UPDATE_MESSAGE, WPARAM(0), LPARAM(0)) };
        thread::sleep(SAMPLE_INTERVAL);
    }

    for manager in &managers {
        // SAFETY: This exactly reverses the successful-or-idempotent registrations.
        let _ = unsafe { manager.UnregisterSessionNotification(&notification) };
    }
    Ok(())
}

fn process_index(applications: &[MonitoredApplication]) -> BTreeMap<u32, usize> {
    applications
        .iter()
        .enumerate()
        .flat_map(|(index, application)| {
            application
                .process_ids
                .iter()
                .copied()
                .map(move |process_id| (process_id, index))
        })
        .collect()
}

fn collect_meters(
    managers: &[IAudioSessionManager2],
    applications: &[MonitoredApplication],
    process_map: &BTreeMap<u32, usize>,
) -> Vec<SessionMeter> {
    let system_sounds_index = applications
        .iter()
        .position(|application| application.is_system_sounds);
    let mut meters = Vec::new();
    for manager in managers {
        let Ok(sessions) = (unsafe { manager.GetSessionEnumerator() }) else {
            continue;
        };
        let Ok(count) = (unsafe { sessions.GetCount() }) else {
            continue;
        };
        for index in 0..count {
            let Ok(control) = (unsafe { sessions.GetSession(index) }) else {
                continue;
            };
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let application_index = if unsafe { control2.IsSystemSoundsSession() } == S_OK {
                system_sounds_index
            } else {
                unsafe { control2.GetProcessId() }
                    .ok()
                    .and_then(|process_id| process_map.get(&process_id).copied())
            };
            let (Some(application_index), Ok(meter)) =
                (application_index, control.cast::<IAudioMeterInformation>())
            else {
                continue;
            };
            meters.push(SessionMeter {
                application_index,
                meter,
            });
        }
    }
    meters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_threshold_noise_is_hidden() {
        assert_eq!(display_level(SILENCE_THRESHOLD / 2.0), 0.0);
        assert!(display_level(SILENCE_THRESHOLD) <= 0.001);
        assert_eq!(display_level(1.0), 1.0);
    }

    #[test]
    fn strongest_source_requires_stable_challenger() {
        let start = Instant::now();
        let mut state = LocatorState::new(2);
        state.update(&[0.2, 0.1], start);
        assert_eq!(state.strongest(), Some(0));

        state.update(&[0.1, 0.4], start + Duration::from_millis(1));
        state.update(&[0.1, 0.4], start + Duration::from_millis(300));
        assert_eq!(state.strongest(), Some(0));
        state.update(&[0.1, 0.4], start + Duration::from_millis(301));
        assert_eq!(state.strongest(), Some(1));
    }

    #[test]
    fn last_source_survives_one_second_of_silence() {
        let start = Instant::now();
        let mut state = LocatorState::new(1);
        state.update(&[0.4], start);
        state.update(&[0.0], start + Duration::from_millis(999));
        assert_eq!(state.strongest(), Some(0));
        assert!(state.is_listening());

        state.update(&[0.0], start + Duration::from_secs(1));
        assert_eq!(state.strongest(), None);
    }

    #[test]
    fn application_uses_maximum_session_level_not_sum() {
        let applications = [MonitoredApplication {
            is_system_sounds: false,
            process_ids: vec![10, 20],
        }];
        assert_eq!(process_index(&applications).get(&10), Some(&0));
        assert_eq!(process_index(&applications).get(&20), Some(&0));
    }
}
