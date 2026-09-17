# Changelog

## Unreleased

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
- Raised the minimum supported Rust version to 1.95.
- Configuration files, managed audio, and playback behavior are unchanged.

### Fixed

- Prevented reentrant popup teardown when opening Manage Profiles.
