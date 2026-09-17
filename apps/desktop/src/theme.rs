//! Jalak's semantic macOS appearance, expressed through GPUI Kit theme tokens.
//!
//! Views read roles straight from `cx.theme()`:
//!
//! | Role                   | Token                                   |
//! | ---------------------- | --------------------------------------- |
//! | surface                | `background`, `sidebar`, `popover`      |
//! | text                   | `foreground`                            |
//! | muted                  | `muted_foreground`                      |
//! | accent                 | `primary`, `slider_bar`                 |
//! | selection              | `list_active`, `selection`              |
//! | focus                  | `ring`                                  |
//! | warning                | `warning`                               |
//! | error and destructive  | `danger`                                |

use gpui_kit::{
    App, Hsla, Window, WindowAppearance,
    component::{Theme, ThemeColor, ThemeMode, ThemeTokens},
    px, rgb,
};

/// macOS body text size.
const FONT_SIZE: f32 = 13.0;
/// macOS control corner radius.
const RADIUS: f32 = 6.0;

struct Palette {
    accent: u32,
    surface: u32,
    sidebar: u32,
    text: u32,
    muted: u32,
    separator: u32,
    fill: u32,
    switch_off: u32,
    switch_thumb: u32,
    warning: u32,
    danger: u32,
}

/// Approximations of AppKit's system colors; GPUI does not expose `NSColor`.
const LIGHT: Palette = Palette {
    accent: 0x007aff,
    surface: 0xffffff,
    sidebar: 0xf0f0f2,
    text: 0x1d1d1f,
    muted: 0x6e6e73,
    separator: 0xd8d8dc,
    fill: 0xe8e8ed,
    switch_off: 0xe5e5ea,
    switch_thumb: 0xffffff,
    warning: 0xc76a00,
    danger: 0xd70015,
};

const DARK: Palette = Palette {
    accent: 0x0a84ff,
    surface: 0x1e1e1e,
    sidebar: 0x2a2a2c,
    text: 0xf5f5f7,
    muted: 0x98989d,
    separator: 0x3a3a3c,
    fill: 0x3a3a3c,
    switch_off: 0x5a5a5e,
    switch_thumb: 0xf2f2f7,
    warning: 0xff9f0a,
    danger: 0xff453a,
};

/// Applies the current system appearance before any window renders.
pub fn init(cx: &mut App) {
    apply(cx.window_appearance(), cx);
}

/// Re-applies the theme after macOS changes appearance under `window`.
pub fn sync(window: &mut Window, cx: &mut App) {
    apply(window.appearance(), cx);
    window.refresh();
}

fn apply(appearance: WindowAppearance, cx: &mut App) {
    let mode = ThemeMode::from(appearance);
    Theme::change(mode, None, cx);
    let theme = Theme::global_mut(cx);
    tune(&mut theme.colors, mode.is_dark());
    // Buttons and sidebars read these legacy tokens, which `Theme::change`
    // resolved from the stock palette; rebuild them from the tuned colors.
    theme.tokens = ThemeTokens::from(&theme.colors);
    theme.font_size = px(FONT_SIZE);
    theme.radius = px(RADIUS);
    Theme::sync_base(cx);
}

fn tune(colors: &mut ThemeColor, dark: bool) {
    let palette = if dark { &DARK } else { &LIGHT };
    let accent = color(palette.accent);
    let on_accent = color(0xffffff);
    let surface = color(palette.surface);
    let text = color(palette.text);
    let separator = color(palette.separator);

    colors.background = surface;
    colors.popover = surface;
    colors.list = surface;
    colors.foreground = text;
    colors.popover_foreground = text;
    colors.muted_foreground = color(palette.muted);
    colors.border = separator;

    colors.sidebar = color(palette.sidebar);
    colors.sidebar_foreground = text;
    colors.sidebar_border = separator;

    colors.primary = accent;
    colors.primary_hover = with_alpha(accent, 0.9);
    colors.primary_active = with_alpha(accent, 0.8);
    colors.primary_foreground = on_accent;
    colors.button_primary = accent;
    colors.button_primary_hover = with_alpha(accent, 0.9);
    colors.button_primary_active = with_alpha(accent, 0.8);
    colors.button_primary_foreground = on_accent;

    let fill = color(palette.fill);
    colors.secondary = fill;
    colors.secondary_hover = with_alpha(fill, 0.8);
    colors.secondary_active = with_alpha(fill, 0.6);
    colors.secondary_foreground = text;
    colors.button_secondary = fill;
    colors.button_secondary_hover = with_alpha(fill, 0.8);
    colors.button_secondary_active = with_alpha(fill, 0.6);
    colors.button_secondary_foreground = text;

    colors.switch = color(palette.switch_off);
    colors.switch_thumb = color(palette.switch_thumb);

    colors.ring = accent;
    colors.selection = with_alpha(accent, 0.35);
    colors.list_active = with_alpha(accent, 0.22);
    colors.list_active_border = accent;
    colors.slider_bar = accent;
    colors.slider_thumb = on_accent;

    colors.warning = color(palette.warning);
    colors.danger = color(palette.danger);
    colors.danger_foreground = on_accent;
    colors.button_danger = color(palette.danger);
    colors.button_danger_foreground = on_accent;
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

fn with_alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuned(dark: bool) -> ThemeColor {
        let mut colors = ThemeColor::default();
        tune(&mut colors, dark);
        colors
    }

    #[test]
    fn appearances_use_distinct_surfaces_and_system_accent() {
        let light = tuned(false);
        let dark = tuned(true);
        assert_ne!(light.background, dark.background);
        assert_ne!(light.foreground, dark.foreground);
        assert_eq!(light.primary, color(LIGHT.accent));
        assert_eq!(dark.primary, color(DARK.accent));
        assert_eq!(dark.ring, dark.primary);
        assert_eq!(dark.danger, color(DARK.danger));
    }

    #[test]
    fn primary_button_tokens_follow_the_tuned_palette() {
        for dark in [false, true] {
            let colors = tuned(dark);
            let tokens = ThemeTokens::from(&colors);
            assert_eq!(tokens.button_primary.color, colors.primary);
            assert_ne!(
                tokens.button_primary.color,
                colors.button_primary_foreground
            );
        }
    }

    #[test]
    fn switch_uses_accent_track_and_white_thumb_in_both_appearances() {
        for dark in [false, true] {
            let colors = tuned(dark);
            let tokens = ThemeTokens::from(&colors);
            assert_eq!(tokens.primary.color, colors.primary);
            assert!(colors.switch_thumb.l > 0.9);
            assert_ne!(colors.switch, colors.switch_thumb);
        }
    }

    #[test]
    fn light_surfaces_are_light_and_dark_surfaces_are_dark() {
        assert!(tuned(false).background.l > 0.9);
        assert!(tuned(true).background.l < 0.2);
        assert!(tuned(false).foreground.l < 0.2);
        assert!(tuned(true).foreground.l > 0.9);
    }
}
