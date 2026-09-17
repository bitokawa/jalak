## ADDED Requirements

### Requirement: Persistent macOS menu-bar access
The application SHALL expose a macOS menu-bar status item while running, and closing its popup or management window SHALL NOT stop audio playback or quit the application.

#### Scenario: Launch installed application
- **WHEN** the user opens Jalak from Finder or Applications
- **THEN** Jalak opens or focuses its profile management window and also exposes its menu-bar status item

#### Scenario: Open popup
- **WHEN** the user activates the menu-bar status item
- **THEN** the application opens its control popup adjacent to that item

#### Scenario: Close visible surfaces
- **WHEN** the user closes the popup and management window during playback
- **THEN** playback continues and the status item remains available

#### Scenario: Reopen management from background
- **WHEN** Jalak is running without a management window and the user activates Manage Profiles
- **THEN** Jalak opens one management window and brings it to the front

#### Scenario: Repeatedly open management from popup
- **WHEN** the user repeatedly opens the popup and activates Manage Profiles
- **THEN** each activation closes at most one popup, opens or focuses exactly one management window, and does not terminate the application

### Requirement: Installable macOS application
The application SHALL build as a standard `Jalak.app` bundle with stable bundle metadata and Dockless background-agent behavior.

#### Scenario: Install application bundle
- **WHEN** the user builds the release bundle and copies it into Applications
- **THEN** macOS recognizes Jalak as an application that can be launched normally

#### Scenario: Close management window
- **WHEN** the user closes Jalak's management window
- **THEN** the application remains running through its menu-bar status item without a persistent Dock icon

### Requirement: Quick profile controls
The menu-bar popup SHALL show the active profile, allow profile selection, and expose profile-wide play/pause.

#### Scenario: Select profile from popup
- **WHEN** the user selects another profile in the popup
- **THEN** the popup reflects the new active profile and initiates its auto-play switch

#### Scenario: Pause from popup
- **WHEN** the user activates profile-wide pause in the popup
- **THEN** all eligible active-profile tracks pause and the displayed control changes to resume

### Requirement: Quick track controls
The menu-bar popup SHALL list every track in the active profile with its name, play/pause state, mute state, and volume control.

#### Scenario: Adjust track from popup
- **WHEN** the user changes a track control in the popup
- **THEN** playback and every visible surface reflect the same updated state

#### Scenario: Active profile has no tracks
- **WHEN** the popup opens for an empty profile
- **THEN** it presents an empty state and a route to manage profiles

### Requirement: Profile management surface
The application SHALL provide a GPUI management window with profiles in a left sidebar and selected-profile content in a right pane, reachable from the menu-bar popup.

#### Scenario: Open management window
- **WHEN** the user activates Manage Profiles
- **THEN** the application opens or focuses one management window without duplicating it

#### Scenario: Select profile in management window
- **WHEN** the user selects a profile row in the left sidebar
- **THEN** the right pane displays that profile's import action, tracks, errors, and management actions

### Requirement: Accessible and bounded popup
The menu-bar popup SHALL expose keyboard-reachable labeled controls and remain inside the visible bounds of the screen containing the status item.

#### Scenario: Keyboard operation
- **WHEN** the user navigates the popup without a pointer
- **THEN** profile, playback, mute, volume, management, and quit controls are reachable and have accessible names

#### Scenario: Status item near screen edge
- **WHEN** the status item is near a screen edge
- **THEN** the popup is placed directly below the clicked status button and clamped within that screen's visible bounds rather than aligned to the screen corner

### Requirement: Native macOS popup presentation
The menu-bar popup SHALL use macOS Control Center visual conventions: a compact translucent panel, one active-profile header, single-line track rows, blue active accents, and horizontal volume controls. Its height SHALL follow visible content up to a bounded maximum, and the full profile list SHALL remain hidden until profile switching is requested.

#### Scenario: Open native-styled popup
- **WHEN** the user opens the menu-bar popup
- **THEN** the active profile and its sound controls are visible without an always-expanded profile list, and management actions remain visually secondary

#### Scenario: Switch profile from compact popup
- **WHEN** the user activates the profile header
- **THEN** the popup reveals profile choices without opening the management window
