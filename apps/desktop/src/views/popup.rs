use gpui_kit::{
    Context, Entity, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window,
    assets::IconName as LucideIcon,
    component::{
        ActiveTheme, Sizable,
        alert::Alert,
        button::{Button, ButtonVariants},
        h_flex,
        menu::{DropdownMenu, PopupMenuItem},
        separator::Separator,
        switch::Switch,
        v_flex,
    },
    div,
    prelude::FluentBuilder,
    px, size,
};

use crate::{
    app::AppState,
    model::Track,
    platform::macos,
    theme,
    views::{self, track_sliders::TrackSliders},
};

pub(crate) const POPUP_WIDTH: f32 = 332.0;
const MAX_HEIGHT: f32 = 480.0;
const PADDING: f32 = 8.0;
const BORDER: f32 = 1.0;
const GAP: f32 = 4.0;
const HEADER_HEIGHT: f32 = 32.0;
const TITLE_HEIGHT: f32 = 20.0;
const ROW_HEIGHT: f32 = 48.0;
const SEPARATOR_HEIGHT: f32 = 9.0;
const FOOTER_HEIGHT: f32 = 24.0;

pub struct PopupView {
    state: Entity<AppState>,
    sliders: TrackSliders,
    _subscriptions: [Subscription; 3],
}

impl PopupView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.observe_in(&state, window, |this, _, window, cx| {
                this.sliders.sync(&this.state, window, cx);
                let height = preferred_height(this.state.read(cx).active_profile().tracks.len());
                if window.bounds().size.height != px(height) {
                    window.resize(size(px(POPUP_WIDTH), px(height)));
                }
                cx.notify();
            }),
            cx.observe_window_activation(window, |_, window, cx| {
                if !window.is_window_active() {
                    cx.defer(|cx| {
                        macos::dismiss_popup(cx);
                    });
                }
            }),
            cx.observe_window_appearance(window, |_, window, cx| theme::sync(window, cx)),
        ];
        let mut view = Self {
            state,
            sliders: TrackSliders::default(),
            _subscriptions: subscriptions,
        };
        view.sliders.sync(&view.state, window, cx);
        view
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let name = app.active_profile().name.clone();
        let playing = !app.config.profile_paused;
        let menu_state = self.state.clone();
        let pause_state = self.state.clone();

        h_flex()
            .h(px(HEADER_HEIGHT))
            .justify_between()
            .gap_2()
            .child(
                Button::new("profile-menu")
                    .ghost()
                    .label(name)
                    .dropdown_caret(true)
                    .accessibility_label("Switch profile")
                    .tooltip("Switch Profile")
                    .dropdown_menu(move |menu, _, cx| {
                        let app = menu_state.read(cx);
                        let active_id = app.config.active_profile_id;
                        let profiles = app.config.profiles.clone();
                        profiles.into_iter().fold(
                            menu.scrollable(true).max_h(px(260.0)).min_w(px(180.0)),
                            |menu, profile| {
                                let state = menu_state.clone();
                                let profile_id = profile.id;
                                menu.item(
                                    PopupMenuItem::new(profile.name)
                                        .checked(profile_id == active_id)
                                        .on_click(move |_, _, cx| {
                                            views::run(&state, cx, |state| {
                                                state.select_profile(profile_id)
                                            });
                                        }),
                                )
                            },
                        )
                    }),
            )
            .child(
                Switch::new("profile-playing")
                    .checked(playing)
                    .accessibility_label("Play profile")
                    .tooltip(if playing { "Pause All" } else { "Resume All" })
                    .on_change(move |checked, _, cx| {
                        if *checked != playing {
                            views::run(&pause_state, cx, AppState::toggle_profile_paused);
                        }
                    }),
            )
    }

    /// A Control Center style row: the round toggle turns the sound on or off
    /// and fills with the accent color while it is on.
    fn render_track(
        &self,
        track: Track,
        error: Option<String>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let track_id = track.id;
        let active = track.is_active();
        let toggle_state = self.state.clone();
        let toggle_label = if active { "Turn Off" } else { "Turn On" };

        h_flex()
            .id(("track", track_id))
            .min_h(px(ROW_HEIGHT))
            .gap_2()
            .px_1()
            .child(
                views::icon_button(
                    ("track-toggle", track_id),
                    if active {
                        LucideIcon::Volume2
                    } else {
                        LucideIcon::VolumeX
                    },
                    toggle_label,
                    track.name.clone(),
                )
                .when(active, |button| button.primary())
                .toggled(active)
                .on_click(move |_, _, cx| {
                    views::run(&toggle_state, cx, |state| {
                        state.set_track_active(track_id, !active)
                    });
                }),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(div().text_sm().truncate().child(track.name.clone()))
                    .child(self.sliders.render(&track, cx))
                    .when_some(error, |this, error| {
                        this.child(div().text_xs().text_color(cx.theme().danger).child(error))
                    }),
            )
    }
}

impl Render for PopupView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.state.read(cx);
        let tracks = app.active_profile().tracks.clone();
        let error = app.last_error.clone();
        let track_errors = app.track_errors.clone();
        let manage_state = self.state.clone();
        let theme = cx.theme();
        let surface = Hsla {
            a: 0.86,
            ..theme.popover
        };

        v_flex()
            .id("popup")
            .size_full()
            .overflow_y_scroll()
            .rounded(theme.radius_lg)
            .border_1()
            .border_color(theme.border)
            .bg(surface)
            .text_color(theme.popover_foreground)
            .p(px(PADDING))
            .gap(px(GAP))
            .child(self.render_header(cx))
            .child(separator())
            .child(
                div()
                    .h(px(TITLE_HEIGHT))
                    .px_1()
                    .flex()
                    .items_end()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().muted_foreground)
                    .child("Audio"),
            )
            .when(tracks.is_empty(), |this| {
                this.child(
                    h_flex()
                        .h(px(ROW_HEIGHT))
                        .px_1()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No sounds yet. Add some in Manage Profiles."),
                )
            })
            .children(tracks.into_iter().map(|track| {
                let error = track_errors.get(&track.id).cloned();
                self.render_track(track, error, cx)
            }))
            .when_some(error, |this, error| {
                this.child(Alert::error("popup-error", error).small())
            })
            .child(separator())
            .child(
                footer_button("manage", "Manage Profiles").on_click(move |_, _, cx| {
                    let state = manage_state.clone();
                    cx.defer(move |cx| macos::show_management(cx, state));
                }),
            )
            .child(footer_button("quit", "Quit").on_click(|_, _, cx| macos::quit(cx)))
    }
}

fn separator() -> impl IntoElement {
    // `Separator` paints an absolutely positioned line and takes no height.
    div()
        .h(px(SEPARATOR_HEIGHT))
        .flex()
        .items_center()
        .child(Separator::horizontal().w_full().relative())
}

/// A full-width, left-aligned menu-style row. A label child (not `.label`)
/// is used because GPUI Kit centers button labels.
fn footer_button(id: &'static str, label: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .small()
        .w_full()
        .h(px(FOOTER_HEIGHT))
        .accessibility_label(label)
        .child(div().flex_1().child(label))
}

/// Exact content height for `track_count` rows, so the popup ends right
/// below Quit; the profile menu overlays instead of resizing.
pub(crate) fn preferred_height(track_count: usize) -> f32 {
    let rows = track_count.max(1) as f32;
    let fixed = 2.0 * (PADDING + BORDER)
        + HEADER_HEIGHT
        + TITLE_HEIGHT
        + 2.0 * SEPARATOR_HEIGHT
        + 2.0 * FOOTER_HEIGHT;
    // Children: header, separator, title, rows, separator, two footer rows.
    let gaps = (5.0 + rows) * GAP;
    (fixed + rows * ROW_HEIGHT + gaps).min(MAX_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::preferred_height;

    #[test]
    fn popup_height_is_content_driven_and_bounded() {
        assert_eq!(preferred_height(0), preferred_height(1));
        assert_eq!(preferred_height(1), 208.0);
        assert_eq!(preferred_height(2) - preferred_height(1), 52.0);
        assert_eq!(preferred_height(20), 480.0);
    }
}
