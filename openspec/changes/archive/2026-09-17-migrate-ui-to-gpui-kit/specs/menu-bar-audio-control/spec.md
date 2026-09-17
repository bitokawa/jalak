## MODIFIED Requirements

### Requirement: Quick profile controls
The menu-bar popup SHALL use GPUI Kit controls to show the active profile, expose profile-wide play/pause as a switch, and open profile choices in an on-demand anchored menu.

#### Scenario: Select profile from popup
- **WHEN** the user selects another profile from the anchored profile menu
- **THEN** the menu dismisses, the popup reflects the new active profile, and the application initiates its auto-play switch

#### Scenario: Pause from popup
- **WHEN** the user turns off the labeled profile-wide switch
- **THEN** all eligible active-profile tracks pause and the switch, tooltip, and accessible value change to off

### Requirement: Quick track controls
The menu-bar popup SHALL use GPUI Kit components to list every track in the active profile under an "Audio" section title, each with its name, a leading circular on/off toggle that fills with the accent color while the track is playing and audible, and a draggable volume slider.

#### Scenario: Toggle a track from popup
- **WHEN** the user activates a track's circular toggle
- **THEN** an active track pauses, an inactive track plays unmuted, and the toggle's fill and accessible state update

#### Scenario: Drag track volume
- **WHEN** the user clicks or drags a track volume slider
- **THEN** the track gain changes continuously within its valid range without altering sibling tracks

#### Scenario: Volume slider is pointer-only
- **WHEN** the user navigates the popup with the keyboard
- **THEN** track volume sliders are not required to take focus or respond to adjustment keys; pointer and assistive-technology adjustments remain supported

#### Scenario: Active profile has no tracks
- **WHEN** the popup opens for an empty profile
- **THEN** it presents a GPUI Kit empty state and a route to open profile management

### Requirement: Profile management surface
The application SHALL provide a GPUI Kit management window with profiles in a left sidebar and selected-profile content in a right pane, reachable from the menu-bar popup.

#### Scenario: Open management window
- **WHEN** the user activates Manage Profiles
- **THEN** the application dismisses the popup after click dispatch and opens or focuses one management window without duplicating or terminating it

#### Scenario: Select profile in management window
- **WHEN** the user selects a profile row in the GPUI Kit sidebar
- **THEN** the right pane displays that profile's import action, tracks, errors, and contextual management actions

### Requirement: Accessible and bounded popup
The menu-bar popup SHALL use labeled GPUI Kit controls, remain keyboard-operable, and remain inside the visible bounds of the screen containing the status item.

#### Scenario: Keyboard operation
- **WHEN** the user navigates the popup without a pointer
- **THEN** profile menu, playback, mute, management, and quit controls are reachable in logical order with visible focus

#### Scenario: Status item near screen edge
- **WHEN** the status item is near a screen edge
- **THEN** the popup is placed directly below the clicked status button and clamped within that screen's visible bounds rather than aligned to the screen corner

### Requirement: Native macOS popup presentation
The menu-bar popup SHALL combine the existing blurred macOS popup shell with GPUI Kit semantic components, system light/dark appearance, icon actions, compact single-line hierarchy, and a stable content-bounded size that ends directly below the Quit row.

#### Scenario: Open native-styled popup
- **WHEN** the user opens the menu-bar popup in light or dark appearance
- **THEN** the active profile, sliders, icons, selection, focus, and secondary actions use the matching semantic appearance without Unicode control glyphs

#### Scenario: Switch profile from compact popup
- **WHEN** the user activates the profile control
- **THEN** an anchored menu reveals profile choices without expanding the popup or opening the management window
