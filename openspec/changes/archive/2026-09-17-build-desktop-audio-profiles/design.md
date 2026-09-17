## Context

Jalak currently contains OpenSpec scaffolding but no application workspace. This change establishes a macOS-only Rust desktop application under `apps/desktop`. Its primary interaction is a menu-bar popup for frequent controls; a separate GPUI window handles less frequent profile and track management.

Audio must keep playing while application windows are closed, mix several long-running files with low memory overhead, and preserve control state across launches. Imported files must remain usable if their original source is moved or deleted.

## Goals / Non-Goals

**Goals:**

- Provide responsive, concurrent playback of multiple looping local tracks.
- Make active-profile and track controls reachable from the macOS menu bar.
- Keep one discoverable state model shared by audio, menu-bar, and management views.
- Own imported media and save configuration durably with minimal infrastructure.
- Isolate macOS-specific unsafe integration from portable domain and audio code.

**Non-Goals:**

- YouTube or other network import, streaming, accounts, or cloud synchronization.
- Windows or Linux tray integration.
- Audio editing, waveform display, equalization, effects, or output-device routing.
- Persisting playback positions across application restarts.
- A database, background service, plugin system, or generalized media framework.

## Decisions

### Use one Rust workspace with one desktop crate

Root Cargo workspace will initially contain only `apps/desktop`. The application remains one crate with modules for domain state, persistence, imports, playback, macOS menu-bar integration, and GPUI views. Modules may become crates only after a measured build or ownership problem appears.

Alternative considered: multiple domain, platform, and UI crates. Rejected because one application has no independent consumers and early crate boundaries would add ceremony without isolation benefit.

### Keep one authoritative application state

One GPUI-owned application model will hold profiles, active-profile ID, persisted control values, transient playback state, and per-track errors. Menu-bar and management views dispatch commands into that model; they do not own parallel copies. The model coordinates persistence and the audio engine after each accepted command.

Alternative considered: independent UI and audio stores synchronized through channels. Rejected because this application runs in one process and a second store creates avoidable reconciliation paths.

### Mix active tracks through one Rodio output stream

The audio engine will open one default output stream and attach one controllable player per track in the active profile to its mixer. Decoders will stream file data rather than load entire files. Each track loops independently.

Stored volume remains in the `0.0..=1.0` range. Muting applies effective volume zero without overwriting stored volume. Individual pause preserves that player's position. Profile-wide pause and resume operate on every player. Switching profile drops old players, constructs players for the new profile, and begins playback automatically; a failed track reports an error without preventing other valid tracks from playing.

Alternative considered: one OS output stream per track. Rejected because a single mixer provides simpler lifecycle, coordination, and device handling.

### Copy imports into a managed application library

An import validates and decodes the selected file before copying it under `~/Library/Application Support/Jalak/audio/`. Managed filenames use the track's persisted numeric ID plus retained extension, avoiding collision and filename-sanitization logic. Initial supported formats are MP3, WAV, FLAC, OGG Vorbis, and M4A/AAC where the configured decoder accepts the file.

Deleting a track removes its managed file only after the state update can be persisted. Adding the same source twice creates separate track entries so each profile placement can retain independent settings.

Alternative considered: retaining original file paths. Rejected because moved files would break profiles and sandboxed distribution would later require security-scoped bookmark handling.

### Persist versioned JSON atomically

Profiles and durable settings will live in one versioned JSON document under Application Support. IDs come from a persisted monotonically increasing counter; no UUID dependency is needed. Saves write a sibling temporary file and rename it over the previous configuration. Startup with no config creates one empty default profile. Invalid config is preserved for diagnosis and produces a visible recovery error instead of being silently overwritten.

Alternative considered: SQLite. Rejected because the data is small, single-process, and replaced as one document.

### Use an installable Dockless app with AppKit status item and GPUI surfaces

A narrow `objc2-app-kit` adapter will own `NSStatusItem`, status-button actions, and application activation policy. A standard `Jalak.app` bundle uses `LSUIElement=true`, matching the Macshot lifecycle: launching the app opens or focuses its management window without adding a persistent Dock icon, while closing that window leaves the status item and audio engine running. Clicking the icon opens a compact GPUI popup anchored below the clicked status button. The popup contains profile selection, global play/pause, per-track play/pause, mute and volume, plus Manage Profiles and Quit actions. Manage Profiles opens or focuses the same regular GPUI window.

The application bundle contains its executable, `Info.plist`, and app icon resources and can be copied into `/Applications`. Development packaging remains a small repository script instead of a packaging dependency. Unsafe AppKit calls remain confined to the adapter and execute on the main thread.

Alternative considered: a native `NSMenu` containing custom controls. Rejected because sliders and richer row state fit a GPUI popup better and would split UI implementation across frameworks.

The GPUI popup follows macOS Control Center rather than a generic application card: blurred window material, compact grouped rows, muted section labels, blue active state, and horizontal volume rails. Its default state shows one active-profile header and single-line track controls; the full profile list appears only when the user requests profile switching. Height follows visible content up to a bounded maximum instead of reserving a fixed tall panel.

The management window uses one persistent split layout. A narrow left sidebar owns profile selection and creation. The right content pane owns the selected profile's audio import and track management. Row selection replaces redundant Select buttons; secondary rename and delete actions remain visually subordinate.

Opening management from the popup is deferred until the current popup click dispatch completes. Popup dismissal has one registry-owned path; focus loss requests that same path instead of independently destroying the GPUI window. This prevents reentrant or duplicate window removal while preserving outside-click dismissal.

### Pin unstable dependencies and keep platform scope explicit

GPUI remains pre-1.0, so Cargo.lock and an exact compatible dependency revision/version will be committed. AppKit integration is macOS-gated. No cross-platform abstraction will be created until another platform is in scope.

## Risks / Trade-offs

- **GPUI or AppKit integration changes upstream** → Pin dependencies and cover popup lifecycle with focused runtime proof.
- **Malformed or unsupported media fails during import or playback** → Validate through the same decoder path, reject without copying partial files, and display track-specific errors.
- **Audio output disappears or changes** → Keep profiles intact, surface engine failure, and allow a later play command to reopen the default device.
- **Atomic rename or media deletion fails** → Report the filesystem error and keep the last valid persisted configuration; never silently discard user state.
- **Large libraries consume disk space** → Show managed track size and require confirmation before destructive deletion; deduplication remains deferred.
- **Status popup positioning differs across displays** → Derive placement from the status-button screen frame and clamp to the active screen's visible bounds.
- **App launch and background behavior diverge** → Exercise bundle launch, management-window close/reopen, menu-bar persistence, and clean quit as one runtime flow.
- **Popup-to-management transition reenters window teardown** → Defer management opening, route dismissal through one owner, and repeatedly exercise the transition.

## Migration Plan

No existing application data requires migration. First launch creates Application Support directories and a version-1 configuration with one empty default profile. During development, rollback consists of removing the new desktop workspace and its generated local Application Support data; no repository data is transformed.

Future configuration changes must increment the document version and add an explicit migration before writing newer data.

## Open Questions

None blocking implementation. Exact signed application bundle and distribution channel remain release work, not this feature change.
