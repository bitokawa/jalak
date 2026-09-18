use std::{
    collections::HashMap,
    fs::File,
    hash::{BuildHasher, RandomState},
    io::BufReader,
    path::Path,
    time::Duration,
};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

use crate::model::{Profile, effective_volume};

/// Keeps a random start from landing on the very end of a track.
const START_HEADROOM: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct AudioEngine {
    output: Option<MixerDeviceSink>,
    players: HashMap<u64, Player>,
}

impl AudioEngine {
    /// Rebuilds the players for `profile`, reusing the current output device.
    pub fn sync_profile(
        &mut self,
        profile: &Profile,
        audio_dir: &Path,
        profile_paused: bool,
    ) -> Result<HashMap<u64, String>> {
        self.build(profile, audio_dir, profile_paused, false)
    }

    /// Reopens the output device and rebuilds every player from a random
    /// position.
    ///
    /// macOS tears down the output stream on sleep or a device change, and
    /// neither rodio nor cpal reconnects. The old sink then accepts players
    /// forever while producing silence, so resuming a profile always starts
    /// from a fresh device instead of trusting the one we hold.
    pub fn restart_profile(
        &mut self,
        profile: &Profile,
        audio_dir: &Path,
        profile_paused: bool,
    ) -> Result<HashMap<u64, String>> {
        self.output = None;
        self.build(profile, audio_dir, profile_paused, true)
    }

    fn build(
        &mut self,
        profile: &Profile,
        audio_dir: &Path,
        profile_paused: bool,
        randomize_start: bool,
    ) -> Result<HashMap<u64, String>> {
        self.players.clear();
        if profile.tracks.is_empty() {
            return Ok(HashMap::new());
        }
        if self.output.is_none() {
            let mut sink = DeviceSinkBuilder::open_default_sink()
                .context("default audio output is unavailable")?;
            // Resuming drops the previous sink on purpose; rodio's warning
            // about that is noise here.
            sink.log_on_drop(false);
            self.output = Some(sink);
        }
        let mixer = self.output.as_ref().expect("opened above").mixer();
        let mut errors = HashMap::new();
        for track in &profile.tracks {
            let result = (|| -> Result<Player> {
                let path = audio_dir.join(&track.file_name);
                let mut decoder = Decoder::new_looped(BufReader::new(File::open(&path)?))?;
                if randomize_start && let Some(offset) = random_start(&path) {
                    // A track that cannot seek just starts from the beginning.
                    let _ = decoder.try_seek(offset);
                }
                let player = Player::connect_new(mixer);
                player.set_volume(effective_volume(track));
                player.append(decoder);
                if profile_paused || !track.playing {
                    player.pause();
                }
                Ok(player)
            })();
            match result {
                Ok(player) => {
                    self.players.insert(track.id, player);
                }
                Err(error) => {
                    errors.insert(track.id, format!("{error:#}"));
                }
            }
        }
        Ok(errors)
    }

    pub fn set_volume(&self, track_id: u64, volume: f32) {
        if let Some(player) = self.players.get(&track_id) {
            player.set_volume(volume);
        }
    }

    pub fn set_playing(&self, track_id: u64, playing: bool, profile_paused: bool) {
        if let Some(player) = self.players.get(&track_id) {
            if playing && !profile_paused {
                player.play();
            } else {
                player.pause();
            }
        }
    }

    pub fn set_profile_paused(&self, profile: &Profile, paused: bool) {
        for track in &profile.tracks {
            self.set_playing(track.id, track.playing, paused);
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

/// A random position inside the track, so resuming a profile does not always
/// replay the same passage. `None` when the duration is unknown.
fn random_start(path: &Path) -> Option<Duration> {
    let total = Decoder::new(BufReader::new(File::open(path).ok()?))
        .ok()?
        .total_duration()?;
    let span = total.checked_sub(START_HEADROOM)?;
    Some(span.mul_f64(random_fraction()))
}

/// A value in `[0, 1)`. `RandomState` seeds every instance differently, which
/// is randomness enough for picking a starting point.
fn random_fraction() -> f64 {
    RandomState::new().hash_one(()) as f64 / (u64::MAX as f64 + 1.0)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, Read, Seek, SeekFrom},
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use crate::{imports::write_test_wav, model::Track};

    use super::*;

    struct CountingReader<R> {
        inner: R,
        bytes_read: Arc<AtomicUsize>,
    }

    impl<R: Read> Read for CountingReader<R> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let count = self.inner.read(buffer)?;
            self.bytes_read.fetch_add(count, Ordering::Relaxed);
            Ok(count)
        }
    }

    impl<R: Seek> Seek for CountingReader<R> {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(position)
        }
    }

    #[test]
    fn looped_decoder_streams_instead_of_buffering_whole_file() {
        let mut bytes = Vec::new();
        let temp = std::env::temp_dir().join(format!("jalak-stream-{}.wav", std::process::id()));
        write_test_wav(&temp, 1_000_000);
        File::open(&temp).unwrap().read_to_end(&mut bytes).unwrap();
        std::fs::remove_file(temp).unwrap();

        let bytes_read = Arc::new(AtomicUsize::new(0));
        let reader = CountingReader {
            inner: Cursor::new(bytes.clone()),
            bytes_read: bytes_read.clone(),
        };
        let mut decoder = Decoder::new_looped(reader).unwrap();
        assert_eq!(decoder.size_hint().1, None);
        for _ in 0..128 {
            let _ = decoder.next();
        }
        assert!(bytes_read.load(Ordering::Relaxed) < bytes.len() / 2);
    }

    #[test]
    fn player_controls_are_independent_and_preserve_mute_volume() {
        let (first, _first_output) = Player::new();
        let (second, _second_output) = Player::new();
        let mut engine = AudioEngine::default();
        engine.players.insert(1, first);
        engine.players.insert(2, second);

        engine.set_volume(1, 0.25);
        assert_eq!(engine.players[&1].volume(), 0.25);
        assert_eq!(engine.players[&2].volume(), 1.0);
        engine.set_playing(1, false, false);
        assert!(engine.players[&1].is_paused());
        assert!(!engine.players[&2].is_paused());

        let track = Track {
            id: 1,
            name: "Rain".into(),
            file_name: "1.wav".into(),
            byte_size: 1,
            volume: 0.25,
            muted: true,
            playing: true,
        };
        engine.set_volume(1, effective_volume(&track));
        assert_eq!(engine.players[&1].volume(), 0.0);
        assert_eq!(track.volume, 0.25);
    }

    #[test]
    fn empty_profile_switch_stops_previous_players_without_device() {
        let (player, _output) = Player::new();
        let mut engine = AudioEngine::default();
        engine.players.insert(7, player);
        let profile = Profile {
            id: 2,
            name: "Empty".into(),
            tracks: Vec::new(),
        };
        let errors = engine
            .sync_profile(&profile, Path::new("."), false)
            .unwrap();
        assert!(errors.is_empty());
        assert_eq!(engine.player_count(), 0);
        engine
            .restart_profile(&profile, Path::new("."), false)
            .unwrap();
        assert_eq!(engine.player_count(), 0);
    }

    #[test]
    fn random_start_stays_inside_the_track_and_moves_around() {
        let path = std::env::temp_dir().join(format!("jalak-start-{}.wav", std::process::id()));
        write_test_wav(&path, 8_000 * 60);
        let starts: Vec<Duration> = (0..16).filter_map(|_| random_start(&path)).collect();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(starts.len(), 16);
        let limit = Duration::from_secs(60) - START_HEADROOM;
        assert!(starts.iter().all(|start| *start < limit));
        assert!(starts.iter().any(|start| *start != starts[0]));
    }

    #[test]
    fn random_start_is_none_for_a_track_shorter_than_the_headroom() {
        let path = std::env::temp_dir().join(format!("jalak-short-{}.wav", std::process::id()));
        write_test_wav(&path, 8_000);
        let start = random_start(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(start.is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn active_profile_keeps_valid_players_when_sibling_track_fails() {
        let root = std::env::temp_dir().join(format!("jalak-audio-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        write_test_wav(&root.join("1.wav"), 8_000);
        let track = |id, file_name: &str| Track {
            id,
            name: format!("Track {id}"),
            file_name: file_name.into(),
            byte_size: 16_044,
            volume: 0.5,
            muted: false,
            playing: true,
        };
        let profile = Profile {
            id: 1,
            name: "Focus".into(),
            tracks: vec![track(1, "1.wav"), track(2, "missing.wav")],
        };
        let mut engine = AudioEngine::default();
        let errors = engine.sync_profile(&profile, &root, false).unwrap();
        assert_eq!(engine.player_count(), 1);
        assert!(errors.contains_key(&2));

        // Resuming reopens the device, which is how playback recovers after
        // macOS tore the previous output down.
        let errors = engine.restart_profile(&profile, &root, false).unwrap();
        assert_eq!(engine.player_count(), 1);
        assert!(errors.contains_key(&2));
        std::fs::remove_dir_all(root).unwrap();
    }
}
