## ADDED Requirements

### Requirement: Concurrent looping playback
The application SHALL mix every enabled track in the active profile concurrently through one output device, loop each track independently, and stream decoding without buffering entire audio files in memory.

#### Scenario: Play profile with multiple tracks
- **WHEN** the user plays a profile containing multiple valid tracks
- **THEN** all enabled tracks begin or resume concurrent looping playback

#### Scenario: One track fails
- **WHEN** one active-profile track cannot be opened or decoded
- **THEN** the application reports that track's error while other valid tracks continue playing

### Requirement: Independent track controls
The application SHALL provide independent play/pause, mute, and volume controls for every active-profile track.

#### Scenario: Change one track volume
- **WHEN** the user changes one track's volume
- **THEN** that track's gain changes without altering another track's gain or playback state

#### Scenario: Mute and unmute track
- **WHEN** the user mutes and then unmutes a track
- **THEN** the track becomes silent and then returns to its previously stored volume

#### Scenario: Pause and resume track
- **WHEN** the user pauses and resumes one track
- **THEN** that track resumes from its paused position while sibling tracks remain unaffected

### Requirement: Profile-wide pause and resume
The application SHALL pause and resume all active-profile tracks as one operation while retaining each track's position and individual enabled state.

#### Scenario: Resume profile
- **WHEN** the user resumes a profile after profile-wide pause
- **THEN** tracks that were individually playing resume from their paused positions and individually paused tracks remain paused

### Requirement: Auto-playing profile switch
The application SHALL stop the old profile and automatically begin the newly selected profile.

#### Scenario: Switch active profile
- **WHEN** the user selects another profile
- **THEN** old-profile tracks stop and enabled tracks in the new profile begin from their starts

#### Scenario: Switch to empty profile
- **WHEN** the user selects a profile with no tracks
- **THEN** old-profile tracks stop and the application remains in a valid playing state with no audio

### Requirement: Recoverable output failure
The application SHALL preserve profile state when the default audio output cannot be opened and SHALL retry opening it on a later play command.

#### Scenario: Output device unavailable
- **WHEN** playback starts while no output device is available
- **THEN** the application reports the failure without losing profiles or track settings

#### Scenario: Retry after output recovery
- **WHEN** an output device becomes available and the user plays again
- **THEN** the application reopens the default output and starts eligible tracks
