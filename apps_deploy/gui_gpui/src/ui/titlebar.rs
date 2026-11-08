use crate::app::SamdefApp;
use crate::ui::theme::*;
use gpui::{
    canvas, div, point, prelude::*, px, rgb, Context, CursorStyle, IntoElement, MouseButton,
    PathBuilder, ResizeEdge, Window,
};

#[derive(Clone, Copy)]
enum WinIcon {
    Minimize,
    Maximize,
    Restore,
    FullscreenEnter,
    FullscreenExit,
    Close,
}

fn render_win_icon(kind: WinIcon, color_hex: u32) -> impl IntoElement {
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| {
            let (x, y): (f32, f32) = (bounds.origin.x.into(), bounds.origin.y.into());
            let mut b = PathBuilder::stroke(px(1.4));
            let seg = |b: &mut PathBuilder, x1: f32, y1: f32, x2: f32, y2: f32| {
                b.move_to(point(px(x + x1), px(y + y1)));
                b.line_to(point(px(x + x2), px(y + y2)));
            };
            let rect = |b: &mut PathBuilder, x1: f32, y1: f32, x2: f32, y2: f32| {
                b.move_to(point(px(x + x1), px(y + y1)));
                b.line_to(point(px(x + x2), px(y + y1)));
                b.line_to(point(px(x + x2), px(y + y2)));
                b.line_to(point(px(x + x1), px(y + y2)));
                b.line_to(point(px(x + x1), px(y + y1)));
            };

            match kind {
                WinIcon::Minimize => seg(&mut b, 1.0, 6.5, 11.0, 6.5),
                WinIcon::Maximize => rect(&mut b, 1.5, 1.5, 10.5, 10.5),
                WinIcon::Restore => {
                    rect(&mut b, 1.5, 3.5, 8.5, 10.5);
                    seg(&mut b, 3.5, 3.5, 3.5, 1.5);
                    seg(&mut b, 3.5, 1.5, 10.5, 1.5);
                    seg(&mut b, 10.5, 1.5, 10.5, 8.5);
                    seg(&mut b, 10.5, 8.5, 8.5, 8.5);
                }
                WinIcon::FullscreenEnter | WinIcon::FullscreenExit => {
                    let corners = if matches!(kind, WinIcon::FullscreenEnter) {
                        [(1.0, 1.0, 3.5, 3.5), (11.0, 1.0, -3.5, 3.5), (1.0, 11.0, 3.5, -3.5), (11.0, 11.0, -3.5, -3.5)]
                    } else {
                        [(4.0, 4.0, -3.0, -3.0), (8.0, 4.0, 3.0, -3.0), (4.0, 8.0, -3.0, 3.0), (8.0, 8.0, 3.0, 3.0)]
                    };
                    for (cx_p, cy_p, dx, dy) in corners {
                        seg(&mut b, cx_p + dx, cy_p, cx_p, cy_p);
                        seg(&mut b, cx_p, cy_p, cx_p, cy_p + dy);
                    }
                }
                WinIcon::Close => {
                    seg(&mut b, 1.5, 1.5, 10.5, 10.5);
                    seg(&mut b, 10.5, 1.5, 1.5, 10.5);
                }
            }
            if let Ok(path) = b.build() {
                window.paint_path(path, rgb(color_hex));
            }
        },
    )
    .w(px(12.0))
    .h(px(12.0))
}

impl SamdefApp {
    /// Client-side window titlebar with drag-move, system menu, Fullscreen, Minimize, Maximize & Close
    pub fn render_titlebar(&self, window: &Window, _cx: &Context<Self>) -> impl IntoElement {
        let is_maximized = window.is_maximized();
        let is_fullscreen = window.is_fullscreen();
        let scene_label = self
            .current_json_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("No Scene Loaded")
            .to_string();

        div()
            .id("custom-window-titlebar")
            .h(px(34.0))
            .w_full()
            .bg(rgb(0x0b0b0e))
            .border_b_1()
            .border_color(rgb(BORDER_STRONG))
            .flex()
            .flex_row()
            .items_center()
            .flex_none()
            // Draggable titlebar region (sibling to buttons so button clicks never start drag)
            .child(
                div()
                    .id("titlebar-drag-area")
                    .flex_1()
                    .h_full()
                    .px_3()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2p5()
                    .on_mouse_down(MouseButton::Left, |ev, window, _| {
                        if ev.click_count >= 2 {
                            window.zoom_window();
                        } else {
                            window.start_window_move();
                        }
                    })
                    .on_mouse_down(MouseButton::Right, |ev, window, _| {
                        window.show_window_menu(ev.position);
                    })
                    .child(div().size_2p5().rounded_full().bg(rgb(ACCENT_CYAN)))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(TEXT_PRIMARY))
                            .child("SAMDEF GIS VIEWER"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_xs()
                            .bg(rgb(BG_INPUT))
                            .border_1()
                            .border_color(rgb(BORDER_SUBTLE))
                            .text_xs()
                            .text_color(rgb(ACCENT_BLUE))
                            .child(scene_label),
                    ),
            )
            // Right window control cluster: Fullscreen [F11] | Minimize | Maximize/Restore | Close
            .child(
                div()
                    .id("titlebar-window-controls")
                    .h_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    // 1. Fullscreen toggle button
                    .child(
                        div()
                            .id("win-btn-fullscreen")
                            .h_full()
                            .px_2p5()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .bg(rgb(if is_fullscreen { ACCENT_BLUE } else { 0x0b0b0e }))
                            .hover(|s| s.bg(rgb(if is_fullscreen { 0x2563eb } else { BG_HOVER })).cursor_pointer())
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                cx.stop_propagation();
                                window.toggle_fullscreen();
                            })
                            .child(render_win_icon(
                                if is_fullscreen {
                                    WinIcon::FullscreenExit
                                } else {
                                    WinIcon::FullscreenEnter
                                },
                                TEXT_PRIMARY,
                            ))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_SECONDARY))
                                    .child(if is_fullscreen { "Exit [F11]" } else { "F11" }),
                            ),
                    )
                    // 2. Minimize button
                    .child(
                        div()
                            .id("win-btn-minimize")
                            .w(px(40.0))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                cx.stop_propagation();
                                window.minimize_window();
                            })
                            .child(render_win_icon(WinIcon::Minimize, TEXT_PRIMARY)),
                    )
                    // 3. Maximize / Restore button
                    .child(
                        div()
                            .id("win-btn-maximize")
                            .w(px(40.0))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                cx.stop_propagation();
                                window.zoom_window();
                            })
                            .child(render_win_icon(
                                if is_maximized {
                                    WinIcon::Restore
                                } else {
                                    WinIcon::Maximize
                                },
                                TEXT_PRIMARY,
                            )),
                    )
                    // 4. Close button (red hover)
                    .child(
                        div()
                            .id("win-btn-close")
                            .w(px(44.0))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(0xdc2626)).cursor_pointer())
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                cx.stop_propagation();
                                window.remove_window();
                            })
                            .child(render_win_icon(WinIcon::Close, TEXT_PRIMARY)),
                    ),
            )
    }

    /// Thin interactive border resize strips when window is un-maximized in CSD mode
    pub fn render_resize_strips(window: &Window) -> Option<impl IntoElement> {
        if window.is_maximized() || window.is_fullscreen() {
            return None;
        }
        let strip = |id: &'static str, edge: ResizeEdge, cursor: CursorStyle| {
            div()
                .id(id)
                .absolute()
                .cursor(cursor)
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    window.start_window_resize(edge);
                })
        };

        Some(
            div()
                .child(strip("rsz-n", ResizeEdge::Top, CursorStyle::ResizeUpDown).top_0().left(px(6.0)).right(px(6.0)).h(px(4.0)))
                .child(strip("rsz-s", ResizeEdge::Bottom, CursorStyle::ResizeUpDown).bottom_0().left(px(6.0)).right(px(6.0)).h(px(4.0)))
                .child(strip("rsz-w", ResizeEdge::Left, CursorStyle::ResizeLeftRight).left_0().top(px(6.0)).bottom(px(6.0)).w(px(4.0)))
                .child(strip("rsz-e", ResizeEdge::Right, CursorStyle::ResizeLeftRight).right_0().top(px(6.0)).bottom(px(6.0)).w(px(4.0)))
                .child(strip("rsz-nw", ResizeEdge::TopLeft, CursorStyle::ResizeUpLeftDownRight).top_0().left_0().size(px(8.0)))
                .child(strip("rsz-se", ResizeEdge::BottomRight, CursorStyle::ResizeUpLeftDownRight).bottom_0().right_0().size(px(8.0)))
                .child(strip("rsz-ne", ResizeEdge::TopRight, CursorStyle::ResizeUpRightDownLeft).top_0().right_0().size(px(8.0)))
                .child(strip("rsz-sw", ResizeEdge::BottomLeft, CursorStyle::ResizeUpRightDownLeft).bottom_0().left_0().size(px(8.0))),
        )
    }
}
