//! Icon assets: GPUI Kit's default component icons plus the few Lucide icons
//! Jalak needs beyond them.

use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString, assets::Assets};

gpui_kit::assets::icon_assets!(ExtraIcons, [AudioLines, Pencil, Trash, Volume2, VolumeX]);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match ExtraIcons.load(path)? {
            Some(bytes) => Ok(Some(bytes)),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use gpui_kit::assets::IconName;

    use super::*;

    #[test]
    fn serves_default_and_extra_icons() {
        for icon in [
            IconName::Play,
            IconName::Ellipsis,
            IconName::Volume2,
            IconName::Trash,
        ] {
            let path = icon.path();
            assert!(AppAssets.load(&path).unwrap().is_some(), "missing {path}");
        }
    }
}
