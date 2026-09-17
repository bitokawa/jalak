pub mod management;
pub mod popup;
mod profile_list;
mod track_sliders;

use gpui_kit::{
    App, ElementId, Entity, ParentElement, PathPromptOptions, SharedString, Styled,
    component::{
        Icon,
        button::{Button, ButtonRounded, ButtonVariants},
        menu::{PopupMenu, PopupMenuItem},
    },
    px,
};

use crate::{app::AppState, platform::macos};

const ICON_BUTTON_SIZE: f32 = 30.0;
const ICON_GLYPH_SIZE: f32 = 15.0;

/// A filled circular icon button in the style of macOS control buttons.
/// `label` is announced to assistive technology; `tooltip` is shown on hover.
pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    icon: impl Into<Icon>,
    tooltip: impl Into<SharedString>,
    label: impl Into<SharedString>,
) -> Button {
    Button::new(id)
        .secondary()
        .size(px(ICON_BUTTON_SIZE))
        .rounded(ButtonRounded::Size(px(ICON_BUTTON_SIZE / 2.0)))
        .accessibility_label(label)
        .tooltip(tooltip)
        .child(icon.into().size(px(ICON_GLYPH_SIZE)))
}

/// Runs one `AppState` command, surfaces its error, and notifies observers.
pub(crate) fn run(
    state: &Entity<AppState>,
    cx: &mut App,
    command: impl FnOnce(&mut AppState) -> anyhow::Result<()>,
) {
    state.update(cx, |state, context| {
        if let Err(error) = command(state) {
            state.last_error = Some(format!("{error:#}"));
        }
        context.notify();
    });
}

pub(crate) fn import_audio(state: &Entity<AppState>, cx: &mut App) {
    let receiver = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: true,
        prompt: Some("Import".into()),
    });
    let state = state.clone();
    cx.spawn(async move |cx| {
        if let Ok(Ok(Some(paths))) = receiver.await {
            for path in paths {
                cx.update(|cx| run(&state, cx, |state| state.import(&path)));
            }
        }
    })
    .detach();
}

pub(crate) fn rename_profile(state: &Entity<AppState>, profile_id: u64, name: &str, cx: &mut App) {
    if let Some(new_name) = macos::prompt_name("Rename profile", name) {
        run(state, cx, |state| {
            state.rename_profile(profile_id, &new_name)
        });
    }
}

pub(crate) fn delete_profile(state: &Entity<AppState>, profile_id: u64, cx: &mut App) {
    if macos::confirm(
        "Delete profile?",
        "Managed audio used only by this profile will also be deleted.",
    ) {
        run(state, cx, |state| state.delete_profile(profile_id));
    }
}

pub(crate) fn rename_track(state: &Entity<AppState>, track_id: u64, name: &str, cx: &mut App) {
    if let Some(new_name) = macos::prompt_name("Rename sound", name) {
        run(state, cx, |state| state.rename_track(track_id, &new_name));
    }
}

pub(crate) fn delete_track(state: &Entity<AppState>, track_id: u64, cx: &mut App) {
    if macos::confirm("Delete sound?", "This removes Jalak's managed audio copy.") {
        run(state, cx, |state| state.remove_track(track_id, true));
    }
}

/// Right-click menu for a profile row.
pub(crate) fn profile_menu(
    menu: PopupMenu,
    state: &Entity<AppState>,
    profile_id: u64,
    name: &str,
    can_delete: bool,
) -> PopupMenu {
    let (rename_state, delete_state) = (state.clone(), state.clone());
    let name = name.to_owned();
    menu.item(
        PopupMenuItem::new("Rename Profile")
            .on_click(move |_, _, cx| rename_profile(&rename_state, profile_id, &name, cx)),
    )
    .separator()
    .item(
        PopupMenuItem::new("Delete Profile")
            .disabled(!can_delete)
            .on_click(move |_, _, cx| delete_profile(&delete_state, profile_id, cx)),
    )
}

/// Right-click menu for a track row of the active profile.
pub(crate) fn track_menu(
    menu: PopupMenu,
    state: &Entity<AppState>,
    track_id: u64,
    name: &str,
) -> PopupMenu {
    let (rename_state, delete_state) = (state.clone(), state.clone());
    let name = name.to_owned();
    menu.item(
        PopupMenuItem::new("Rename Sound")
            .on_click(move |_, _, cx| rename_track(&rename_state, track_id, &name, cx)),
    )
    .separator()
    .item(
        PopupMenuItem::new("Delete Sound")
            .on_click(move |_, _, cx| delete_track(&delete_state, track_id, cx)),
    )
}
