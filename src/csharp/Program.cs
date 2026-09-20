using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

internal static unsafe class Program
{
    private const uint CLSCTX_ALL = 23;
    private const uint DEVICE_STATE_ACTIVE = 1;
    private const uint STGM_READ = 0;
    private const uint PROCESS_QUERY_LIMITED_INFORMATION = 0x1000;

    private static readonly Guid CLSID_MMDeviceEnumerator = new("BCDE0395-E52F-467C-8E3D-C4579291692E");
    private static readonly Guid IID_IMMDeviceEnumerator = new("A95664D2-9614-4F35-A746-DE8DB63617E6");
    private static readonly Guid IID_IAudioSessionManager2 = new("77AA99A0-1BD6-484F-8BC7-2C654C9A9B6F");
    private static readonly Guid IID_IAudioSessionControl2 = new("BFB7FF88-7239-4FC9-8FA2-07C950BE9C6D");
    private static readonly Guid IID_ISimpleAudioVolume = new("87CE5498-68D6-44E5-9215-6DA47EF883D8");
    private static readonly Guid IID_IMMNotificationClient = new("7991EEC9-7E89-4D85-8390-6C703CEC60C0");
    private static readonly Guid IID_IAudioSessionNotification = new("641DD20B-4D41-49CC-ABA3-174B9477BB08");
    private static readonly Guid IID_IUnknown = new("00000000-0000-0000-C000-000000000046");
    private static readonly nint DeviceCallbackVTable = CreateDeviceCallbackVTable();
    private static readonly nint SessionCallbackVTable = CreateSessionCallbackVTable();
    private static readonly PropertyKey PKEY_Device_FriendlyName = new(
        new Guid("A45C254E-DF1C-4EFD-8020-67D146A850E0"), 14);

    public static int Main(string[] args)
    {
        Console.OutputEncoding = Encoding.UTF8;
        Options options;
        try
        {
            options = ParseOptions(args);
        }
        catch (ArgumentException error)
        {
            Console.Error.WriteLine(error.Message);
            return 2;
        }

        Stopwatch total = Stopwatch.StartNew();
        int hr = CoInitializeEx(0, 0);
        bool uninitialize = hr >= 0;
        if (hr < 0 && hr != unchecked((int)0x80010106))
            return Fail("CoInitializeEx", hr);

        nint enumerator = 0;
        nint deviceCallback = 0;
        nint sessionCallback = 0;
        Snapshot? snapshot = null;
        try
        {
            Guid clsid = CLSID_MMDeviceEnumerator;
            Guid iid = IID_IMMDeviceEnumerator;
            hr = CoCreateInstance(&clsid, 0, CLSCTX_ALL, &iid, &enumerator);
            if (hr < 0)
                return Fail("CoCreateInstance(MMDeviceEnumerator)", hr);

            if (options.HoldSeconds > 0)
            {
                deviceCallback = CreateCallback(DeviceCallbackVTable);
                sessionCallback = CreateCallback(SessionCallbackVTable);
                ThrowIfFailed(RegisterEndpointNotificationCallback(enumerator, deviceCallback));
            }

            Stopwatch scan = Stopwatch.StartNew();
            snapshot = ReadSnapshot(enumerator, options, sessionCallback);
            scan.Stop();

            foreach (Device device in snapshot.Devices)
            {
                Console.WriteLine($"DEVICE default={(device.IsDefault ? 1 : 0)} name={Escape(device.Name)} id={Escape(device.Id)}");
                foreach (Session session in device.Sessions)
                    Console.WriteLine($"SESSION pid={session.ProcessId} state={session.State} volume={session.VolumePercent:F1} muted={(session.Muted ? 1 : 0)} process={Escape(session.ProcessName)} device={Escape(device.Name)}");
            }

            if (options.TargetPid is not null)
                Console.WriteLine($"UPDATED pid={options.TargetPid} sessions={snapshot.UpdatedCount}");

            total.Stop();
            Console.WriteLine($"READY language=csharp devices={snapshot.Devices.Count} sessions={snapshot.SessionCount} scan_ms={scan.Elapsed.TotalMilliseconds:F3} startup_ms={total.Elapsed.TotalMilliseconds:F3}");
            if (options.HoldSeconds > 0)
                Thread.Sleep(TimeSpan.FromSeconds(options.HoldSeconds));
            return 0;
        }
        catch (COMException error)
        {
            Console.Error.WriteLine($"Core Audio failed: 0x{error.HResult:X8} {error.Message}");
            return 1;
        }
        finally
        {
            if (snapshot is not null)
            {
                foreach (nint manager in snapshot.SessionManagers)
                {
                    if (sessionCallback != 0)
                        UnregisterSessionNotification(manager, sessionCallback);
                    Release(manager);
                }
            }
            if (deviceCallback != 0 && enumerator != 0)
                UnregisterEndpointNotificationCallback(enumerator, deviceCallback);
            if (deviceCallback != 0)
                NativeMemory.Free((void*)deviceCallback);
            if (sessionCallback != 0)
                NativeMemory.Free((void*)sessionCallback);
            Release(enumerator);
            if (uninitialize)
                CoUninitialize();
        }
    }

    private static Snapshot ReadSnapshot(nint enumerator, Options options, nint sessionCallback)
    {
        string defaultId = ReadDefaultDeviceId(enumerator);
        nint collection = 0;
        int hr = EnumAudioEndpoints(enumerator, 0, DEVICE_STATE_ACTIVE, &collection);
        ThrowIfFailed(hr);

        try
        {
            uint count = 0;
            ThrowIfFailed(CollectionGetCount(collection, &count));
            var snapshot = new Snapshot();
            for (uint i = 0; i < count; i++)
            {
                nint device = 0;
                ThrowIfFailed(CollectionItem(collection, i, &device));
                try
                {
                    string id = DeviceId(device);
                    var item = new Device(id, DeviceName(device), id == defaultId);
                    ReadSessions(device, item.Sessions, snapshot, options, sessionCallback);
                    snapshot.Devices.Add(item);
                    snapshot.SessionCount += item.Sessions.Count;
                }
                finally
                {
                    Release(device);
                }
            }
            return snapshot;
        }
        finally
        {
            Release(collection);
        }
    }

    private static string ReadDefaultDeviceId(nint enumerator)
    {
        nint device = 0;
        int hr = GetDefaultAudioEndpoint(enumerator, 0, 1, &device);
        if (hr < 0)
            return string.Empty;
        try { return DeviceId(device); }
        finally { Release(device); }
    }

    private static string DeviceId(nint device)
    {
        nint value = 0;
        ThrowIfFailed(GetDeviceId(device, &value));
        try { return Marshal.PtrToStringUni(value) ?? string.Empty; }
        finally { CoTaskMemFree(value); }
    }

    private static string DeviceName(nint device)
    {
        nint store = 0;
        if (OpenPropertyStore(device, STGM_READ, &store) < 0)
            return "(unknown)";
        try
        {
            PropertyKey key = PKEY_Device_FriendlyName;
            PropVariant value = default;
            int hr = PropertyGetValue(store, &key, &value);
            if (hr < 0 || value.Type != 31 || value.Pointer == 0)
                return "(unknown)";
            try { return Marshal.PtrToStringUni(value.Pointer) ?? "(unknown)"; }
            finally { PropVariantClear(&value); }
        }
        finally
        {
            Release(store);
        }
    }

    private static void ReadSessions(nint device, List<Session> sessions, Snapshot snapshot, Options options, nint sessionCallback)
    {
        Guid iid = IID_IAudioSessionManager2;
        nint manager = 0;
        int hr = Activate(device, &iid, CLSCTX_ALL, 0, &manager);
        if (hr < 0)
            return;
        bool retainManager = false;
        try
        {
            nint sessionEnumerator = 0;
            if (SessionManagerGetEnumerator(manager, &sessionEnumerator) < 0)
                return;
            try
            {
                int count = 0;
                ThrowIfFailed(SessionEnumeratorGetCount(sessionEnumerator, &count));
                for (int i = 0; i < count; i++)
                {
                    nint control = 0;
                    if (SessionEnumeratorGetSession(sessionEnumerator, i, &control) < 0)
                        continue;
                    try
                    {
                        nint control2 = QueryInterface(control, IID_IAudioSessionControl2);
                        if (control2 == 0)
                            continue;
                        try
                        {
                            uint pid = 0;
                            int state = 0;
                            if (SessionGetProcessId(control2, &pid) >= 0 && SessionGetState(control2, &state) >= 0)
                            {
                                nint volume = QueryInterface(control, IID_ISimpleAudioVolume);
                                float level = 0;
                                int muted = 0;
                                if (volume != 0)
                                {
                                    try
                                    {
                                        if (options.TargetPid == pid)
                                        {
                                            if (options.Volume is float newLevel)
                                                ThrowIfFailed(SetMasterVolume(volume, newLevel, null));
                                            if (options.Muted is bool newMuted)
                                                ThrowIfFailed(SetMute(volume, newMuted ? 1 : 0, null));
                                            snapshot.UpdatedCount++;
                                        }
                                        GetMasterVolume(volume, &level);
                                        GetMute(volume, &muted);
                                    }
                                    finally { Release(volume); }
                                }
                                sessions.Add(new Session(pid, StateName(state), ProcessName(pid), level * 100, muted != 0));
                            }
                        }
                        finally { Release(control2); }
                    }
                    finally { Release(control); }
                }
            }
            finally { Release(sessionEnumerator); }

            if (sessionCallback != 0 && RegisterSessionNotification(manager, sessionCallback) >= 0)
            {
                snapshot.SessionManagers.Add(manager);
                retainManager = true;
            }
        }
        finally
        {
            if (!retainManager)
                Release(manager);
        }
    }

    private static string ProcessName(uint pid)
    {
        if (pid == 0)
            return "system-sounds";
        nint process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if (process == 0)
            return "(unavailable)";
        try
        {
            uint length = 32768;
            char* buffer = stackalloc char[(int)length];
            if (!QueryFullProcessImageNameW(process, 0, buffer, &length))
                return "(unavailable)";
            string path = new(buffer, 0, (int)length);
            int separator = Math.Max(path.LastIndexOf('\\'), path.LastIndexOf('/'));
            return separator >= 0 ? path[(separator + 1)..] : path;
        }
        finally { CloseHandle(process); }
    }

    private static Options ParseOptions(string[] args)
    {
        int hold = 0;
        uint? pid = null;
        float? volume = null;
        bool? muted = null;
        for (int i = 0; i < args.Length;)
        {
            if (i + 1 < args.Length && args[i] == "--hold" && int.TryParse(args[i + 1], out int seconds) && seconds is >= 0 and <= 3600)
            {
                hold = seconds;
                i += 2;
            }
            else if (i + 2 < args.Length && args[i] == "--set-volume" && uint.TryParse(args[i + 1], out uint volumePid) && float.TryParse(args[i + 2], out float percent) && percent is >= 0 and <= 100)
            {
                EnsureSamePid(ref pid, volumePid);
                volume = percent / 100;
                i += 3;
            }
            else if (i + 2 < args.Length && args[i] == "--mute" && uint.TryParse(args[i + 1], out uint mutePid) && ParseBoolean(args[i + 2], out bool mute))
            {
                EnsureSamePid(ref pid, mutePid);
                muted = mute;
                i += 3;
            }
            else
            {
                throw new ArgumentException(Usage);
            }
        }
        return new Options(hold, pid, volume, muted);
    }

    private static void EnsureSamePid(ref uint? current, uint next)
    {
        if (current is not null && current != next)
            throw new ArgumentException("Volume and mute must target the same PID.\n" + Usage);
        current = next;
    }

    private static bool ParseBoolean(string value, out bool result)
    {
        result = value == "on";
        return result || value == "off";
    }

    private const string Usage = "Usage: AudioProbeCSharp [--hold 0..3600] [--set-volume PID 0..100] [--mute PID on|off]";

    private static string StateName(int state) => state switch
    {
        0 => "inactive",
        1 => "active",
        2 => "expired",
        _ => "unknown"
    };

    private static string Escape(string value) => value.Replace(' ', '_').Replace('\r', '_').Replace('\n', '_');

    private static void ThrowIfFailed(int hr)
    {
        if (hr < 0)
            Marshal.ThrowExceptionForHR(hr);
    }

    private static int Fail(string operation, int hr)
    {
        Console.Error.WriteLine($"{operation} failed: 0x{hr:X8}");
        return 1;
    }

    private static nint QueryInterface(nint instance, Guid iid)
    {
        nint result = 0;
        Guid local = iid;
        var method = (delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)VTable(instance)[0];
        return method(instance, &local, &result) >= 0 ? result : 0;
    }

    private static void Release(nint instance)
    {
        if (instance != 0)
        {
            var method = (delegate* unmanaged[Stdcall]<nint, uint>)VTable(instance)[2];
            method(instance);
        }
    }

    private static nint* VTable(nint instance) => *(nint**)instance;
    private static int EnumAudioEndpoints(nint p, int flow, uint mask, nint* result) => ((delegate* unmanaged[Stdcall]<nint, int, uint, nint*, int>)VTable(p)[3])(p, flow, mask, result);
    private static int GetDefaultAudioEndpoint(nint p, int flow, int role, nint* result) => ((delegate* unmanaged[Stdcall]<nint, int, int, nint*, int>)VTable(p)[4])(p, flow, role, result);
    private static int CollectionGetCount(nint p, uint* count) => ((delegate* unmanaged[Stdcall]<nint, uint*, int>)VTable(p)[3])(p, count);
    private static int CollectionItem(nint p, uint index, nint* item) => ((delegate* unmanaged[Stdcall]<nint, uint, nint*, int>)VTable(p)[4])(p, index, item);
    private static int Activate(nint p, Guid* iid, uint context, nint activation, nint* result) => ((delegate* unmanaged[Stdcall]<nint, Guid*, uint, nint, nint*, int>)VTable(p)[3])(p, iid, context, activation, result);
    private static int OpenPropertyStore(nint p, uint mode, nint* result) => ((delegate* unmanaged[Stdcall]<nint, uint, nint*, int>)VTable(p)[4])(p, mode, result);
    private static int GetDeviceId(nint p, nint* id) => ((delegate* unmanaged[Stdcall]<nint, nint*, int>)VTable(p)[5])(p, id);
    private static int PropertyGetValue(nint p, PropertyKey* key, PropVariant* value) => ((delegate* unmanaged[Stdcall]<nint, PropertyKey*, PropVariant*, int>)VTable(p)[5])(p, key, value);
    private static int SessionManagerGetEnumerator(nint p, nint* result) => ((delegate* unmanaged[Stdcall]<nint, nint*, int>)VTable(p)[5])(p, result);
    private static int SessionEnumeratorGetCount(nint p, int* count) => ((delegate* unmanaged[Stdcall]<nint, int*, int>)VTable(p)[3])(p, count);
    private static int SessionEnumeratorGetSession(nint p, int index, nint* result) => ((delegate* unmanaged[Stdcall]<nint, int, nint*, int>)VTable(p)[4])(p, index, result);
    private static int SessionGetState(nint p, int* state) => ((delegate* unmanaged[Stdcall]<nint, int*, int>)VTable(p)[3])(p, state);
    private static int SessionGetProcessId(nint p, uint* pid) => ((delegate* unmanaged[Stdcall]<nint, uint*, int>)VTable(p)[14])(p, pid);
    private static int SetMasterVolume(nint p, float level, Guid* context) => ((delegate* unmanaged[Stdcall]<nint, float, Guid*, int>)VTable(p)[3])(p, level, context);
    private static int GetMasterVolume(nint p, float* level) => ((delegate* unmanaged[Stdcall]<nint, float*, int>)VTable(p)[4])(p, level);
    private static int SetMute(nint p, int muted, Guid* context) => ((delegate* unmanaged[Stdcall]<nint, int, Guid*, int>)VTable(p)[5])(p, muted, context);
    private static int GetMute(nint p, int* muted) => ((delegate* unmanaged[Stdcall]<nint, int*, int>)VTable(p)[6])(p, muted);
    private static int RegisterEndpointNotificationCallback(nint p, nint callback) => ((delegate* unmanaged[Stdcall]<nint, nint, int>)VTable(p)[6])(p, callback);
    private static int UnregisterEndpointNotificationCallback(nint p, nint callback) => ((delegate* unmanaged[Stdcall]<nint, nint, int>)VTable(p)[7])(p, callback);
    private static int RegisterSessionNotification(nint p, nint callback) => ((delegate* unmanaged[Stdcall]<nint, nint, int>)VTable(p)[6])(p, callback);
    private static int UnregisterSessionNotification(nint p, nint callback) => ((delegate* unmanaged[Stdcall]<nint, nint, int>)VTable(p)[7])(p, callback);

    private static nint CreateCallback(nint vtable)
    {
        CallbackObject* callback = (CallbackObject*)NativeMemory.Alloc((nuint)sizeof(CallbackObject));
        callback->VTable = vtable;
        callback->References = 1;
        return (nint)callback;
    }

    private static nint CreateDeviceCallbackVTable()
    {
        nint* table = (nint*)NativeMemory.Alloc(8, (nuint)sizeof(nint));
        table[0] = (nint)(delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)&CallbackQueryInterface;
        table[1] = (nint)(delegate* unmanaged[Stdcall]<nint, uint>)&CallbackAddRef;
        table[2] = (nint)(delegate* unmanaged[Stdcall]<nint, uint>)&CallbackRelease;
        table[3] = (nint)(delegate* unmanaged[Stdcall]<nint, char*, uint, int>)&OnDeviceStateChanged;
        table[4] = (nint)(delegate* unmanaged[Stdcall]<nint, char*, int>)&OnDeviceAdded;
        table[5] = (nint)(delegate* unmanaged[Stdcall]<nint, char*, int>)&OnDeviceRemoved;
        table[6] = (nint)(delegate* unmanaged[Stdcall]<nint, int, int, char*, int>)&OnDefaultDeviceChanged;
        table[7] = (nint)(delegate* unmanaged[Stdcall]<nint, char*, PropertyKey, int>)&OnDevicePropertyChanged;
        return (nint)table;
    }

    private static nint CreateSessionCallbackVTable()
    {
        nint* table = (nint*)NativeMemory.Alloc(4, (nuint)sizeof(nint));
        table[0] = (nint)(delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)&SessionCallbackQueryInterface;
        table[1] = (nint)(delegate* unmanaged[Stdcall]<nint, uint>)&CallbackAddRef;
        table[2] = (nint)(delegate* unmanaged[Stdcall]<nint, uint>)&CallbackRelease;
        table[3] = (nint)(delegate* unmanaged[Stdcall]<nint, nint, int>)&OnSessionCreated;
        return (nint)table;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int CallbackQueryInterface(nint self, Guid* iid, nint* result) => QueryCallback(self, iid, result, IID_IMMNotificationClient);

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int SessionCallbackQueryInterface(nint self, Guid* iid, nint* result) => QueryCallback(self, iid, result, IID_IAudioSessionNotification);

    private static int QueryCallback(nint self, Guid* iid, nint* result, Guid supported)
    {
        if (*iid == IID_IUnknown || *iid == supported)
        {
            *result = self;
            Interlocked.Increment(ref ((CallbackObject*)self)->References);
            return 0;
        }
        *result = 0;
        return unchecked((int)0x80004002);
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static uint CallbackAddRef(nint self) => (uint)Interlocked.Increment(ref ((CallbackObject*)self)->References);

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static uint CallbackRelease(nint self) => (uint)Interlocked.Decrement(ref ((CallbackObject*)self)->References);

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnDeviceStateChanged(nint _, char* id, uint state)
    {
        Console.WriteLine($"EVENT type=device-state state={state} id={Escape(new string(id))}");
        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnDeviceAdded(nint _, char* id)
    {
        Console.WriteLine($"EVENT type=device-added id={Escape(new string(id))}");
        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnDeviceRemoved(nint _, char* id)
    {
        Console.WriteLine($"EVENT type=device-removed id={Escape(new string(id))}");
        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnDefaultDeviceChanged(nint _, int flow, int role, char* id)
    {
        Console.WriteLine($"EVENT type=default-device flow={flow} role={role} id={Escape(id == null ? string.Empty : new string(id))}");
        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnDevicePropertyChanged(nint _, char* id, PropertyKey __)
    {
        Console.WriteLine($"EVENT type=device-property id={Escape(new string(id))}");
        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvStdcall)])]
    private static int OnSessionCreated(nint _, nint control)
    {
        nint control2 = QueryInterface(control, IID_IAudioSessionControl2);
        if (control2 != 0)
        {
            uint pid = 0;
            if (SessionGetProcessId(control2, &pid) >= 0)
                Console.WriteLine($"EVENT type=session-created pid={pid} process={Escape(ProcessName(pid))}");
            Release(control2);
        }
        return 0;
    }

    [DllImport("ole32.dll")]
    private static extern int CoInitializeEx(nint reserved, uint flags);
    [DllImport("ole32.dll")]
    private static extern void CoUninitialize();
    [DllImport("ole32.dll")]
    private static extern int CoCreateInstance(Guid* clsid, nint outer, uint context, Guid* iid, nint* result);
    [DllImport("ole32.dll")]
    private static extern void CoTaskMemFree(nint memory);
    [DllImport("ole32.dll")]
    private static extern int PropVariantClear(PropVariant* value);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern nint OpenProcess(uint access, [MarshalAs(UnmanagedType.Bool)] bool inheritHandle, uint processId);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool QueryFullProcessImageNameW(nint process, uint flags, char* path, uint* size);
    [DllImport("kernel32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(nint handle);

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct PropertyKey(Guid formatId, uint propertyId)
    {
        public readonly Guid FormatId = formatId;
        public readonly uint PropertyId = propertyId;
    }

    [StructLayout(LayoutKind.Explicit, Size = 24)]
    private struct PropVariant
    {
        [FieldOffset(0)] public ushort Type;
        [FieldOffset(8)] public nint Pointer;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct CallbackObject
    {
        public nint VTable;
        public int References;
    }

    private sealed class Snapshot
    {
        public List<Device> Devices { get; } = [];
        public List<nint> SessionManagers { get; } = [];
        public int SessionCount { get; set; }
        public int UpdatedCount { get; set; }
    }

    private sealed record Device(string Id, string Name, bool IsDefault)
    {
        public List<Session> Sessions { get; } = [];
    }

    private sealed record Session(uint ProcessId, string State, string ProcessName, float VolumePercent, bool Muted);
    private sealed record Options(int HoldSeconds, uint? TargetPid, float? Volume, bool? Muted);
}
