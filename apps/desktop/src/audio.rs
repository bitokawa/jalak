use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::model::{Profile, effective_volume};

#[derive(Default)]
pub struct AudioEngine {
    output: Option<MixerDeviceSink>,
    players: HashMap<u64, Player>,
}

impl AudioEngine {
    pub fn sync_profile(
        &mut self,
        profile: &Profile,
        audio_dir: &Path,
        profile_paused: bool,
    ) -> Result<HashMap<u64, String>> {
        self.players.clear();
        if profile.tracks.is_empty() {
            return Ok(HashMap::new());
        }
        if self.output.is_none() {
            self.output = Some(
                DeviceSinkBuilder::open_default_sink()
                    .context("default audio output is unavailable")?,
            );
        }
        let mixer = self.output.as_ref().expect("opened above").mixer();
        let mut errors = HashMap::new();
        for track in &profile.tracks {
            let result = (|| -> Result<Player> {
                let file = File::open(audio_dir.join(&track.file_name))?;
                let decoder = Decoder::new_looped(BufReader::new(file))?;
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

    pub fn retry_output(&mut self) {
        self.players.clear();
        self.output = None;
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
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
        engine.retry_output();
        assert_eq!(engine.player_count(), 0);
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
        engine.retry_output();
        assert_eq!(engine.player_count(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
