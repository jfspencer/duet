//! Duet desktop application entry point.
//!
//! The binary opens one window whose root view is [`DuetApp`], wrapped in the
//! GPUI Component [`Root`] so overlays (dialogs, sheets, notifications) have a
//! host element. Structured logging goes through `tracing`; nothing in this
//! crate writes to stdout or stderr directly.

#![forbid(unsafe_code)]

mod app;

use gpui_kit::component::Root;
use gpui_kit::{
    App, AppContext as _, AsyncApp, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point,
    px, size,
};
use tracing_subscriber::EnvFilter;

use crate::app::DuetApp;

/// Install the process-wide `tracing` subscriber.
///
/// `RUST_LOG` selects the filter; the default is `info` for this crate and
/// `warn` for everything else.
fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,duet=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

/// The window options for the main window.
fn main_window_options() -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("Duet".into()),
            ..TitlebarOptions::default()
        }),
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(120.0), px(120.0)),
            size: size(px(960.0), px(640.0)),
        })),
        ..WindowOptions::default()
    }
}

/// Open the main window, or log why it could not open and quit.
fn open_main_window(cx: &AsyncApp) {
    let opened = cx.open_window(main_window_options(), |window, cx| {
        let view = cx.new(|_| DuetApp::default());
        cx.new(|cx| Root::new(view, window, cx))
    });
    if let Err(error) = opened {
        tracing::error!(%error, "failed to open the main window");
        cx.update(|app| app.quit());
    }
}

fn main() {
    init_tracing();
    tracing::info!("starting duet");
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|app: &mut App| {
            gpui_kit::init(app);
            app.spawn(async move |cx: &mut AsyncApp| open_main_window(cx))
                .detach();
        });
}
