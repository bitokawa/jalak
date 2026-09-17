use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::{Context, Result, bail};
use rodio::Decoder;

use crate::{
    model::{Config, Track},
    persistence::ConfigStore,
};

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "mp4"];

pub fn import_track(
    store: &ConfigStore,
    config: &mut Config,
    profile_id: u64,
    source: &Path,
) -> Result<u64> {
    validate_audio(source)?;
    if config.profile(profile_id).is_none() {
        bail!("profile not found");
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .context("audio file must have an extension")?;
    let id = config.next_id;
    let file_name = format!("{id}.{extension}");
    let destination = store.paths.audio.join(&file_name);
    let temporary = store.paths.audio.join(format!(".{id}.tmp.{extension}"));

    fs::copy(source, &temporary).with_context(|| format!("failed to copy {}", source.display()))?;
    if let Err(error) = validate_audio(&temporary) {
        let _ = fs::remove_file(&temporary);
        return Err(error.context("managed copy failed validation"));
    }
    fs::rename(&temporary, &destination)?;

    let byte_size = fs::metadata(&destination)?.len();
    let name = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Audio")
        .to_owned();
    config.next_id += 1;
    config
        .profile_mut(profile_id)
        .expect("profile checked above")
        .tracks
        .push(Track {
            id,
            name,
            file_name,
            byte_size,
            volume: 0.5,
            muted: false,
            playing: true,
        });
    Ok(id)
}

pub fn validate_audio(path: &Path) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .context("audio file must have an extension")?;
    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        bail!("unsupported audio format: {extension}");
    }
    Decoder::new(File::open(path)?).context("audio file cannot be decoded")?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn write_test_wav(path: &Path, samples: usize) {
    let data_size = (samples * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8_000u32.to_le_bytes());
    bytes.extend_from_slice(&16_000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    bytes.resize(44 + data_size as usize, 0);
    fs::write(path, bytes).unwrap();
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::paths::AppPaths;

    use super::*;

    fn temp_store() -> ConfigStore {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let paths = AppPaths::under(std::env::temp_dir().join(format!("jalak-import-{nonce}")));
        fs::create_dir_all(&paths.audio).unwrap();
        ConfigStore::new(paths)
    }

    #[test]
    fn import_owns_copy_and_rejects_corrupt_file() {
        let store = temp_store();
        let source = store.paths.root.join("rain.wav");
        write_test_wav(&source, 32);
        let mut config = Config::default();
        let track_id = import_track(&store, &mut config, 1, &source).unwrap();
        fs::remove_file(&source).unwrap();
        let track = &config.active_profile().tracks[0];
        assert_eq!(track.id, track_id);
        assert!(store.paths.audio.join(&track.file_name).exists());

        let corrupt = store.paths.root.join("bad.mp3");
        fs::write(&corrupt, b"not audio").unwrap();
        assert!(import_track(&store, &mut config, 1, &corrupt).is_err());
        assert_eq!(config.active_profile().tracks.len(), 1);
        fs::remove_dir_all(&store.paths.root).unwrap();
    }
}
