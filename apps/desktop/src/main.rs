use gpui_kit::{App, AppContext};
use jalak_desktop::{app::AppState, assets::AppAssets, platform::macos, theme};

fn main() {
    let application = gpui_kit::application().with_assets(AppAssets);
    application.on_reopen(macos::show_management_on_reopen);
    application.run(|cx: &mut App| {
        gpui_kit::init(cx);
        theme::init(cx);
        let state = match AppState::load() {
            Ok(state) => cx.new(|_| state),
            Err(error) => {
                eprintln!("Jalak failed to start: {error:#}");
                cx.quit();
                return;
            }
        };
        if let Err(error) = macos::install(cx, state.clone()) {
            eprintln!("Jalak failed to create its menu-bar item: {error:#}");
            cx.quit();
            return;
        }
        macos::show_management(cx, state);
    });
}
