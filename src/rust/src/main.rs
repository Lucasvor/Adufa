#![cfg_attr(not(windows), allow(dead_code))]

#[cfg(not(windows))]
compile_error!("This Core Audio probe only supports Windows.");

use std::env;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

type HResult = i32;
type ComPtr = *mut c_void;

const CLSCTX_ALL: u32 = 23;
const DEVICE_STATE_ACTIVE: u32 = 1;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

const CLSID_MM_DEVICE_ENUMERATOR: Guid = Guid::new(
    0xbcde0395,
    0xe52f,
    0x467c,
    [0x8e, 0x3d, 0xc4, 0x57, 0x92, 0x91, 0x69, 0x2e],
);
const IID_IMM_DEVICE_ENUMERATOR: Guid = Guid::new(
    0xa95664d2,
    0x9614,
    0x4f35,
    [0xa7, 0x46, 0xde, 0x8d, 0xb6, 0x36, 0x17, 0xe6],
);
const IID_AUDIO_SESSION_MANAGER_2: Guid = Guid::new(
    0x77aa99a0,
    0x1bd6,
    0x484f,
    [0x8b, 0xc7, 0x2c, 0x65, 0x4c, 0x9a, 0x9b, 0x6f],
);
const IID_AUDIO_SESSION_CONTROL_2: Guid = Guid::new(
    0xbfb7ff88,
    0x7239,
    0x4fc9,
    [0x8f, 0xa2, 0x07, 0xc9, 0x50, 0xbe, 0x9c, 0x6d],
);
const IID_SIMPLE_AUDIO_VOLUME: Guid = Guid::new(
    0x87ce5498,
    0x68d6,
    0x44e5,
    [0x92, 0x15, 0x6d, 0xa4, 0x7e, 0xf8, 0x83, 0xd8],
);
const IID_MM_NOTIFICATION_CLIENT: Guid = Guid::new(
    0x7991eec9,
    0x7e89,
    0x4d85,
    [0x83, 0x90, 0x6c, 0x70, 0x3c, 0xec, 0x60, 0xc0],
);
const IID_AUDIO_SESSION_NOTIFICATION: Guid = Guid::new(
    0x641dd20b,
    0x4d41,
    0x49cc,
    [0xab, 0xa3, 0x17, 0x4b, 0x94, 0x77, 0xbb, 0x08],
);
const IID_IUNKNOWN: Guid = Guid::new(0, 0, 0, [0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
const PKEY_DEVICE_FRIENDLY_NAME: PropertyKey = PropertyKey {
    format_id: Guid::new(
        0xa45c254e,
        0xdf1c,
        0x4efd,
        [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
    ),
    property_id: 14,
};

impl Guid {
    const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        Self {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PropertyKey {
    format_id: Guid,
    property_id: u32,
}

#[repr(C)]
struct PropVariant {
    value_type: u16,
    reserved1: u16,
    reserved2: u16,
    reserved3: u16,
    pointer: *mut u16,
    extra: usize,
}

impl Default for PropVariant {
    fn default() -> Self {
        Self {
            value_type: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            pointer: null_mut(),
            extra: 0,
        }
    }
}

struct Device {
    id: String,
    name: String,
    is_default: bool,
    sessions: Vec<Session>,
}

struct Session {
    pid: u32,
    state: &'static str,
    process: String,
    volume_percent: f32,
    muted: bool,
}

struct Options {
    hold: u64,
    pid: Option<u32>,
    volume: Option<f32>,
    muted: Option<bool>,
}

struct Snapshot {
    devices: Vec<Device>,
    managers: Vec<ComPtr>,
    updated: usize,
}

#[repr(C)]
struct CallbackObject {
    vtable: *const c_void,
    references: AtomicU32,
}

#[repr(C)]
struct DeviceCallbackVTable {
    query_interface: extern "system" fn(ComPtr, *const Guid, *mut ComPtr) -> HResult,
    add_ref: extern "system" fn(ComPtr) -> u32,
    release: extern "system" fn(ComPtr) -> u32,
    state_changed: extern "system" fn(ComPtr, *const u16, u32) -> HResult,
    added: extern "system" fn(ComPtr, *const u16) -> HResult,
    removed: extern "system" fn(ComPtr, *const u16) -> HResult,
    default_changed: extern "system" fn(ComPtr, i32, i32, *const u16) -> HResult,
    property_changed: extern "system" fn(ComPtr, *const u16, PropertyKey) -> HResult,
}

#[repr(C)]
struct SessionCallbackVTable {
    query_interface: extern "system" fn(ComPtr, *const Guid, *mut ComPtr) -> HResult,
    add_ref: extern "system" fn(ComPtr) -> u32,
    release: extern "system" fn(ComPtr) -> u32,
    session_created: extern "system" fn(ComPtr, ComPtr) -> HResult,
}

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, flags: u32) -> HResult;
    fn CoUninitialize();
    fn CoCreateInstance(
        clsid: *const Guid,
        outer: ComPtr,
        context: u32,
        iid: *const Guid,
        result: *mut ComPtr,
    ) -> HResult;
    fn CoTaskMemFree(memory: *mut c_void);
    fn PropVariantClear(value: *mut PropVariant) -> HResult;
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
    fn QueryFullProcessImageNameW(
        process: *mut c_void,
        flags: u32,
        path: *mut u16,
        size: *mut u32,
    ) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

fn main() {
    let options = parse_options().unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2);
    });
    let total = Instant::now();

    unsafe {
        let hr = CoInitializeEx(null_mut(), 0);
        let uninitialize = hr >= 0;
        if hr < 0 && hr != 0x80010106u32 as i32 {
            fail("CoInitializeEx", hr);
        }

        let mut enumerator = null_mut();
        check(
            CoCreateInstance(
                &CLSID_MM_DEVICE_ENUMERATOR,
                null_mut(),
                CLSCTX_ALL,
                &IID_IMM_DEVICE_ENUMERATOR,
                &mut enumerator,
            ),
            "CoCreateInstance(MMDeviceEnumerator)",
        );

        let device_callback = if options.hold > 0 {
            let callback = create_device_callback();
            let register: extern "system" fn(ComPtr, ComPtr) -> HResult = slot(enumerator, 6);
            check(
                register(enumerator, callback),
                "RegisterEndpointNotificationCallback",
            );
            callback
        } else {
            null_mut()
        };
        let session_callback = if options.hold > 0 {
            create_session_callback()
        } else {
            null_mut()
        };

        let scan = Instant::now();
        let snapshot = read_snapshot(enumerator, &options, session_callback);
        let scan_ms = scan.elapsed().as_secs_f64() * 1000.0;

        let session_count: usize = snapshot
            .devices
            .iter()
            .map(|device| device.sessions.len())
            .sum();
        for device in &snapshot.devices {
            println!(
                "DEVICE default={} name={} id={}",
                device.is_default as u8,
                escape(&device.name),
                escape(&device.id)
            );
            for session in &device.sessions {
                println!(
                    "SESSION pid={} state={} volume={:.1} muted={} process={} device={}",
                    session.pid,
                    session.state,
                    session.volume_percent,
                    session.muted as u8,
                    escape(&session.process),
                    escape(&device.name)
                );
            }
        }
        if let Some(pid) = options.pid {
            println!("UPDATED pid={} sessions={}", pid, snapshot.updated);
        }
        println!(
            "READY language=rust devices={} sessions={} scan_ms={:.3} startup_ms={:.3}",
            snapshot.devices.len(),
            session_count,
            scan_ms,
            total.elapsed().as_secs_f64() * 1000.0
        );

        if options.hold > 0 {
            thread::sleep(Duration::from_secs(options.hold));
        }

        if !session_callback.is_null() {
            for manager in &snapshot.managers {
                let unregister: extern "system" fn(ComPtr, ComPtr) -> HResult = slot(*manager, 7);
                unregister(*manager, session_callback);
                release(*manager);
            }
            drop(Box::from_raw(session_callback as *mut CallbackObject));
        }
        if !device_callback.is_null() {
            let unregister: extern "system" fn(ComPtr, ComPtr) -> HResult = slot(enumerator, 7);
            unregister(enumerator, device_callback);
            drop(Box::from_raw(device_callback as *mut CallbackObject));
        }
        release(enumerator);
        if uninitialize {
            CoUninitialize();
        }
    }
}

unsafe fn read_snapshot(
    enumerator: ComPtr,
    options: &Options,
    session_callback: ComPtr,
) -> Snapshot {
    let default_id = default_device_id(enumerator);
    let mut collection = null_mut();
    let enumerate: extern "system" fn(ComPtr, i32, u32, *mut ComPtr) -> HResult =
        slot(enumerator, 3);
    check(
        enumerate(enumerator, 0, DEVICE_STATE_ACTIVE, &mut collection),
        "EnumAudioEndpoints",
    );

    let get_count: extern "system" fn(ComPtr, *mut u32) -> HResult = slot(collection, 3);
    let item: extern "system" fn(ComPtr, u32, *mut ComPtr) -> HResult = slot(collection, 4);
    let mut count = 0;
    check(
        get_count(collection, &mut count),
        "IMMDeviceCollection::GetCount",
    );
    let mut snapshot = Snapshot {
        devices: Vec::with_capacity(count as usize),
        managers: Vec::new(),
        updated: 0,
    };

    for index in 0..count {
        let mut raw_device = null_mut();
        check(
            item(collection, index, &mut raw_device),
            "IMMDeviceCollection::Item",
        );
        let id = device_id(raw_device);
        let mut device = Device {
            is_default: id == default_id,
            name: device_name(raw_device),
            id,
            sessions: Vec::new(),
        };
        read_sessions(
            raw_device,
            &mut device.sessions,
            &mut snapshot,
            options,
            session_callback,
        );
        release(raw_device);
        snapshot.devices.push(device);
    }

    release(collection);
    snapshot
}

unsafe fn default_device_id(enumerator: ComPtr) -> String {
    let get_default: extern "system" fn(ComPtr, i32, i32, *mut ComPtr) -> HResult =
        slot(enumerator, 4);
    let mut device = null_mut();
    if get_default(enumerator, 0, 1, &mut device) < 0 {
        return String::new();
    }
    let id = device_id(device);
    release(device);
    id
}

unsafe fn device_id(device: ComPtr) -> String {
    let get_id: extern "system" fn(ComPtr, *mut *mut u16) -> HResult = slot(device, 5);
    let mut value = null_mut();
    check(get_id(device, &mut value), "IMMDevice::GetId");
    let result = wide_string(value);
    CoTaskMemFree(value.cast());
    result
}

unsafe fn device_name(device: ComPtr) -> String {
    let open: extern "system" fn(ComPtr, u32, *mut ComPtr) -> HResult = slot(device, 4);
    let mut store = null_mut();
    if open(device, 0, &mut store) < 0 {
        return "(unknown)".into();
    }
    let get_value: extern "system" fn(ComPtr, *const PropertyKey, *mut PropVariant) -> HResult =
        slot(store, 5);
    let mut value = PropVariant::default();
    let name = if get_value(store, &PKEY_DEVICE_FRIENDLY_NAME, &mut value) >= 0
        && value.value_type == 31
        && !value.pointer.is_null()
    {
        wide_string(value.pointer)
    } else {
        "(unknown)".into()
    };
    PropVariantClear(&mut value);
    release(store);
    name
}

unsafe fn read_sessions(
    device: ComPtr,
    sessions: &mut Vec<Session>,
    snapshot: &mut Snapshot,
    options: &Options,
    session_callback: ComPtr,
) {
    let activate: extern "system" fn(ComPtr, *const Guid, u32, ComPtr, *mut ComPtr) -> HResult =
        slot(device, 3);
    let mut manager = null_mut();
    if activate(
        device,
        &IID_AUDIO_SESSION_MANAGER_2,
        CLSCTX_ALL,
        null_mut(),
        &mut manager,
    ) < 0
    {
        return;
    }
    let get_enumerator: extern "system" fn(ComPtr, *mut ComPtr) -> HResult = slot(manager, 5);
    let mut session_enumerator = null_mut();
    if get_enumerator(manager, &mut session_enumerator) < 0 {
        release(manager);
        return;
    }
    let get_count: extern "system" fn(ComPtr, *mut i32) -> HResult = slot(session_enumerator, 3);
    let get_session: extern "system" fn(ComPtr, i32, *mut ComPtr) -> HResult =
        slot(session_enumerator, 4);
    let mut count = 0;
    if get_count(session_enumerator, &mut count) >= 0 {
        for index in 0..count {
            let mut control = null_mut();
            if get_session(session_enumerator, index, &mut control) < 0 {
                continue;
            }
            let control2 = query_interface(control, &IID_AUDIO_SESSION_CONTROL_2);
            if !control2.is_null() {
                let get_state: extern "system" fn(ComPtr, *mut i32) -> HResult = slot(control2, 3);
                let get_pid: extern "system" fn(ComPtr, *mut u32) -> HResult = slot(control2, 14);
                let mut state = 0;
                let mut pid = 0;
                if get_pid(control2, &mut pid) >= 0 && get_state(control2, &mut state) >= 0 {
                    let volume = query_interface(control, &IID_SIMPLE_AUDIO_VOLUME);
                    let mut level = 0.0f32;
                    let mut muted = 0i32;
                    if !volume.is_null() {
                        let set_level: extern "system" fn(ComPtr, f32, *const Guid) -> HResult =
                            slot(volume, 3);
                        let get_level: extern "system" fn(ComPtr, *mut f32) -> HResult =
                            slot(volume, 4);
                        let set_mute: extern "system" fn(ComPtr, i32, *const Guid) -> HResult =
                            slot(volume, 5);
                        let get_mute: extern "system" fn(ComPtr, *mut i32) -> HResult =
                            slot(volume, 6);
                        if options.pid == Some(pid) {
                            if let Some(new_level) = options.volume {
                                check(set_level(volume, new_level, null_mut()), "SetMasterVolume");
                            }
                            if let Some(new_muted) = options.muted {
                                check(set_mute(volume, new_muted as i32, null_mut()), "SetMute");
                            }
                            snapshot.updated += 1;
                        }
                        get_level(volume, &mut level);
                        get_mute(volume, &mut muted);
                        release(volume);
                    }
                    sessions.push(Session {
                        pid,
                        state: state_name(state),
                        process: process_name(pid),
                        volume_percent: level * 100.0,
                        muted: muted != 0,
                    });
                }
                release(control2);
            }
            release(control);
        }
    }
    release(session_enumerator);
    if !session_callback.is_null() {
        let register: extern "system" fn(ComPtr, ComPtr) -> HResult = slot(manager, 6);
        if register(manager, session_callback) >= 0 {
            snapshot.managers.push(manager);
            return;
        }
    }
    release(manager);
}

unsafe fn process_name(pid: u32) -> String {
    if pid == 0 {
        return "system-sounds".into();
    }
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process.is_null() {
        return "(unavailable)".into();
    }
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
    CloseHandle(process);
    if ok == 0 {
        return "(unavailable)".into();
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    path.rsplit(['\\', '/']).next().unwrap_or(&path).to_owned()
}

unsafe fn query_interface(instance: ComPtr, iid: *const Guid) -> ComPtr {
    let query: extern "system" fn(ComPtr, *const Guid, *mut ComPtr) -> HResult = slot(instance, 0);
    let mut result = null_mut();
    if query(instance, iid, &mut result) >= 0 {
        result
    } else {
        null_mut()
    }
}

unsafe fn release(instance: ComPtr) {
    if !instance.is_null() {
        let release_fn: extern "system" fn(ComPtr) -> u32 = slot(instance, 2);
        release_fn(instance);
    }
}

unsafe fn slot<T: Copy>(instance: ComPtr, index: usize) -> T {
    let table = *(instance as *const *const usize);
    std::mem::transmute_copy(&*table.add(index))
}

unsafe fn wide_string(value: *const u16) -> String {
    if value.is_null() {
        return String::new();
    }
    let mut length = 0;
    while *value.add(length) != 0 {
        length += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(value, length))
}

static DEVICE_CALLBACK_VTABLE: DeviceCallbackVTable = DeviceCallbackVTable {
    query_interface: device_callback_query_interface,
    add_ref: callback_add_ref,
    release: callback_release,
    state_changed: on_device_state_changed,
    added: on_device_added,
    removed: on_device_removed,
    default_changed: on_default_device_changed,
    property_changed: on_device_property_changed,
};

static SESSION_CALLBACK_VTABLE: SessionCallbackVTable = SessionCallbackVTable {
    query_interface: session_callback_query_interface,
    add_ref: callback_add_ref,
    release: callback_release,
    session_created: on_session_created,
};

fn create_device_callback() -> ComPtr {
    Box::into_raw(Box::new(CallbackObject {
        vtable: (&DEVICE_CALLBACK_VTABLE as *const DeviceCallbackVTable).cast(),
        references: AtomicU32::new(1),
    }))
    .cast()
}

fn create_session_callback() -> ComPtr {
    Box::into_raw(Box::new(CallbackObject {
        vtable: (&SESSION_CALLBACK_VTABLE as *const SessionCallbackVTable).cast(),
        references: AtomicU32::new(1),
    }))
    .cast()
}

extern "system" fn device_callback_query_interface(
    self_pointer: ComPtr,
    iid: *const Guid,
    result: *mut ComPtr,
) -> HResult {
    query_callback(self_pointer, iid, result, &IID_MM_NOTIFICATION_CLIENT)
}

extern "system" fn session_callback_query_interface(
    self_pointer: ComPtr,
    iid: *const Guid,
    result: *mut ComPtr,
) -> HResult {
    query_callback(self_pointer, iid, result, &IID_AUDIO_SESSION_NOTIFICATION)
}

fn query_callback(
    self_pointer: ComPtr,
    iid: *const Guid,
    result: *mut ComPtr,
    supported: &Guid,
) -> HResult {
    unsafe {
        if *iid == IID_IUNKNOWN || *iid == *supported {
            *result = self_pointer;
            callback_add_ref(self_pointer);
            0
        } else {
            *result = null_mut();
            0x80004002u32 as i32
        }
    }
}

extern "system" fn callback_add_ref(self_pointer: ComPtr) -> u32 {
    unsafe {
        (*(self_pointer as *mut CallbackObject))
            .references
            .fetch_add(1, Ordering::Relaxed)
            + 1
    }
}

extern "system" fn callback_release(self_pointer: ComPtr) -> u32 {
    unsafe {
        (*(self_pointer as *mut CallbackObject))
            .references
            .fetch_sub(1, Ordering::Release)
            - 1
    }
}

extern "system" fn on_device_state_changed(_: ComPtr, id: *const u16, state: u32) -> HResult {
    unsafe {
        println!(
            "EVENT type=device-state state={} id={}",
            state,
            escape(&wide_string(id))
        )
    };
    0
}

extern "system" fn on_device_added(_: ComPtr, id: *const u16) -> HResult {
    unsafe { println!("EVENT type=device-added id={}", escape(&wide_string(id))) };
    0
}

extern "system" fn on_device_removed(_: ComPtr, id: *const u16) -> HResult {
    unsafe { println!("EVENT type=device-removed id={}", escape(&wide_string(id))) };
    0
}

extern "system" fn on_default_device_changed(
    _: ComPtr,
    flow: i32,
    role: i32,
    id: *const u16,
) -> HResult {
    unsafe {
        println!(
            "EVENT type=default-device flow={} role={} id={}",
            flow,
            role,
            escape(&wide_string(id))
        )
    };
    0
}

extern "system" fn on_device_property_changed(
    _: ComPtr,
    id: *const u16,
    _: PropertyKey,
) -> HResult {
    unsafe { println!("EVENT type=device-property id={}", escape(&wide_string(id))) };
    0
}

extern "system" fn on_session_created(_: ComPtr, control: ComPtr) -> HResult {
    unsafe {
        let control2 = query_interface(control, &IID_AUDIO_SESSION_CONTROL_2);
        if !control2.is_null() {
            let get_pid: extern "system" fn(ComPtr, *mut u32) -> HResult = slot(control2, 14);
            let mut pid = 0;
            if get_pid(control2, &mut pid) >= 0 {
                println!(
                    "EVENT type=session-created pid={} process={}",
                    pid,
                    escape(&process_name(pid))
                );
            }
            release(control2);
        }
    }
    0
}

fn parse_options() -> Result<Options, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        hold: 0,
        pid: None,
        volume: None,
        muted: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--hold" if index + 1 < args.len() => {
                options.hold = args[index + 1]
                    .parse::<u64>()
                    .ok()
                    .filter(|seconds| *seconds <= 3600)
                    .ok_or_else(usage)?;
                index += 2;
            }
            "--set-volume" if index + 2 < args.len() => {
                let pid = args[index + 1].parse::<u32>().map_err(|_| usage())?;
                ensure_same_pid(&mut options.pid, pid)?;
                options.volume = Some(
                    args[index + 2]
                        .parse::<f32>()
                        .ok()
                        .filter(|percent| *percent >= 0.0 && *percent <= 100.0)
                        .ok_or_else(usage)?
                        / 100.0,
                );
                index += 3;
            }
            "--mute" if index + 2 < args.len() => {
                let pid = args[index + 1].parse::<u32>().map_err(|_| usage())?;
                ensure_same_pid(&mut options.pid, pid)?;
                options.muted = Some(match args[index + 2].as_str() {
                    "on" => true,
                    "off" => false,
                    _ => return Err(usage()),
                });
                index += 3;
            }
            _ => return Err(usage()),
        }
    }
    Ok(options)
}

fn ensure_same_pid(current: &mut Option<u32>, next: u32) -> Result<(), String> {
    if current.is_some_and(|pid| pid != next) {
        return Err(format!(
            "Volume and mute must target the same PID.\n{}",
            usage()
        ));
    }
    *current = Some(next);
    Ok(())
}

fn usage() -> String {
    "Usage: audiorouter-rust [--hold 0..3600] [--set-volume PID 0..100] [--mute PID on|off]".into()
}

fn state_name(state: i32) -> &'static str {
    match state {
        0 => "inactive",
        1 => "active",
        2 => "expired",
        _ => "unknown",
    }
}

fn escape(value: &str) -> String {
    value.replace([' ', '\r', '\n'], "_")
}

fn check(hr: HResult, operation: &str) {
    if hr < 0 {
        fail(operation, hr);
    }
}

fn fail(operation: &str, hr: HResult) -> ! {
    eprintln!("{operation} failed: 0x{:08X}", hr as u32);
    std::process::exit(1);
}
