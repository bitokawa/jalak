use std::collections::HashMap;

use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Styled, Subscription, Window,
    component::{
        ActiveTheme, h_flex,
        slider::{Slider, SliderState},
    },
    div, px,
};

use crate::{app::AppState, model::Track, views};

const VOLUME_STEPS: f32 = 100.0;

/// One transient volume slider per active track. `AppState` stays
/// authoritative: slider changes call `set_track_volume`, and state changes
/// are mirrored back through `sync`.
#[derive(Default)]
pub struct TrackSliders {
    sliders: HashMap<u64, (Entity<SliderState>, Subscription)>,
}

impl TrackSliders {
    pub fn sync<V: 'static>(
        &mut self,
        state: &Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<V>,
    ) {
        let tracks = state.read(cx).active_profile().tracks.clone();
        self.sliders
            .retain(|id, _| tracks.iter().any(|track| track.id == *id));
        for track in tracks {
            let value = track.volume * VOLUME_STEPS;
            if let Some((slider, _)) = self.sliders.get(&track.id) {
                slider.update(cx, |slider, cx| {
                    if (slider.value().end() - value).abs() >= 0.5 {
                        slider.set_value(value, window, cx);
                    }
                });
                continue;
            }
            let slider = cx.new(|_| {
                SliderState::new()
                    .min(0.0)
                    .max(VOLUME_STEPS)
                    .step(1.0)
                    .default_value(value)
            });
            let track_id = track.id;
            let state = state.clone();
            // Observing (not subscribing) also catches VoiceOver increments,
            // which update the value without emitting `SliderEvent`.
            // ponytail: each whole-percent change saves config; debounce if
            // drag-heavy use shows disk churn.
            let subscription = cx.observe(&slider, move |_, slider, cx| {
                let volume = slider.read(cx).value().end() / VOLUME_STEPS;
                let current = state
                    .read(cx)
                    .active_profile()
                    .tracks
                    .iter()
                    .find(|track| track.id == track_id)
                    .map(|track| track.volume);
                if current.is_some_and(|current| (current - volume).abs() >= 0.005) {
                    views::run(&state, cx, |state| state.set_track_volume(track_id, volume));
                }
            });
            self.sliders.insert(track.id, (slider, subscription));
        }
    }

    /// The track's slider followed by its percentage.
    pub fn render(&self, track: &Track, cx: &gpui_kit::App) -> impl IntoElement {
        h_flex()
            .gap_2()
            .children(
                self.sliders
                    .get(&track.id)
                    .map(|(slider, _)| Slider::new(slider).flex_1()),
            )
            .child(
                div()
                    .w(px(32.0))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{}%", (track.volume * VOLUME_STEPS).round() as u8)),
            )
    }
}
