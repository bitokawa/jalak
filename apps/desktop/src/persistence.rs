use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{model::Config, paths::AppPaths};

#[derive(Debug)]
pub struct LoadResult {
    pub config: Config,
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ConfigStore {
    pub paths: AppPaths,
}

impl ConfigStore {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub fn load(&self) -> Result<LoadResult> {
        self.ensure_directories()?;
        if !self.paths.config.exists() {
            let config = Config::default();
            self.save(&config)?;
            return Ok(LoadResult {
                config,
                warning: None,
            });
        }

        match self.read_config() {
            Ok(config) => Ok(LoadResult {
                config,
                warning: None,
            }),
            Err(error) => {
                let preserved = self.next_invalid_path();
                fs::rename(&self.paths.config, &preserved).with_context(|| {
                    format!(
                        "failed to preserve invalid config at {}",
                        preserved.display()
                    )
                })?;
                let config = Config::default();
                self.save(&config)?;
                Ok(LoadResult {
                    config,
                    warning: Some(format!(
                        "Invalid configuration was preserved at {}: {error:#}",
                        preserved.display()
                    )),
                })
            }
        }
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        config.validate()?;
        self.ensure_directories()?;
        let temporary = sibling(&self.paths.config, "config.json.tmp");
        let file = File::create(&temporary)
            .with_context(|| format!("failed to create {}", temporary.display()))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, config)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        fs::rename(&temporary, &self.paths.config).with_context(|| {
            format!(
                "failed to replace configuration {}",
                self.paths.config.display()
            )
        })?;
        Ok(())
    }

    fn read_config(&self) -> Result<Config> {
        let file = File::open(&self.paths.config)?;
        let config: Config = serde_json::from_reader(BufReader::new(file))?;
        config.validate()?;
        Ok(config)
    }

    fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.paths.audio)
            .with_context(|| format!("failed to create {}", self.paths.audio.display()))
    }

    fn next_invalid_path(&self) -> PathBuf {
        for suffix in 0.. {
            let name = if suffix == 0 {
                "config.invalid.json".to_owned()
            } else {
                format!("config.invalid.{suffix}.json")
            };
            let path = self.paths.root.join(name);
            if !path.exists() {
                return path;
            }
        }
        unreachable!()
    }
}

fn sibling(path: &Path, name: &str) -> PathBuf {
    path.parent().unwrap_or_else(|| Path::new(".")).join(name)
}

pub fn remove_managed_file(audio_dir: &Path, file_name: &str) -> Result<()> {
    let path = audio_dir.join(file_name);
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("failed to remove managed audio {}", path.display()))?;
    }
    Ok(())
}

pub fn ensure_managed_name(file_name: &str) -> Result<()> {
    if Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(file_name)
    {
        bail!("invalid managed filename");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_paths(label: &str) -> AppPaths {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        AppPaths::under(std::env::temp_dir().join(format!("jalak-{label}-{nonce}")))
    }

    #[test]
    fn missing_config_creates_default_and_round_trips() {
        let paths = temp_paths("roundtrip");
        let store = ConfigStore::new(paths.clone());
        let mut loaded = store.load().unwrap();
        assert_eq!(loaded.config, Config::default());
        loaded.config.rename_profile(1, "Work").unwrap();
        store.save(&loaded.config).unwrap();
        assert_eq!(store.load().unwrap().config.active_profile().name, "Work");
        assert!(!paths.config.with_file_name("config.json.tmp").exists());
        fs::remove_dir_all(paths.root).unwrap();
    }

    #[test]
    fn invalid_config_is_preserved_before_recovery() {
        let paths = temp_paths("invalid");
        fs::create_dir_all(&paths.root).unwrap();
        fs::write(&paths.config, b"not json").unwrap();
        let loaded = ConfigStore::new(paths.clone()).load().unwrap();
        assert!(loaded.warning.is_some());
        assert_eq!(
            fs::read(paths.root.join("config.invalid.json")).unwrap(),
            b"not json"
        );
        assert_eq!(loaded.config, Config::default());
        fs::remove_dir_all(paths.root).unwrap();
    }
}
