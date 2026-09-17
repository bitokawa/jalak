use gpui_kit::{
    App, Context, Entity, InteractiveElement, ParentElement, Styled, Window,
    component::{
        ActiveTheme, IndexPath, h_flex,
        list::{ListDelegate, ListItem, ListState},
        menu::ContextMenuExt,
    },
    div,
};

use crate::{app::AppState, model::Profile, views};

#[derive(Clone)]
struct Row {
    id: u64,
    name: String,
    tracks: usize,
}

/// Profile rows for the management sidebar. `AppState` stays authoritative:
/// the rows are a render snapshot and selection dispatches `select_profile`.
pub struct ProfileList {
    state: Entity<AppState>,
    rows: Vec<Row>,
}

impl ProfileList {
    pub fn new(state: Entity<AppState>) -> Self {
        Self {
            state,
            rows: Vec::new(),
        }
    }

    /// Replaces the snapshot and returns the active profile's row.
    pub fn sync(&mut self, profiles: &[Profile], active_id: u64) -> Option<IndexPath> {
        self.rows = profiles
            .iter()
            .map(|profile| Row {
                id: profile.id,
                name: profile.name.clone(),
                tracks: profile.tracks.len(),
            })
            .collect();
        self.rows
            .iter()
            .position(|row| row.id == active_id)
            .map(IndexPath::new)
    }
}

impl ListDelegate for ProfileList {
    type Item = ListItem;

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.rows.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let row = self.rows.get(ix.row)?.clone();
        let state = self.state.clone();
        let can_delete = self.rows.len() > 1;
        let muted = cx.theme().muted_foreground;
        Some(
            ListItem::new(("profile", row.id)).px_2().py_1().child(
                h_flex()
                    .id(("profile-row", row.id))
                    .w_full()
                    .justify_between()
                    .gap_2()
                    .child(div().truncate().child(row.name.clone()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(row.tracks.to_string()),
                    )
                    .context_menu(move |menu, _, _| {
                        views::profile_menu(menu, &state, row.id, &row.name, can_delete)
                    }),
            ),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        let Some(row) = ix.and_then(|ix| self.rows.get(ix.row)) else {
            return;
        };
        let id = row.id;
        if self.state.read(cx).config.active_profile_id != id {
            views::run(&self.state, cx, |state| state.select_profile(id));
        }
    }
}
