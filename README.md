# Jalak

Jalak is a lightweight macOS menu-bar audio player. Profiles group multiple
local audio tracks that can loop together with independent playback, mute, and
volume controls.

## Requirements

- macOS
- Rust 1.95 or newer
- Xcode Command Line Tools
- Working default audio output device

## Build and install

```bash
./scripts/package-macos.sh
cp -R target/Jalak.app /Applications/
open /Applications/Jalak.app
```

Opening Jalak shows its management window. Closing that window leaves audio and
the menu-bar process running. Use **Manage Profiles** to reopen it; use
**Quit Jalak** when you want to stop the process.

For development without packaging:

```bash
cargo run -p jalak-desktop
```

Jalak runs as a Dockless accessory application. Use the `♫` menu-bar item for
quick controls. The compact popup shows the active profile with a switch that
pauses or resumes all of its sounds, then an **Audio** list: each sound has a
round toggle (filled while it plays) and a draggable volume slider. Select
the profile name to switch profiles from a menu that opens over the popup
without resizing it. The management window keeps profiles in its left sidebar
and the selected profile in the right pane: each sound shows its managed file
path, a play/pause button to preview it, and a volume slider. Rename and delete
are the pencil and trash buttons, also available from each row's right-click
menu.

## Interface

Both windows are drawn by [GPUI Kit](https://gpui-kit.com) components (pinned
`gpui-kit` release, which bundles its matching GPUI runtime) with a Jalak theme
that follows the macOS light or dark appearance. Controls are GPUI-rendered,
not AppKit widgets; the menu-bar item, file picker, and name/confirmation
dialogs remain native AppKit.

Keyboard support:

- Management window: Tab and Shift-Tab move between the profile list, New
  Profile, Import Audio, and the rename, delete, and play buttons; Space
  activates the focused button.
- Profile sidebar: Up/Down selects the previous or next profile.
- Menus: Up/Down moves, Return activates, Escape closes.
- Volume sliders expose VoiceOver increment/decrement; GPUI Kit 0.6.1 sliders
  do not yet accept keyboard focus or arrow keys.

## Local data

Jalak stores data under `~/Library/Application Support/Jalak/`:

- `config.json` contains profiles and control state.
- `audio/` contains app-managed copies of imported files.
- `config.invalid.json` preserves an unreadable configuration during recovery.

Importing copies a file into Jalak's managed library, so playback no longer
depends on the source file. Removing a track or profile also removes its
managed copies after confirmation. Supported extensions are MP3, WAV, FLAC,
OGG, M4A, and MP4; every file must also pass decoder validation.

## Current scope

Version 0.1 plays local files only. YouTube URLs, remote streaming, media keys,
global shortcuts, and cross-device sync are not included.

## Verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```
