# Changelog

## Unreleased

## 0.2.0 - 2026-09-29

Jalak 0.2 is the first packaged release: a menu-bar app that loops your own
ambient sounds as profiles, each with per-track volume and mute. The popup
and management window are rebuilt, follow the macOS light and dark
appearance, and wear a new icon and a jalak menu-bar logo. Resuming a profile
now starts each sound somewhere new and recovers audio after sleep or a
device change instead of playing silence.

### Added

- macOS menu-bar audio profiles with concurrent looping local tracks.
- Per-track playback, mute, and volume controls plus profile-wide pause.
- Managed local audio imports and durable profile settings.
- Installable Dockless macOS app bundle that opens profile management on launch.
- Compact content-sized menu popup with an on-demand profile switcher.
- Two-pane profile management with sidebar selection and focused track content.

### Changed

- Rebuilt the menu-bar popup and management window on GPUI Kit components:
  keyboard-navigable profile list, round icon buttons with tooltips, and
  draggable volume sliders in both the popup and the management window.
- The popup uses a profile play switch and Control Center style sound
  toggles under an Audio section, and sizes itself exactly to its content.
- Management rows show each sound's file path with play/pause preview and
  volume controls instead of its file size.
- Jalak now follows the macOS light and dark appearance.
- New app icon, generated from a PNG master instead of the previous SVG.
- The menu-bar item shows the jalak logo as a template image instead of a
  music-note glyph, tinted to match the menu bar appearance.
- Raised the minimum supported Rust version to 1.95.
- Resuming a profile restarts each sound from a random position instead of
  replaying the same passage every time.
- Configuration files, managed audio, and playback behavior are unchanged.

### Fixed

- Resuming a profile reopens the audio output device, so playback recovers
  after macOS tears the previous output down on sleep or a device change.
  Jalak previously kept the dead output and played silence until relaunched.
- Prevented reentrant popup teardown when opening Manage Profiles.
