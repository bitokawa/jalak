## ADDED Requirements

### Requirement: Component-based management workflow
The management window SHALL use GPUI Kit sidebar, list, button, menu, tooltip, separator, empty-state, and feedback components while preserving profile and track behavior.

#### Scenario: Manage profiles from sidebar
- **WHEN** the user selects, creates, renames, or deletes a profile
- **THEN** the sidebar selection and right content remain synchronized through the shared application state

#### Scenario: Use secondary profile action
- **WHEN** the user opens a profile's contextual action menu
- **THEN** rename and eligible delete actions are available without permanent action buttons on every row

#### Scenario: Use track actions
- **WHEN** the user activates a track's rename or delete icon button or opens its right-click menu
- **THEN** rename and delete actions are available and destructive deletion still requires confirmation

#### Scenario: Preview and adjust a track
- **WHEN** the selected profile contains tracks
- **THEN** each track row shows its name, its managed file path, a play/pause button, and a draggable volume slider that route through the shared application state, and does not show the file size

#### Scenario: View empty profile
- **WHEN** the selected profile contains no tracks
- **THEN** the content pane shows a system-aware empty state with a primary Import Audio action

### Requirement: Management keyboard and appearance behavior
The management window SHALL support logical keyboard traversal, visible focus, accessible component metadata, and macOS light and dark appearance.

#### Scenario: Navigate management by keyboard
- **WHEN** the user navigates the management window without a pointer
- **THEN** sidebar selection, profile creation, import, contextual actions, and track actions are reachable in logical order

#### Scenario: Change system appearance
- **WHEN** macOS changes appearance while the management window is visible
- **THEN** sidebar, content, selection, controls, menus, errors, and empty states update to matching semantic colors
