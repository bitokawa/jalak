use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window,
    assets::IconName as LucideIcon,
    component::{
        ActiveTheme, Disableable, Icon, IconName, Sizable,
        alert::Alert,
        button::{Button, ButtonVariants},
        h_flex,
        list::{List, ListState},
        menu::ContextMenuExt,
        separator::Separator,
        v_flex,
    },
    div,
    prelude::FluentBuilder,
    px,
};

use crate::{
    app::AppState,
    model::Track,
    platform::macos,
    theme,
    views::{self, profile_list::ProfileList, track_sliders::TrackSliders},
};

const SIDEBAR_WIDTH: f32 = 220.0;

pub struct ManagementView {
    state: Entity<AppState>,
    profiles: Entity<ListState<ProfileList>>,
    /// Tab stop for the profile list; GPUI Kit's list focus handle is not one.
    profiles_tab_stop: FocusHandle,
    sliders: TrackSliders,
    _subscriptions: [Subscription; 3],
}

impl ManagementView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let profiles = cx.new(|cx| ListState::new(ProfileList::new(state.clone()), window, cx));
        let profiles_tab_stop = cx.focus_handle().tab_stop(true);
        let subscriptions = [
            cx.on_focus(&profiles_tab_stop, window, |this, window, cx| {
                this.profiles.update(cx, |list, cx| list.focus(window, cx));
            }),
            cx.observe_in(&state, window, |this, _, window, cx| {
                this.sync_profiles(window, cx);
                this.sliders.sync(&this.state, window, cx);
            }),
            cx.observe_window_appearance(window, |_, window, cx| theme::sync(window, cx)),
        ];
        let mut view = Self {
            state,
            profiles,
            profiles_tab_stop,
            sliders: TrackSliders::default(),
            _subscriptions: subscriptions,
        };
        view.sliders.sync(&view.state, window, cx);
        view.sync_profiles(window, cx);
        view.profiles.update(cx, |list, cx| list.focus(window, cx));
        view
    }

    fn sync_profiles(&self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        let profiles = state.config.profiles.clone();
        let active_id = state.config.active_profile_id;
        self.profiles.update(cx, |list, cx| {
            let selected = list.delegate_mut().sync(&profiles, active_id);
            list.set_selected_index(selected, window, cx);
            cx.notify();
        });
        cx.notify();
    }

    fn render_sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        v_flex()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            .child(
                div()
                    .px_4()
                    .pt_4()
                    .pb_2()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().muted_foreground)
                    .child("Profiles"),
            )
            .child(
                div()
                    .flex_1()
                    .px_2()
                    .track_focus(&self.profiles_tab_stop)
                    .child(List::new(&self.profiles)),
            )
            .child(Separator::horizontal())
            .child(
                h_flex().p_2().child(
                    Button::new("create-profile")
                        .ghost()
                        .small()
                        .icon(IconName::Plus)
                        .label("New Profile")
                        .on_click(move |_, _, cx| {
                            if let Some(name) = macos::prompt_name("New profile", "") {
                                views::run(&state, cx, |state| state.create_profile(&name));
                            }
                        }),
                ),
            )
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let profile = app.active_profile();
        let (profile_id, name) = (profile.id, profile.name.clone());
        let count = profile.tracks.len();
        let can_delete = app.config.profiles.len() > 1;
        let (import_state, rename_state, delete_state) =
            (self.state.clone(), self.state.clone(), self.state.clone());
        let rename_name = name.clone();

        h_flex()
            .justify_between()
            .gap_3()
            .child(
                v_flex()
                    .gap_0p5()
                    .min_w_0()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .truncate()
                            .child(name),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(sound_count(count)),
                    ),
            )
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        views::icon_button(
                            "rename-profile",
                            LucideIcon::Pencil,
                            "Rename Profile",
                            "Rename profile",
                        )
                        .on_click(move |_, _, cx| {
                            views::rename_profile(&rename_state, profile_id, &rename_name, cx)
                        }),
                    )
                    .child(
                        views::icon_button(
                            "delete-profile",
                            LucideIcon::Trash,
                            "Delete Profile",
                            "Delete profile",
                        )
                        .disabled(!can_delete)
                        .on_click(move |_, _, cx| {
                            views::delete_profile(&delete_state, profile_id, cx)
                        }),
                    )
                    .child(
                        Button::new("import-audio")
                            .primary()
                            .small()
                            .ml_2()
                            .icon(IconName::Plus)
                            .label("Import Audio")
                            .on_click(move |_, _, cx| views::import_audio(&import_state, cx)),
                    ),
            )
    }

    fn render_empty(&self, cx: &Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        v_flex()
            .flex_1()
            .min_h(px(240.0))
            .items_center()
            .justify_center()
            .gap_2()
            .child(
                Icon::new(IconName::Inbox)
                    .size_8()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("No sounds yet"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Import MP3, WAV, FLAC, OGG, or M4A files."),
            )
            .child(
                Button::new("empty-import-audio")
                    .primary()
                    .small()
                    .mt_2()
                    .label("Import Audio")
                    .on_click(move |_, _, cx| views::import_audio(&state, cx)),
            )
    }

    fn render_track(&self, track: Track, error: Option<String>, cx: &App) -> impl IntoElement {
        let (track_id, name) = (track.id, track.name.clone());
        let app = self.state.read(cx);
        let path = app.track_path(&track).display().to_string();
        let (play_state, rename_state, delete_state, context_state) = (
            self.state.clone(),
            self.state.clone(),
            self.state.clone(),
            self.state.clone(),
        );
        let (rename_name, context_name) = (name.clone(), name.clone());
        let muted = cx.theme().muted_foreground;
        let play_label = if track.playing { "Pause" } else { "Play" };

        h_flex()
            .gap_3()
            .py_2()
            .child(
                views::icon_button(
                    ("play", track_id),
                    if track.playing {
                        IconName::Pause
                    } else {
                        IconName::Play
                    },
                    play_label,
                    format!("{play_label} {name}"),
                )
                .on_click(move |_, _, cx| {
                    views::run(&play_state, cx, |state| {
                        state.toggle_track_playing(track_id)
                    });
                }),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(
                        // The context menu wraps only the text: GPUI Kit's `ContextMenu`
                        // drops focusable descendants from the Tab order.
                        v_flex()
                            .id(("track", track_id))
                            .min_w_0()
                            .child(div().truncate().child(name))
                            .child(div().text_xs().text_color(muted).truncate().child(path))
                            .context_menu(move |menu, _, _| {
                                views::track_menu(menu, &context_state, track_id, &context_name)
                            }),
                    )
                    .child(self.sliders.render(&track, cx))
                    .when_some(error, |this, error| {
                        this.child(div().text_xs().text_color(cx.theme().danger).child(error))
                    }),
            )
            .child(
                views::icon_button(
                    ("rename-track", track_id),
                    LucideIcon::Pencil,
                    "Rename Sound",
                    format!("Rename {}", track.name),
                )
                .on_click(move |_, _, cx| {
                    views::rename_track(&rename_state, track_id, &rename_name, cx)
                }),
            )
            .child(
                views::icon_button(
                    ("delete-track", track_id),
                    LucideIcon::Trash,
                    "Delete Sound",
                    format!("Delete {}", track.name),
                )
                .on_click(move |_, _, cx| views::delete_track(&delete_state, track_id, cx)),
            )
    }
}

impl Render for ManagementView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let tracks = state.active_profile().tracks.clone();
        let warning = state.warning.clone();
        let error = state.last_error.clone();
        let track_errors = state.track_errors.clone();

        h_flex()
            .size_full()
            .items_start()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(
                v_flex()
                    .id("profile-content")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .px_6()
                    .py_5()
                    .gap_4()
                    .child(self.render_header(cx))
                    .when_some(warning, |this, warning| {
                        this.child(Alert::warning("config-warning", warning))
                    })
                    .when_some(error, |this, error| {
                        this.child(Alert::error("last-error", error))
                    })
                    .child(Separator::horizontal())
                    .when(tracks.is_empty(), |this| this.child(self.render_empty(cx)))
                    .children(tracks.into_iter().enumerate().map(|(index, track)| {
                        let error = track_errors.get(&track.id).cloned();
                        v_flex()
                            .when(index > 0, |this| this.child(Separator::horizontal()))
                            .child(self.render_track(track, error, cx))
                    })),
            )
    }
}

fn sound_count(count: usize) -> String {
    match count {
        1 => "1 sound".to_owned(),
        count => format!("{count} sounds"),
    }
}

#[cfg(test)]
mod tests {
    use super::sound_count;

    #[test]
    fn sound_count_is_pluralized() {
        assert_eq!(sound_count(0), "0 sounds");
        assert_eq!(sound_count(1), "1 sound");
        assert_eq!(sound_count(3), "3 sounds");
    }
}
