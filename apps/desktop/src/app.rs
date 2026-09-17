use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    audio::AudioEngine,
    imports::import_track,
    model::{Config, Profile, Track, effective_volume},
    paths::AppPaths,
    persistence::{ConfigStore, remove_managed_file},
};

pub struct AppState {
    pub config: Config,
    pub warning: Option<String>,
    pub last_error: Option<String>,
    pub track_errors: HashMap<u64, String>,
    store: ConfigStore,
    audio: AudioEngine,
}

impl AppState {
    pub fn load() -> Result<Self> {
        let store = ConfigStore::new(AppPaths::discover()?);
        let loaded = store.load()?;
        let mut state = Self {
            config: loaded.config,
            warning: loaded.warning,
            last_error: None,
            track_errors: HashMap::new(),
            store,
            audio: AudioEngine::default(),
        };
        state.sync_active();
        Ok(state)
    }

    #[cfg(test)]
    pub fn with_store(store: ConfigStore) -> Result<Self> {
        let loaded = store.load()?;
        Ok(Self {
            config: loaded.config,
            warning: loaded.warning,
            last_error: None,
            track_errors: HashMap::new(),
            store,
            audio: AudioEngine::default(),
        })
    }

    pub fn create_profile(&mut self, name: &str) -> Result<()> {
        self.update_config(|config| config.create_profile(name).map(|_| ()))?;
        self.sync_active();
        Ok(())
    }

    pub fn rename_profile(&mut self, id: u64, name: &str) -> Result<()> {
        self.update_config(|config| config.rename_profile(id, name))
    }

    pub fn delete_profile(&mut self, id: u64) -> Result<()> {
        let mut candidate = self.config.clone();
        let removed = candidate.delete_profile(id)?;
        self.store.save(&candidate)?;
        self.config = candidate;
        self.sync_active();
        for track in removed.tracks {
            if let Err(error) = remove_managed_file(&self.store.paths.audio, &track.file_name) {
                self.last_error = Some(format!("{error:#}"));
            }
        }
        Ok(())
    }

    pub fn select_profile(&mut self, id: u64) -> Result<()> {
        self.update_config(|config| config.select_profile(id))?;
        self.sync_active();
        Ok(())
    }

    pub fn import(&mut self, source: &Path) -> Result<()> {
        let mut candidate = self.config.clone();
        let profile_id = candidate.active_profile_id;
        let track_id = import_track(&self.store, &mut candidate, profile_id, source)?;
        if let Err(error) = self.store.save(&candidate) {
            let track = candidate.remove_track(profile_id, track_id)?;
            let _ = remove_managed_file(&self.store.paths.audio, &track.file_name);
            return Err(error);
        }
        self.config = candidate;
        self.sync_active();
        Ok(())
    }

    pub fn rename_track(&mut self, track_id: u64, name: &str) -> Result<()> {
        let profile_id = self.config.active_profile_id;
        self.update_config(|config| config.rename_track(profile_id, track_id, name))
    }

    pub fn remove_track(&mut self, track_id: u64, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Ok(());
        }
        let profile_id = self.config.active_profile_id;
        let mut candidate = self.config.clone();
        let removed = candidate.remove_track(profile_id, track_id)?;
        self.store.save(&candidate)?;
        self.config = candidate;
        self.sync_active();
        remove_managed_file(&self.store.paths.audio, &removed.file_name)
    }

    pub fn set_track_volume(&mut self, track_id: u64, volume: f32) -> Result<()> {
        let volume = volume.clamp(0.0, 1.0);
        let profile_id = self.config.active_profile_id;
        self.update_config(|config| {
            config.track_mut(profile_id, track_id)?.volume = volume;
            Ok(())
        })?;
        let track = self
            .config
            .active_profile()
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .context("track not found after save")?;
        self.audio.set_volume(track_id, effective_volume(track));
        Ok(())
    }

    pub fn toggle_track_muted(&mut self, track_id: u64) -> Result<()> {
        let profile_id = self.config.active_profile_id;
        self.update_config(|config| {
            let track = config.track_mut(profile_id, track_id)?;
            track.muted = !track.muted;
            Ok(())
        })?;
        let track = self
            .config
            .active_profile()
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .context("track not found after save")?;
        self.audio.set_volume(track_id, effective_volume(track));
        Ok(())
    }

    pub fn toggle_track_playing(&mut self, track_id: u64) -> Result<()> {
        let profile_id = self.config.active_profile_id;
        self.update_config(|config| {
            let track = config.track_mut(profile_id, track_id)?;
            track.playing = !track.playing;
            Ok(())
        })?;
        let track = self
            .config
            .active_profile()
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .context("track not found after save")?;
        self.audio
            .set_playing(track_id, track.playing, self.config.profile_paused);
        Ok(())
    }

    /// Turns a track on (playing and audible) or off (paused).
    pub fn set_track_active(&mut self, track_id: u64, active: bool) -> Result<()> {
        let profile_id = self.config.active_profile_id;
        self.update_config(|config| {
            let track = config.track_mut(profile_id, track_id)?;
            track.playing = active;
            if active {
                track.muted = false;
            }
            Ok(())
        })?;
        let track = self
            .config
            .active_profile()
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .context("track not found after save")?;
        self.audio.set_volume(track_id, effective_volume(track));
        self.audio
            .set_playing(track_id, track.playing, self.config.profile_paused);
        Ok(())
    }

    pub fn toggle_profile_paused(&mut self) -> Result<()> {
        self.update_config(|config| {
            config.profile_paused = !config.profile_paused;
            Ok(())
        })?;
        self.audio
            .set_profile_paused(self.config.active_profile(), self.config.profile_paused);
        Ok(())
    }

    pub fn retry_audio(&mut self) {
        self.audio.retry_output();
        self.sync_active();
    }

    pub fn active_profile(&self) -> &Profile {
        self.config.active_profile()
    }

    /// Location of a track's managed audio copy.
    pub fn track_path(&self, track: &Track) -> PathBuf {
        self.store.paths.audio.join(&track.file_name)
    }

    pub fn player_count(&self) -> usize {
        self.audio.player_count()
    }

    fn update_config(&mut self, update: impl FnOnce(&mut Config) -> Result<()>) -> Result<()> {
        let mut candidate = self.config.clone();
        update(&mut candidate)?;
        self.store.save(&candidate)?;
        self.config = candidate;
        self.last_error = None;
        Ok(())
    }

    fn sync_active(&mut self) {
        match self.audio.sync_profile(
            self.config.active_profile(),
            &self.store.paths.audio,
            self.config.profile_paused,
        ) {
            Ok(errors) => {
                self.track_errors = errors;
                self.last_error = None;
            }
            Err(error) => {
                self.track_errors.clear();
                self.last_error = Some(format!("{error:#}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{imports::write_test_wav, paths::AppPaths};

    use super::*;

    fn state() -> AppState {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let paths =
            AppPaths::under(std::env::temp_dir().join(format!("jalak-state-{nonce}-{serial}")));
        AppState::with_store(ConfigStore::new(paths)).unwrap()
    }

    #[test]
    fn failed_command_keeps_last_saved_state() {
        let mut state = state();
        state.create_profile("Focus").unwrap();
        let snapshot = state.config.clone();
        assert!(state.create_profile("focus").is_err());
        assert_eq!(state.config, snapshot);
        fs::remove_dir_all(&state.store.paths.root).unwrap();
    }

    #[test]
    fn import_remove_and_cancel_are_transactional() {
        let mut state = state();
        let source = state.store.paths.root.join("rain.wav");
        write_test_wav(&source, 64);
        state.import(&source).unwrap();
        let track = state.active_profile().tracks[0].clone();
        assert!(state.track_path(&track).exists());
        assert!(
            state
                .track_path(&track)
                .starts_with(&state.store.paths.audio)
        );
        state.remove_track(track.id, false).unwrap();
        assert!(state.store.paths.audio.join(&track.file_name).exists());
        state.remove_track(track.id, true).unwrap();
        assert!(!state.store.paths.audio.join(&track.file_name).exists());
        fs::remove_dir_all(&state.store.paths.root).unwrap();
    }

    #[test]
    fn activating_a_track_plays_and_unmutes_it() {
        let mut state = state();
        let source = state.store.paths.root.join("rain.wav");
        write_test_wav(&source, 64);
        state.import(&source).unwrap();
        let track_id = state.active_profile().tracks[0].id;
        state.toggle_track_muted(track_id).unwrap();
        state.set_track_active(track_id, false).unwrap();
        let track = &state.active_profile().tracks[0];
        assert!(!track.playing && track.muted);

        state.set_track_active(track_id, true).unwrap();
        let track = &state.active_profile().tracks[0];
        assert!(track.playing && !track.muted);
        assert!(track.is_active());

        state.set_track_active(track_id, false).unwrap();
        let track = &state.active_profile().tracks[0];
        assert!(!track.playing && !track.is_active());
        fs::remove_dir_all(&state.store.paths.root).unwrap();
    }

    #[test]
    fn commands_persist_one_consistent_snapshot() {
        let mut state = state();
        state.create_profile("Focus").unwrap();
        let profile_id = state
            .config
            .profiles
            .iter()
            .find(|profile| profile.name == "Focus")
            .unwrap()
            .id;
        state.select_profile(profile_id).unwrap();

        let source = state.store.paths.root.join("rain.wav");
        write_test_wav(&source, 64);
        state.import(&source).unwrap();
        let track_id = state.active_profile().tracks[0].id;
        state.rename_track(track_id, "Rain").unwrap();
        state.set_track_volume(track_id, 0.25).unwrap();
        state.toggle_track_muted(track_id).unwrap();
        state.toggle_track_playing(track_id).unwrap();
        state.toggle_profile_paused().unwrap();

        let root = state.store.paths.root.clone();
        let restored = AppState::with_store(ConfigStore::new(state.store.paths.clone())).unwrap();
        let track = &restored.active_profile().tracks[0];
        assert_eq!(restored.active_profile().name, "Focus");
        assert_eq!(track.name, "Rain");
        assert_eq!(track.volume, 0.25);
        assert!(track.muted);
        assert!(!track.playing);
        assert!(restored.config.profile_paused);
        fs::remove_dir_all(root).unwrap();
    }
}
