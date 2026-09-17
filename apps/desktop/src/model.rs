use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub next_id: u64,
    pub active_profile_id: u64,
    pub profile_paused: bool,
    pub profiles: Vec<Profile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: u64,
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub file_name: String,
    pub byte_size: u64,
    pub volume: f32,
    pub muted: bool,
    pub playing: bool,
}

impl Track {
    /// Playing and audible; what the popup's track toggle shows as on.
    pub fn is_active(&self) -> bool {
        self.playing && !self.muted
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            next_id: 2,
            active_profile_id: 1,
            profile_paused: false,
            profiles: vec![Profile {
                id: 1,
                name: "Default".into(),
                tracks: Vec::new(),
            }],
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.version != CONFIG_VERSION {
            bail!("unsupported configuration version {}", self.version);
        }
        if self.profiles.is_empty() {
            bail!("configuration must contain a profile");
        }
        if !self.profiles.iter().any(|p| p.id == self.active_profile_id) {
            bail!("active profile does not exist");
        }
        for (index, profile) in self.profiles.iter().enumerate() {
            validate_name(&profile.name)?;
            if self.profiles[..index]
                .iter()
                .any(|other| names_equal(&other.name, &profile.name))
            {
                bail!("duplicate profile name: {}", profile.name);
            }
            for track in &profile.tracks {
                validate_name(&track.name)?;
                if !(0.0..=1.0).contains(&track.volume) {
                    bail!("track volume must be between 0 and 1");
                }
            }
        }
        Ok(())
    }

    pub fn active_profile(&self) -> &Profile {
        self.profile(self.active_profile_id)
            .expect("validated config must have active profile")
    }

    pub fn active_profile_mut(&mut self) -> &mut Profile {
        self.profile_mut(self.active_profile_id)
            .expect("validated config must have active profile")
    }

    pub fn profile(&self, id: u64) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn profile_mut(&mut self, id: u64) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|profile| profile.id == id)
    }

    pub fn create_profile(&mut self, name: &str) -> Result<u64> {
        let name = normalized_name(name)?;
        if self
            .profiles
            .iter()
            .any(|profile| names_equal(&profile.name, &name))
        {
            bail!("profile name already exists");
        }
        let id = self.take_id();
        self.profiles.push(Profile {
            id,
            name,
            tracks: Vec::new(),
        });
        self.active_profile_id = id;
        self.profile_paused = false;
        Ok(id)
    }

    pub fn rename_profile(&mut self, id: u64, name: &str) -> Result<()> {
        let name = normalized_name(name)?;
        if self
            .profiles
            .iter()
            .any(|profile| profile.id != id && names_equal(&profile.name, &name))
        {
            bail!("profile name already exists");
        }
        self.profile_mut(id)
            .ok_or_else(|| anyhow::anyhow!("profile not found"))?
            .name = name;
        Ok(())
    }

    pub fn delete_profile(&mut self, id: u64) -> Result<Profile> {
        if self.profiles.len() == 1 {
            bail!("cannot delete final profile");
        }
        let index = self
            .profiles
            .iter()
            .position(|profile| profile.id == id)
            .ok_or_else(|| anyhow::anyhow!("profile not found"))?;
        let removed = self.profiles.remove(index);
        if self.active_profile_id == id {
            self.active_profile_id = self.profiles[0].id;
            self.profile_paused = false;
        }
        Ok(removed)
    }

    pub fn select_profile(&mut self, id: u64) -> Result<()> {
        if self.profile(id).is_none() {
            bail!("profile not found");
        }
        self.active_profile_id = id;
        self.profile_paused = false;
        Ok(())
    }

    pub fn rename_track(&mut self, profile_id: u64, track_id: u64, name: &str) -> Result<()> {
        let name = normalized_name(name)?;
        self.track_mut(profile_id, track_id)?.name = name;
        Ok(())
    }

    pub fn remove_track(&mut self, profile_id: u64, track_id: u64) -> Result<Track> {
        let profile = self
            .profile_mut(profile_id)
            .ok_or_else(|| anyhow::anyhow!("profile not found"))?;
        let index = profile
            .tracks
            .iter()
            .position(|track| track.id == track_id)
            .ok_or_else(|| anyhow::anyhow!("track not found"))?;
        Ok(profile.tracks.remove(index))
    }

    pub fn track_mut(&mut self, profile_id: u64, track_id: u64) -> Result<&mut Track> {
        self.profile_mut(profile_id)
            .ok_or_else(|| anyhow::anyhow!("profile not found"))?
            .tracks
            .iter_mut()
            .find(|track| track.id == track_id)
            .ok_or_else(|| anyhow::anyhow!("track not found"))
    }

    pub fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub fn effective_volume(track: &Track) -> f32 {
    if track.muted { 0.0 } else { track.volume }
}

fn normalized_name(name: &str) -> Result<String> {
    let name = name.trim();
    validate_name(name)?;
    Ok(name.to_owned())
}

fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("name cannot be empty");
    }
    Ok(())
}

fn names_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_lifecycle_enforces_names_and_final_profile() {
        let mut config = Config::default();
        assert!(config.create_profile("  Focus  ").is_ok());
        assert_eq!(config.active_profile().name, "Focus");
        assert!(config.create_profile("focus").is_err());
        assert!(config.rename_profile(1, "Focus").is_err());
        assert!(config.delete_profile(1).is_ok());
        assert!(config.delete_profile(config.active_profile_id).is_err());
    }

    #[test]
    fn track_settings_are_independent() {
        let mut config = Config::default();
        config.active_profile_mut().tracks.extend([
            Track {
                id: 2,
                name: "Rain".into(),
                file_name: "2.wav".into(),
                byte_size: 12,
                volume: 0.4,
                muted: false,
                playing: true,
            },
            Track {
                id: 3,
                name: "Keys".into(),
                file_name: "3.wav".into(),
                byte_size: 12,
                volume: 0.8,
                muted: false,
                playing: true,
            },
        ]);
        let first = config.track_mut(1, 2).unwrap();
        first.volume = 0.2;
        first.muted = true;
        assert_eq!(effective_volume(first), 0.0);
        assert_eq!(config.track_mut(1, 3).unwrap().volume, 0.8);
    }

    #[test]
    fn profile_selection_autoplays() {
        let mut config = Config::default();
        let focus = config.create_profile("Focus").unwrap();
        config.profile_paused = true;
        config.select_profile(1).unwrap();
        assert_eq!(config.active_profile_id, 1);
        assert!(!config.profile_paused);
        assert!(config.select_profile(focus + 99).is_err());
    }
}
