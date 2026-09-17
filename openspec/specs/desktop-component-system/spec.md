# Desktop Component System Specification

## Purpose

TBD - created by archiving change migrate-ui-to-gpui-kit. Defines Jalak's GPUI Kit dependency boundary, semantic macOS theme, component reuse policy, accessibility baseline, and performance guardrails.

## Requirements

### Requirement: Single GPUI Kit runtime
The application SHALL use one pinned GPUI Kit release and its matching re-exported GPUI runtime for desktop views, SHALL initialize the component system once before opening GPUI windows, and SHALL require stable Rust 1.95 or newer.

#### Scenario: Build with supported stable Rust
- **WHEN** Jalak is built with its declared minimum Rust toolchain
- **THEN** GPUI Kit and its matching GPUI runtime compile without nightly features, `RUSTC_BOOTSTRAP`, or a locally patched GPUI fork

#### Scenario: Launch migrated application
- **WHEN** Jalak starts after the migration
- **THEN** GPUI Kit is initialized before the management window or popup renders and both surfaces use the same GPUI runtime

#### Scenario: Inspect dependency graph
- **WHEN** the release dependency graph is inspected
- **THEN** it contains no second direct GPUI version and excludes unused shell, webview, editor, chart, inspector, and JavaScript-extension systems

### Requirement: Semantic macOS appearance
The desktop UI SHALL derive colors, typography, spacing, radii, focus treatment, and destructive treatment from a shared semantic theme that supports macOS light and dark appearance.

#### Scenario: Change system appearance
- **WHEN** macOS changes between light and dark appearance while Jalak is running
- **THEN** visible Jalak surfaces remain legible and update without retaining hardcoded dark-only backgrounds

#### Scenario: Render interactive state
- **WHEN** a control is hovered, pressed, focused, selected, disabled, or destructive
- **THEN** its semantic state is visually distinct without changing application data

### Requirement: Upstream component reuse
Jalak SHALL use GPUI Kit components for buttons, sliders, sidebars or lists, menus, tooltips, separators, and feedback when the library provides the required behavior.

#### Scenario: Complete component migration
- **WHEN** both desktop surfaces have migrated
- **THEN** superseded handwritten button, volume-rail, menu-row, and selection-row helpers are removed

#### Scenario: Upstream component gap
- **WHEN** GPUI Kit cannot provide required behavior
- **THEN** a Jalak-owned component is added only with a documented gap, focused verification, and at least two call sites, or the design is revised before implementation continues

### Requirement: Authoritative application state
GPUI Kit component state SHALL remain transient, and every durable profile or playback mutation SHALL route through the existing shared `AppState` commands.

#### Scenario: Commit slider value
- **WHEN** the user completes a volume gesture
- **THEN** the value is applied through `AppState`, persisted by the existing command path, and reflected by every visible surface

#### Scenario: Re-render component
- **WHEN** a view re-renders after an external state change
- **THEN** it reads the current profile and track values from `AppState` rather than a parallel durable UI store

### Requirement: Accessible component baseline
Interactive desktop components SHALL expose stable identities, appropriate roles, accessible names, current values or toggle states, visible focus, and keyboard actions.

#### Scenario: Operate without pointer
- **WHEN** the user traverses Jalak with the keyboard
- **THEN** profile selection, import, menus, playback, mute, management, and quit actions are reachable and operable in logical order; track volume sliders are pointer-only

#### Scenario: Inspect volume control
- **WHEN** assistive technology focuses a track volume slider
- **THEN** it receives the track name, slider role, current value, and available adjustment action

### Requirement: Bounded migration cost
The migration SHALL record comparable release measurements and SHALL NOT increase idle or popup-open RSS by more than 15 percent over the pre-migration baseline without explicit acceptance.

#### Scenario: Compare release measurements
- **WHEN** pre- and post-migration builds run with the same configuration and measurement procedure
- **THEN** binary size, idle RSS, popup RSS, and measurement conditions are recorded in the change proof

#### Scenario: Exceed memory guardrail
- **WHEN** post-migration idle or popup RSS exceeds baseline by more than 15 percent
- **THEN** the task remains incomplete until the regression is removed or explicitly accepted in the change artifacts
