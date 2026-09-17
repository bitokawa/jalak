## Context

Jalak is one Rust desktop crate. `AppState` owns profiles, persistence, errors, and the Rodio engine; two GPUI views render a menu-bar popup and management window; a narrow `objc2-app-kit` adapter owns the status item and native lifecycle. The views currently implement their own buttons, rows, volume rail, colors, focus styling, and icons.

GPUI Kit 0.6 packages a matching GPUI runtime with component, theme, icon, menu, sidebar, slider, tooltip, and accessibility support. Because GPUI Kit pins and re-exports GPUI, migration must avoid linking or importing a second direct GPUI version. User configuration and audio files cannot require migration.

GPUI Kit's matching GPUI runtime calls `std::hint::cold_path`, stabilized in Rust 1.95. The workspace therefore raises its MSRV from Rust 1.94 to 1.95 and uses current stable Rust for implementation and verification.

## Goals / Non-Goals

**Goals:**

- Replace handwritten interaction primitives with maintained GPUI Kit components.
- Make the popup and manager follow macOS system appearance, hierarchy, control behavior, keyboard operation, and accessibility conventions.
- Keep one authoritative `AppState` and preserve all audio, persistence, AppKit lifecycle, and packaging behavior.
- Keep runtime and dependency cost measurable and bounded.
- Finish with one component path; delete superseded custom helpers.

**Non-Goals:**

- Rewriting the audio engine, data model, persistence format, import pipeline, or AppKit status-item adapter.
- Replacing GPUI Kit with SwiftUI, Cacao, Slint, Iced, egui, or a native AppKit view hierarchy.
- Adding cross-platform support, plugins, webviews, editors, charts, or speculative design-system infrastructure.
- Reproducing every macOS control in a Jalak-owned component library.

## Decisions

### Use GPUI Kit as the only GPUI dependency

Replace the workspace's direct `gpui` dependency with a pinned GPUI Kit release and import GPUI types through GPUI Kit's re-export. Initialize GPUI Kit once during application startup and wrap each GPUI window in the root element required by the component library.

The first implementation checkpoint is dependency-only: resolve the minimum required features, compile the existing application, inspect `cargo tree -d`, and record release binary/RSS baselines. Shell, webview, editor, chart, inspector, and JavaScript-extension features stay disabled or absent.

Alternative considered: add GPUI Kit beside `gpui = 0.2.2`. Rejected because mismatched GPUI types and duplicate runtime versions create integration risk.

### Require stable Rust 1.95 or newer

Set workspace `rust-version` and contributor documentation to Rust 1.95. GPUI Kit's GPUI runtime requires the stable `std::hint::cold_path` API, so Rust 1.94 cannot compile the dependency. Development and release verification use the current stable toolchain; no nightly feature, `RUSTC_BOOTSTRAP`, or locally patched GPUI fork is permitted.

Alternative considered: patch GPUI locally or enable unstable APIs on Rust 1.94. Rejected because either path creates a private runtime fork for a compiler optimization hint and weakens reproducibility.

### Keep application state authoritative

GPUI Kit component state is transient presentation state only. Slider, menu, sidebar, and focus events dispatch existing `AppState` commands. Persisted volume, selection, pause, mute, and names continue to come from `AppState`; components never become a second durable store.

Slider gestures may maintain temporary drag state, but each committed value routes through `set_track_volume`. State changes notify both surfaces through their existing observation path.

Alternative considered: mirror profile and track data into a component-level store. Rejected because reconciliation would be more code than the current single-state model.

### Use GPUI Kit components directly with one small Jalak theme

Use GPUI Kit `Button`, `Slider`, sidebar/list, menu, tooltip, separator, and feedback components directly. Add only a small theme module that maps Jalak's semantic roles to system-aware colors, system typography, spacing, radius, focus, and destructive states. Do not wrap every upstream component. A Jalak-owned component requires a documented upstream gap and at least two call sites.

Use GPUI Kit's supplied icon mechanism rather than Unicode glyphs. Control labels, accessible names, values, toggle state, and stable IDs remain explicit.

Alternative considered: build a full Jalak design system above GPUI Kit. Rejected because two small surfaces do not justify another abstraction layer.

### Migrate management before popup

Management is a regular window and safer proving ground for initialization, themes, root setup, buttons, sidebar selection, menus, empty states, and keyboard focus. Preserve the current left sidebar/right content information architecture. Creation stays in the sidebar footer; import stays primary in content header; rename/delete move to contextual menus.

After management passes focused checks, migrate the popup. Replace the display-only rail and plus/minus buttons with one draggable slider. Replace the expanding profile list with an anchored menu so popup size remains stable. Keep the existing AppKit status item, positioning, outside-click dismissal, and deferred Manage Profiles transition.

Alternative considered: migrate both surfaces in one rewrite. Rejected because failures would conflate dependency, component, and popup-lifecycle problems.

### Delete legacy helpers at cutover

Once each surface is migrated, remove `button`, `round_button`, `small_button`, `menu_button`, `volume_rail`, duplicated hardcoded colors, and superseded row construction. No compatibility wrappers or runtime switch remains.

Alternative considered: retain old controls as fallback. Rejected because two UI systems increase maintenance and can silently drift.

### Accept migration through behavior, accessibility, visuals, and measurements

Run existing unit tests unchanged, add focused component/state tests where new event translation is non-trivial, and exercise packaged lifecycle flows. Record release binary size and idle/popup RSS using the same fixture before and after migration. Post-migration idle and popup RSS SHALL not exceed the recorded baseline by more than 15% without explicit acceptance.

Verify light and dark appearance, keyboard traversal, slider adjustment, menu dismissal, VoiceOver metadata, multi-display popup bounds, repeated Manage Profiles transitions, and one-window ownership. A visually polished screenshot does not replace interaction proof.

## Component Conventions

- Icons come from GPUI Kit's default component set plus a compile-time `icon_assets!` subset (`AudioLines`, `Music`, `Power`, `Volume2`, `VolumeX`) composed in `assets.rs`; no Unicode control glyphs.
- Every interactive element has a stable ID derived from its purpose and, for per-row controls, the profile or track ID (`("mute", track_id)`).
- Icon-only buttons set `accessibility_label` naming the action and its target and a `tooltip` naming the action; state changes swap both (`Pause All` / `Resume All`).
- Two-state track controls use `Button::toggled`, which GPUI Kit exposes as an accessibility checkbox with a checked value.
- Icon-only controls use `views::icon_button`: a 30 px filled circle (secondary fill token) with a 15 px glyph, macOS control style. Rename and delete are explicit Pencil/Trash icon buttons (profile header and each track row) and are mirrored by right-click menus; both call the same `views::rename_*` / `views::delete_*` actions. No ellipsis (`…`) triggers or ellipsis labels are used (review feedback 2026-09-17).
- The popup follows Control Center: a profile play `Switch` (macOS style: accent track with a white thumb in both appearances), an "Audio" section title, and a leading circular toggle per track that uses the primary accent fill while the track is active (`Track::is_active`). Popup height is computed from the same layout constants used to render it.
- Volume sliders are shared by the popup and management rows through `views::track_sliders::TrackSliders`.
- The theme rebuilds GPUI Kit's legacy `ThemeTokens` from the tuned colors, because buttons read their background from those tokens (dark-mode primary buttons were otherwise near-white on white).
- Durable mutations go through `views::run`, which calls one `AppState` command and surfaces its error.
- The profile column uses GPUI Kit `List` rather than `Sidebar`, because `SidebarMenuItem` has no focus, role, or keyboard selection while `List` provides `Role::List`/`ListItem`, `aria_selected`, and Up/Down/Enter bindings.

## Upstream Gaps (GPUI Kit 0.6.1)

Recorded 2026-09-17 from source inspection and runtime automation:

1. `Slider` has no keyboard focus or arrow-key handling and no API for an accessible name; VoiceOver sees an unnamed slider with value, range, step, and `AXIncrement`/`AXDecrement`. Its increment/decrement call `set_value`, which does not emit `SliderEvent`, so Jalak observes the slider entity instead of subscribing to events. Decision 2026-09-17: keyboard slider adjustment is out of scope; sliders are pointer- and assistive-technology-operated only, and no wrapper is added. The missing accessible name remains an open gap against the "Inspect volume control" scenario, to be reviewed in 6.3.
2. `ListState`'s focus handle is not a tab stop. The management view wraps the list in a tab-stop element whose focus is forwarded to the list (`profiles_tab_stop`); verified Tab cycle: list -> New Profile -> Import Audio -> Profile actions -> track actions -> list.
3. `dropdown_menu` opens on pointer click but did not open on an `AXPress` from System Events; Jalak now uses it only for the popup profile switcher.
4. `gpui-component` compiles its `chart`/`plot` modules and depends on `lsp-types` and `markdown` unconditionally; no feature flag removes them.
6. `ContextMenu` (`context_menu`) removes focusable descendants from the Tab order. Track rows therefore attach the right-click menu to the name block only, leaving the actions button outside the wrapper.
5. GPUI static text is not exposed through the accessibility tree, so visual-only content (headers, counts, errors) needs manual VoiceOver review.

No Jalak-owned component was introduced for any gap; the list tab-stop forwarder is a few lines inside the management view.

## Risks / Trade-offs

- **GPUI Kit pins a different GPUI API** → Complete dependency/bootstrap checkpoint before changing views; use only GPUI Kit's re-export.
- **Rust 1.94 cannot compile GPUI Kit's GPUI runtime** → Raise the workspace MSRV to stable Rust 1.95 and verify with current stable Rust.
- **Default features pull unrelated systems** → Inspect feature graph and disable shell, webview, editor, chart, inspector, and language tooling.
- **Component state diverges from `AppState`** → Treat component state as transient and route every durable mutation through existing commands.
- **Native-like theme still feels cross-platform** → Validate both surfaces visually; keep AppKit popup rewrite as a separate future decision, not a mixed fallback.
- **Popup regression reintroduces crashes or poor anchoring** → Preserve platform adapter ownership and repeat lifecycle/multi-display checks after migration.
- **New components increase RSS or binary size** → Record comparable before/after release measurements and enforce the 15% RSS guardrail.
- **Accessibility claims exceed actual metadata** → Inspect roles, names, values, toggle states, focus order, and keyboard actions rather than relying on visual focus alone.

## Migration Plan

1. Record current release dependency tree, binary size, idle RSS, popup RSS, and lifecycle results.
2. Replace direct GPUI dependency with pinned GPUI Kit, initialize it, add required root wrappers, and restore a green compile before changing appearance.
3. Add the semantic Jalak theme and migrate management components.
4. Migrate popup components and slider/profile-menu behavior without changing AppKit ownership.
5. Remove legacy primitives and unused dependencies/assets.
6. Run focused behavior/accessibility/visual checks, full Rust gates, release packaging, and before/after measurements.

Rollback is code-only: restore the pinned direct GPUI dependency and prior view files. Configuration, managed audio, bundle identity, and stored user data remain compatible.

## Open Questions

- Exact GPUI Kit feature flags and icon asset subset will be resolved by the dependency checkpoint because the published feature graph is authoritative.
- If GPUI Kit cannot expose required slider accessibility values or popup menu behavior, stop and update this design rather than adding a second custom control path.
