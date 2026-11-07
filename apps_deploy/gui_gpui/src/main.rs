mod app;
mod geo;
mod io;
mod model;
mod ui;
mod viewer;

use app::SamdefApp;
use gpui::{
    px, size, App, AppContext, Application, Bounds, WindowBackgroundAppearance, WindowBounds,
    WindowDecorations, WindowOptions,
};

fn main() {
    println!("Starting SAMDEF GIS Exploitation Viewer [GPUI Edition]...");

    Application::new().run(|cx: &mut App| {
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let restore_bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        let window_options = WindowOptions {
            titlebar: None,
            window_decorations: Some(WindowDecorations::Client),
            window_background: WindowBackgroundAppearance::Opaque,
            window_bounds: Some(WindowBounds::Maximized(restore_bounds)),
            window_min_size: Some(size(px(960.0), px(620.0))),
            app_id: Some("samdef-gis-viewer".to_string()),
            focus: true,
            ..Default::default()
        };

        let window = cx
            .open_window(window_options, |window, cx| {
                window.set_window_title("SAMDEF GIS Exploitation Viewer");
                cx.new(|cx| {
                    cx.observe_window_bounds(window, |_, _, cx| cx.notify())
                        .detach();
                    SamdefApp::new(cx)
                })
            })
            .expect("Failed to initialize GPUI window");

        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
        cx.observe_keystrokes(move |ev, window, cx| {
            view.update(cx, |app, cx| {
                app.handle_keystroke(ev, window, cx);
            });
        })
        .detach();

        cx.activate(true);
    });
}
