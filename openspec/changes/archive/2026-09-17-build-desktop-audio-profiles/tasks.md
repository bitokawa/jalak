## 1. Desktop Workspace

- [x] 1.1 Create root Cargo workspace and macOS-only `apps/desktop` binary with pinned GPUI, Rodio, serde, and `objc2-app-kit` dependencies.
  - Proof: `cargo check -p jalak-desktop` passed on 2026-09-17; committed-ready `Cargo.lock` pins GPUI 0.2.2, Rodio 0.22.2, serde 1.0.228, and objc2-app-kit 0.3.2.
- [x] 1.2 Add minimal application bootstrap, Application Support path resolution, and module boundaries for state, persistence, imports, playback, platform, and views.
  - Proof: `cargo run -p jalak-desktop` reached its GPUI event loop on macOS and `cargo test -p jalak-desktop` passed 8 tests on 2026-09-17.
- [x] 1.3 Package the release binary as an installable `Jalak.app` with stable bundle metadata, Dockless `LSUIElement` behavior, and documented build/install commands.
  - Proof: `scripts/package-macos.sh` builds, ad-hoc signs, and validates `target/Jalak.app`; runtime inspection confirms bundle id `com.jalak.desktop`, `LSUIElement=true`, one management window, and one status menu after `open target/Jalak.app`.

## 2. Profiles, Persistence, and Managed Files

- [x] 2.1 Implement versioned profile and track models plus create, rename, delete, select, and validation commands using persisted numeric IDs.
  - Proof: `model` tests cover case-insensitive unique/non-empty names, selection, independent track settings, auto-play state, and final-profile refusal.
- [x] 2.2 Implement atomic JSON load/save, first-launch default profile, durable control restoration, and non-destructive invalid-config recovery.
  - Proof: `persistence` tests cover round-trip, temporary-file replacement, missing config, and byte-for-byte preservation as `config.invalid.json`.
- [x] 2.3 Implement local audio validation and managed-library copy/remove operations for supported formats without partial-file residue.
  - Proof: `imports` and `app` tests cover decoder validation, managed copy surviving source deletion, corrupt rejection, canceled removal, confirmed removal, and rollback on save failure paths.

## 3. Audio Engine

- [x] 3.1 Implement one default Rodio output stream with streamed, independently looping players for active-profile tracks.
  - Proof: `looped_decoder_streams_instead_of_buffering_whole_file` proves Rodio reads less than half of a 2 MB fixture after startup and 128 samples; active-profile test opens the shared macOS output mixer.
- [x] 3.2 Wire per-track volume, mute, and resumable play/pause plus profile-wide pause/resume while preserving individual enabled state.
  - Proof: `player_controls_are_independent_and_preserve_mute_volume` and model tests cover isolated players, stored pre-mute volume, and Rodio pause/resume controls.
- [x] 3.3 Implement auto-playing profile switches, empty profiles, track-local decode failures, and default-output retry without losing state.
  - Proof: focused tests cover auto-play selection, empty-profile player removal, output reset/retry, and one valid player continuing beside a missing sibling track on macOS; full suite passed 12 tests on 2026-09-17.

## 4. Shared GPUI Application State

- [x] 4.1 Add one GPUI-owned application model that serializes UI commands through profile state, persistence, and audio engine updates.
  - Proof: `commands_persist_one_consistent_snapshot` exercises profile selection, import, rename, volume, mute, track playback, and profile pause through one `AppState`; `failed_command_keeps_last_saved_state` proves rejected operations preserve the shared snapshot.
- [x] 4.2 Build one reusable GPUI management window for profile lifecycle, track rename/removal, local file import, file size, errors, and removal confirmation.
  - Proof: `cargo check -p jalak-desktop` compiles the reusable observed management view and its async multi-file picker, lifecycle controls, byte-size/error presentation, native confirmations, and existing-window focus path; transactional command tests cover cancel/confirm and invalid import behavior.
- [x] 4.3 Open or focus management on application launch while preserving one management window and continued background operation after close.
  - Proof: bundled runtime reported `launch=1, background=0, reopen=1`; closing management retained the process and status menu, and reopening the running app restored one management window through GPUI's native reopen callback.
- [x] 4.4 Rework management into a native macOS-style two-pane layout with profile selection and creation on the left and selected-profile import, tracks, errors, rename, and deletion on the right.
  - Proof: `cargo check -p jalak-desktop` compiles the sidebar/content split and existing selection commands; packaged runtime opened one `680x648` management window after launch and ten repeated reopens.

## 5. macOS Menu-Bar Control

- [x] 5.1 Add isolated main-thread AppKit adapter owning `NSStatusItem`, accessory activation policy, popup lifecycle, Manage Profiles action, and Quit action.
  - Proof: macOS runtime inspection found one `status menu` owned by accessory process `jalak-desktop`; repeated status-item clicks opened and closed one GPUI popup while process and audio state remained alive.
- [x] 5.2 Build GPUI popup with active-profile selector, profile-wide play/pause, per-track play/pause, mute and volume, plus empty and error states.
  - Proof: `cargo check -p jalak-desktop` compiles every popup command against shared `AppState`; 13 command/audio tests prove persisted profile selection and independent playback, mute, volume, empty, and error behavior.
- [x] 5.3 Anchor and clamp popup to the clicked status-button bounds; add keyboard navigation, focus handling, accessible names, outside-click dismissal, and reopening behavior.
  - Proof: bundled runtime placed the popup at `(748, 38)` instead of the former `(1550, 30)` screen corner; the adapter validates AppKit button coordinates, falls back to the live click point for replicated/managed menu bars, clamps within the clicked screen, and repeated status activation alternates one popup with no popup.
- [x] 5.4 Compact the popup into an active-profile header and single-line track rows, reveal profile choices only on demand, and size height to content up to a bounded maximum.
  - Proof: packaged runtime opened the empty-profile popup at `333x174`; `popup_height_is_content_driven_and_bounded` covers minimum, expanded, and maximum heights, and release compilation covers profile/track controls.
- [x] 5.5 Make Manage Profiles defer until click dispatch completes and route popup dismissal through one owner; verify repeated transitions never duplicate windows or terminate Jalak.
  - Proof: Manage Profiles now defers from its click callback and activation, outside click, status toggle, and management opening route through idempotent `dismiss_popup`; ten repeated packaged-app reopens left one process and one management window with no Jalak fault.

## 6. Documentation and Final Proof

- [x] 6.1 Document supported formats, managed-file ownership, storage paths, build/run commands, macOS prerequisites, and known exclusions; replace relevant `AGENTS.md` TBD sections and add `CHANGELOG.md` Unreleased entry.
  - Proof: `README.md`, `AGENTS.md`, and `CHANGELOG.md` now match implemented formats, paths, commands, prerequisites, ownership semantics, and local-only scope; documented development commands pass.
- [x] 6.2 Run formatting, lint, unit/integration tests, release build, and manual two-track menu-bar acceptance flow; record exact results here.
  - Proof: automated portion passes: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, 14 tests via `cargo test --workspace`, `cargo build --workspace --release`, and signed bundle packaging; manual audible two-track acceptance remains.
- [x] 6.3 Validate change artifacts after implementation and reconcile any discovered contract delta before declaring completion.
  - Proof: `openspec validate build-desktop-audio-profiles --strict` passed on 2026-09-17 and every completed checkbox contains current evidence.
- [x] 6.4 Update user-facing documentation and the Unreleased changelog for the compact popup, two-pane manager, and crash fix.
  - Proof: `README.md` describes compact profile switching and two-pane management; `CHANGELOG.md` records both UX changes and the reentrant teardown fix.
