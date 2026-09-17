## Why

Jalak's Rust audio core and macOS lifecycle are lightweight, but its hand-built GPUI controls still look and behave like a custom web-style interface. Migrating to GPUI Kit gives Jalak mature sliders, sidebars, menus, icons, semantic themes, focus behavior, and accessibility while preserving the existing Rust architecture.

## What Changes

- Replace the direct GPUI dependency and handwritten view primitives with the matching GPUI Kit release and its re-exported GPUI runtime.
- Raise the workspace minimum Rust version from 1.94 to 1.95, required by GPUI Kit's matching GPUI runtime.
- Introduce one small Jalak macOS theme using semantic colors, system typography, consistent spacing, system appearance, and accessible interaction states.
- Rebuild the management window from GPUI Kit sidebar, button, menu, tooltip, separator, and list components while preserving the current left-profile/right-content information architecture.
- Rebuild the menu-bar popup with GPUI Kit controls, including a real draggable volume slider, icon buttons, tooltips, and an on-demand profile menu.
- Preserve the existing AppKit `NSStatusItem`, Dockless lifecycle, popup anchoring, native file picker and confirmations, Rust application state, persistence format, and Rodio audio engine.
- Remove superseded custom button, rail, row, and menu helpers after every surface uses GPUI Kit; no parallel legacy component path remains.
- Measure release RSS, binary size, popup responsiveness, keyboard access, and light/dark visual behavior before accepting the migration.
- Keep a native AppKit popover rewrite outside this change; it remains a fallback only if GPUI Kit cannot meet the accepted macOS interaction requirements.

## Capabilities

### New Capabilities

- `desktop-component-system`: Defines Jalak's GPUI Kit dependency boundary, semantic macOS theme, reusable component policy, accessibility baseline, and performance guardrails.

### Modified Capabilities

- `menu-bar-audio-control`: Requires GPUI Kit controls, a draggable accessible volume slider, icon-based actions, an on-demand profile menu, and system light/dark appearance.
- `audio-profile-management`: Requires GPUI Kit sidebar/list interaction, contextual secondary actions, system appearance, and consistent keyboard/accessibility behavior.

## Impact

- Replaces direct `gpui` usage in `apps/desktop` with `gpui-kit` and its matching GPUI export.
- Raises the workspace MSRV to Rust 1.95; current stable Rust remains supported.
- Changes application bootstrap and both GPUI views; the macOS AppKit adapter changes only where GPUI Kit initialization or window roots require it.
- Leaves domain models, configuration files, managed audio, playback behavior, bundle identity, and user data unchanged.
- Adds migration and performance risk from GPUI Kit's pinned GPUI version, controlled by a compile-first dependency checkpoint and before/after release measurements.
