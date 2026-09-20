# Audio Routing

This context describes how people direct application audio to output devices
across operating systems without assuming that every system exposes identical
controls.

## Language

**Application**:
A user-recognizable program that owns one or more audio streams. Its identity survives process termination, restart, and PID changes.
_Avoid_: Process, PID, session

**Application Identity**:
The strongest stable identity supplied by the platform for one Application, with canonical executable path as fallback. Display name, icon, and PID are not identity.
_Avoid_: Process name, executable name, PID

**Application Reassociation**:
An explicit user decision to attach existing Application Routes to an Application whose Application Identity changed.
_Avoid_: Automatic name matching

**Application Route**:
A persistent assignment from an Application to its Desired Output. It applies to the application's current and future audio streams.
The interface groups child processes and audio sessions beneath the Application so users can expand and inspect them without losing the stable application-level route.
_Avoid_: Session route, PID route, temporary route

**Application Volume**:
The current volume and mute state applied to an Application's active audio sessions. Adufa changes this live state but does not persist or repeatedly enforce it; the operating system remains responsible for any volume restoration.
_Avoid_: Saved volume, route volume, preferred volume

**Desired Output**:
The Output Device selected by the user for an Application Route. It remains selected while unavailable.
_Avoid_: Current device, active device

**Effective Output**:
The Output Device currently receiving an Application's audio. It may temporarily be the system default while the Desired Output is unavailable.
_Avoid_: Saved device, preferred device

**Output Device**:
An operating-system playback destination to which application audio can be directed.
_Avoid_: Speaker when referring to every kind of output

**Device Identity**:
The platform-provided stable identity of one exact Output Device. Display name, manufacturer, and model are descriptive attributes, not identity.
_Avoid_: Device name, friendly name

**Device Reassociation**:
An explicit user decision to replace an unavailable Desired Output with a similar Output Device whose Device Identity differs.
_Avoid_: Automatic name matching

**Platform Capability**:
An audio-routing operation that a specific operating system can reliably provide. Product behavior is defined explicitly per capability instead of assuming feature parity.
_Avoid_: Universal feature, cross-platform guarantee

**Sound Locator**:
A temporary inspection mode that shows the current signal level of every audible Application and emphasizes the strongest source. It exists only while the compact popup remains open and does not discover Applications, refresh the application list, or change an Application Route.
_Avoid_: Rescan, refresh, application search
