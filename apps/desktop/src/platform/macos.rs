use std::cell::RefCell;

use anyhow::{Context as _, Result};
use block2::RcBlock;
use gpui_kit::component::Root;
use gpui_kit::{
    App, AppContext, AsyncApp, BorrowAppContext, Bounds, Entity, Global,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions, point, px,
    size,
};
use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSApplicationActivationPolicy, NSControl,
    NSEvent, NSEventMask, NSScreen, NSSquareStatusItemLength, NSStatusBar, NSStatusItem,
    NSTextField,
};
use objc2_foundation::{NSObject, NSPoint, NSRect, NSSize, NSString};

use crate::{
    app::AppState,
    views::{
        management::ManagementView,
        popup::{POPUP_WIDTH, PopupView, preferred_height},
    },
};

type StatusCallback = Box<dyn FnMut(&NSControl)>;

struct StatusTargetIvars {
    callback: RefCell<StatusCallback>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = StatusTargetIvars]
    struct StatusTarget;

    unsafe impl NSObjectProtocol for StatusTarget {}

    impl StatusTarget {
        #[unsafe(method(clicked:))]
        fn clicked(&self, sender: &NSControl) {
            (self.ivars().callback.borrow_mut())(sender);
        }
    }
);

impl StatusTarget {
    fn new(mtm: MainThreadMarker, callback: impl FnMut(&NSControl) + 'static) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(StatusTargetIvars {
            callback: RefCell::new(Box::new(callback)),
        });
        unsafe { msg_send![super(this), init] }
    }
}

struct StatusController {
    _item: Retained<NSStatusItem>,
    _target: Retained<StatusTarget>,
    _outside_click_monitor: Retained<objc2::runtime::AnyObject>,
}

pub struct WindowRegistry {
    _status: StatusController,
    state: Entity<AppState>,
    popup: Option<WindowHandle<Root>>,
    management: Option<WindowHandle<Root>>,
}

impl Global for WindowRegistry {}

pub fn install(cx: &mut App, state: Entity<AppState>) -> Result<()> {
    let mtm = MainThreadMarker::new().context("Jalak must start on the macOS main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSSquareStatusItemLength);
    let button = item.button(mtm).context("status item has no button")?;
    button.setTitle(&NSString::from_str("♫"));
    button.setToolTip(Some(&NSString::from_str("Jalak audio profiles")));

    let async_app = cx.to_async();
    let callback_state = state.clone();
    let target = StatusTarget::new(mtm, move |sender| {
        let origin = popup_origin(sender);
        schedule(&async_app, {
            let state = callback_state.clone();
            move |cx| toggle_popup(cx, state, origin)
        });
    });
    unsafe {
        button.setTarget(Some(&target));
        button.setAction(Some(sel!(clicked:)));
    }

    let monitor_app = cx.to_async();
    let outside_click = RcBlock::new(move |_: std::ptr::NonNull<NSEvent>| {
        schedule(&monitor_app, |cx| {
            dismiss_popup(cx);
        });
    });
    let outside_click_monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
        NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown | NSEventMask::OtherMouseDown,
        &outside_click,
    )
    .context("failed to monitor outside clicks")?;

    cx.set_global(WindowRegistry {
        _status: StatusController {
            _item: item,
            _target: target,
            _outside_click_monitor: outside_click_monitor,
        },
        state,
        popup: None,
        management: None,
    });
    Ok(())
}

pub fn show_management_on_reopen(cx: &mut App) {
    let state = cx.global::<WindowRegistry>().state.clone();
    show_management(cx, state);
}

pub fn toggle_popup(
    cx: &mut App,
    state: Entity<AppState>,
    origin: Option<gpui_kit::Point<gpui_kit::Pixels>>,
) {
    if dismiss_popup(cx) {
        return;
    }
    cx.update_global::<WindowRegistry, _>(|registry, cx| {
        let popup_size = size(
            px(POPUP_WIDTH),
            px(preferred_height(
                state.read(cx).active_profile().tracks.len(),
            )),
        );
        let Some(origin) = origin.or_else(|| {
            cx.primary_display().map(|display| {
                let bounds = display.bounds();
                point(
                    bounds.right() - popup_size.width - px(10.0),
                    bounds.top() + px(8.0),
                )
            })
        }) else {
            return;
        };
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(origin, popup_size))),
            titlebar: None,
            focus: true,
            show: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Blurred,
            ..Default::default()
        };
        match cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| PopupView::new(state.clone(), window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        }) {
            Ok(handle) => registry.popup = Some(handle),
            Err(error) => state.update(cx, |state, context| {
                state.last_error = Some(format!("failed to open menu: {error:#}"));
                context.notify();
            }),
        }
    })
}

fn popup_origin(sender: &NSControl) -> Option<gpui_kit::Point<gpui_kit::Pixels>> {
    let mtm = MainThreadMarker::new()?;
    let mouse = NSEvent::mouseLocation();
    let converted_x = sender
        .window()
        .map(|window| window.convertRectToScreen(sender.convertRect_toView(sender.bounds(), None)))
        .filter(|frame| frame.size.width > 0.0 && frame.size.width <= 80.0)
        .map(|frame| frame.origin.x + frame.size.width / 2.0);
    let screens = NSScreen::screens(mtm);
    let status_x = converted_x
        .filter(|x| {
            (0..screens.count()).any(|index| {
                let frame = screens.objectAtIndex(index).frame();
                *x >= frame.origin.x && *x <= frame.origin.x + frame.size.width
            })
        })
        .unwrap_or(mouse.x);
    let screen = (0..screens.count())
        .map(|index| screens.objectAtIndex(index))
        .find(|screen| {
            let frame = screen.frame();
            status_x >= frame.origin.x
                && status_x <= frame.origin.x + frame.size.width
                && mouse.y >= frame.origin.y
                && mouse.y <= frame.origin.y + frame.size.height
        })
        .or_else(|| {
            (0..screens.count())
                .map(|index| screens.objectAtIndex(index))
                .find(|screen| {
                    let frame = screen.frame();
                    status_x >= frame.origin.x && status_x <= frame.origin.x + frame.size.width
                })
        })?;
    let primary_height = screens.objectAtIndex(0).frame().size.height;
    let visible = screen.visibleFrame();
    let popup_width = f64::from(POPUP_WIDTH);
    let popup_height = 480.0;
    let visible_top = primary_height - visible.origin.y - visible.size.height;
    let visible_bottom = primary_height - visible.origin.y;
    let x = (status_x - popup_width / 2.0).clamp(
        visible.origin.x + 8.0,
        visible.origin.x + visible.size.width - popup_width - 8.0,
    );
    let y = (visible_top + 8.0).clamp(visible_top + 6.0, visible_bottom - popup_height - 8.0);
    Some(point(px(x as f32), px(y as f32)))
}

pub fn show_management(cx: &mut App, state: Entity<AppState>) {
    dismiss_popup(cx);
    cx.update_global::<WindowRegistry, _>(|registry, cx| {
        if let Some(handle) = registry.management
            && handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
        {
            cx.activate(true);
            return;
        }

        let bounds = Bounds::centered(None, size(px(680.0), px(620.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::TitlebarOptions {
                title: Some("Jalak".into()),
                ..Default::default()
            }),
            // Keeps the sidebar plus header actions visible.
            window_min_size: Some(size(px(560.0), px(400.0))),
            ..Default::default()
        };
        match cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| ManagementView::new(state.clone(), window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        }) {
            Ok(handle) => {
                registry.management = Some(handle);
                cx.activate(true);
            }
            Err(error) => state.update(cx, |state, context| {
                state.last_error = Some(format!("failed to open management window: {error:#}"));
                context.notify();
            }),
        }
    });
}

pub(crate) fn dismiss_popup(cx: &mut App) -> bool {
    cx.update_global::<WindowRegistry, _>(|registry, cx| {
        if let Some(handle) = registry.popup.take() {
            handle
                .update(cx, |_, window, _| window.remove_window())
                .is_ok()
        } else {
            false
        }
    })
}

pub fn quit(cx: &mut App) {
    cx.quit();
}

pub fn prompt_name(title: &str, current: &str) -> Option<String> {
    let mtm = MainThreadMarker::new()?;
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str("Names must be non-empty and unique."));
    alert.addButtonWithTitle(&NSString::from_str("Save"));
    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    let field = NSTextField::initWithFrame(
        NSTextField::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(280.0, 24.0)),
    );
    field.setStringValue(&NSString::from_str(current));
    alert.setAccessoryView(Some(&field));
    (alert.runModal() == NSAlertFirstButtonReturn).then(|| field.stringValue().to_string())
}

pub fn confirm(title: &str, message: &str) -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(message));
    alert.addButtonWithTitle(&NSString::from_str("Delete"));
    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    alert.runModal() == NSAlertFirstButtonReturn
}

fn schedule(async_app: &AsyncApp, action: impl FnOnce(&mut App) + 'static) {
    let app = async_app.clone();
    async_app
        .foreground_executor()
        .spawn(async move {
            app.update(action);
        })
        .detach();
}
