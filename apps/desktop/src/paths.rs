use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::BaseDirs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPaths {
    pub root: PathBuf,
    pub config: PathBuf,
    pub audio: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let data_dir = BaseDirs::new()
            .context("could not locate user data directory")?
            .data_dir()
            .to_path_buf();
        Ok(Self::under(data_dir.join("Jalak")))
    }

    pub fn under(root: PathBuf) -> Self {
        Self {
            config: root.join("config.json"),
            audio: root.join("audio"),
            root,
        }
    }
}
