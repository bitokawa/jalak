## Why

Playing separate YouTube videos for ambient sound and focus music repeatedly consumes bandwidth and makes coordinated playback controls awkward. Jalak needs a lightweight macOS menu-bar application that mixes reusable local audio tracks and controls them as one focus environment.

## What Changes

- Add an installable Rust macOS application bundle backed by `apps/desktop`, using GPUI for application views and a persistent menu-bar entry for quick access.
- Open or focus the management window when Jalak is launched as an app, while keeping the Dockless menu-bar process alive after that window closes.
- Add profiles that group multiple managed local audio tracks.
- Import supported local audio files into an application-owned library so playback does not depend on source files remaining in place.
- Mix all tracks in the active profile concurrently, with independent volume, mute, and play/pause controls.
- Add profile-wide play/pause that resumes track positions, and auto-play the newly selected profile when switching profiles.
- Keep the menu-bar popup compact by showing only the active profile and its tracks by default, with profile switching available on demand.
- Present profile management as a native macOS-style two-pane window with profiles on the left and selected-profile content on the right.
- Make the Manage Profiles transition safe under repeated clicks by giving popup dismissal one non-reentrant lifecycle path.
- Persist profiles, track settings, active-profile selection, and playback preferences across launches.
- Keep YouTube import, cloud sync, accounts, equalization, and waveform editing outside this change.

## Capabilities

### New Capabilities

- `audio-profile-management`: Create, edit, delete, select, and persist profiles and their managed local audio tracks.
- `multi-track-audio-playback`: Concurrent looping playback with per-track and profile-wide controls, including resumable pause and profile switching.
- `menu-bar-audio-control`: macOS menu-bar access to active-profile selection and essential playback controls, backed by a GPUI management window.

### Modified Capabilities

None.

## Impact

- Introduces root Rust workspace configuration and new `apps/desktop` application.
- Adds GPUI, Rust audio playback/decoding, serialization, and macOS AppKit binding dependencies.
- Writes configuration and imported audio under Jalak's macOS Application Support directory.
- Adds macOS application packaging and focused unit/runtime verification for persistence, playback state, imports, and menu-bar behavior.
- Adds an application lifecycle where Finder/Applications launch reaches management and the status item remains the background entry point.
- Refines popup and management-window interaction without adding another UI framework or application state store.
