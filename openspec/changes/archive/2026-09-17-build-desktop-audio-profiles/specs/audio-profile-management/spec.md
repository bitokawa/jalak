## ADDED Requirements

### Requirement: Profile lifecycle
The application SHALL let the user create, rename, delete, and select audio profiles, and SHALL always retain at least one profile.

#### Scenario: Create and select profile
- **WHEN** the user creates a profile with a non-empty unique name
- **THEN** the application adds and selects that profile

#### Scenario: Reject invalid profile name
- **WHEN** the user submits an empty name or a name already used by another profile
- **THEN** the application rejects the change and preserves existing profiles

#### Scenario: Delete final profile
- **WHEN** the user attempts to delete the only remaining profile
- **THEN** the application rejects the deletion

### Requirement: Managed local audio import
The application SHALL validate supported local audio files and copy successful imports into its managed Application Support audio library.

#### Scenario: Import supported audio
- **WHEN** the user selects a decodable MP3, WAV, FLAC, OGG Vorbis, or M4A/AAC file
- **THEN** the application copies it into managed storage and adds a track to the selected profile

#### Scenario: Reject unsupported or corrupt audio
- **WHEN** the selected file cannot be decoded as a supported audio format
- **THEN** the application reports the failure without adding a track or leaving a partial managed file

#### Scenario: Original file disappears
- **WHEN** the original source file is moved or deleted after import
- **THEN** the managed track remains playable

### Requirement: Track management
The application SHALL let the user rename and remove tracks from profiles while keeping each track's playback settings independent.

#### Scenario: Remove managed track
- **WHEN** the user confirms removal of a track
- **THEN** the application removes its profile entry and managed audio file

#### Scenario: Cancel track removal
- **WHEN** the user cancels track removal
- **THEN** the application preserves both track entry and managed audio file

### Requirement: Durable profile configuration
The application SHALL atomically persist configuration version, profiles, managed track references, active-profile selection, volumes, mute states, and track play/pause preferences.

#### Scenario: Relaunch with saved configuration
- **WHEN** the application launches with a valid saved configuration
- **THEN** it restores profiles, tracks, selected profile, and durable control settings

#### Scenario: First launch
- **WHEN** no saved configuration exists
- **THEN** the application creates one empty default profile

#### Scenario: Invalid saved configuration
- **WHEN** saved configuration cannot be parsed or migrated
- **THEN** the application preserves the invalid file and presents a recovery error without silently replacing it
