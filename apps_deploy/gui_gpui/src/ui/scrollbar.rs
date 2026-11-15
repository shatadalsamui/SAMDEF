use crate::app::SamdefApp;
use crate::ui::theme::*;
use gpui::{
    canvas, div, fill, point, prelude::*, px, rgb, size, Bounds, Context, IntoElement, MouseButton,
    Pixels, ScrollHandle,
};
use std::cell::Cell;
use std::rc::Rc;

const SCROLLBAR_WIDTH: f32 = 16.0;
const ARROW_BTN_HEIGHT: f32 = 16.0;
const MIN_THUMB_HEIGHT: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct ThumbMetrics {
    thumb_y_offset: f32,
    thumb_height: f32,
    travel: f32,
}

#[inline]
fn resolve_max_scroll(handle: &ScrollHandle, viewport_height: f32, content_height_est: f32) -> f32 {
    let gpui_max: f32 = handle.max_offset().height.into();
    if gpui_max > 0.5 {
        gpui_max
    } else {
        (content_height_est - viewport_height).max(0.0)
    }
}

#[inline]
fn compute_thumb_metrics(
    track_height: f32,
    viewport_height: f32,
    max_scroll: f32,
    scroll_top: f32,
) -> ThumbMetrics {
    if track_height <= 1.0 || max_scroll <= 0.5 {
        return ThumbMetrics {
            thumb_y_offset: 0.0,
            thumb_height: track_height.max(0.0),
            travel: 0.0,
        };
    }
    let total_content = (viewport_height + max_scroll).max(viewport_height);
    let thumb_height = ((viewport_height / total_content) * track_height).clamp(MIN_THUMB_HEIGHT, track_height);
    let travel = (track_height - thumb_height).max(0.0);
    let frac = (scroll_top / max_scroll).clamp(0.0, 1.0);
    ThumbMetrics {
        thumb_y_offset: frac * travel,
        thumb_height,
        travel,
    }
}

#[inline]
fn scroll_top_from_track_y(
    track_origin_y: f32,
    track_height: f32,
    cursor_y: f32,
    viewport_height: f32,
    max_scroll: f32,
) -> f32 {
    if max_scroll <= 0.5 || track_height <= 1.0 {
        return 0.0;
    }
    let metrics = compute_thumb_metrics(track_height, viewport_height, max_scroll, 0.0);
    if metrics.travel <= 0.5 {
        return 0.0;
    }
    let frac = ((cursor_y - track_origin_y - metrics.thumb_height * 0.5) / metrics.travel).clamp(0.0, 1.0);
    frac * max_scroll
}

#[inline]
fn stepped_scroll_top(current_offset_y: f32, delta_px: f32, max_scroll: f32) -> f32 {
    if max_scroll <= 0.5 {
        return 0.0;
    }
    let cur_top = (-current_offset_y).clamp(0.0, max_scroll);
    (cur_top + delta_px).clamp(0.0, max_scroll)
}

fn apply_scroll_from_cursor(
    handle: &ScrollHandle,
    track_bounds: Bounds<Pixels>,
    cursor_y: Pixels,
    viewport_height: f32,
    max_scroll: f32,
) {
    if max_scroll <= 0.5 {
        return;
    }
    let ty: f32 = track_bounds.origin.y.into();
    let th: f32 = track_bounds.size.height.into();
    let wy: f32 = cursor_y.into();
    let new_top = scroll_top_from_track_y(ty, th, wy, viewport_height, max_scroll);
    handle.set_offset(point(px(0.0), px(-new_top)));
}

/// Reusable vertical scrollbar with Up (`^`) / Down (`v`) step buttons and a
/// click-and-drag thumb track backed by a GPUI `ScrollHandle`.
#[allow(clippy::too_many_arguments)]
pub fn render_vertical_scrollbar(
    id_prefix: &'static str,
    scroll_handle: ScrollHandle,
    track_bounds_cell: Rc<Cell<Bounds<Pixels>>>,
    viewport_height: f32,
    content_height_est: f32,
    step_px: f32,
    thumb_accent_hex: u32,
    cx: &Context<SamdefApp>,
) -> impl IntoElement {
    let max_scroll = resolve_max_scroll(&scroll_handle, viewport_height, content_height_est);
    let can_scroll = max_scroll > 0.5;
    let cur_offset_y: f32 = scroll_handle.offset().y.into();
    let scroll_top = (-cur_offset_y).clamp(0.0, max_scroll);

    let up_handle = scroll_handle.clone();
    let down_handle = scroll_handle.clone();
    let track_down_handle = scroll_handle.clone();
    let track_move_handle = scroll_handle.clone();
    let track_down_bounds = track_bounds_cell.clone();
    let track_move_bounds = track_bounds_cell.clone();
    let paint_bounds_cell = track_bounds_cell;

    div()
        .id((id_prefix, 0usize))
        .w(px(SCROLLBAR_WIDTH))
        .h_full()
        .bg(rgb(BG_CARD))
        .border_l_1()
        .border_color(rgb(BORDER_SUBTLE))
        .flex()
        .flex_col()
        .child(
            div()
                .id((id_prefix, 1usize))
                .h(px(ARROW_BTN_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .border_b_1()
                .border_color(rgb(BORDER_SUBTLE))
                .text_xs()
                .text_color(rgb(if can_scroll { TEXT_SECONDARY } else { TEXT_DIM }))
                .hover(|s| if can_scroll { s.bg(rgb(BG_HOVER)).cursor_pointer() } else { s })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_, _, _, cx| {
                        let cur_y: f32 = up_handle.offset().y.into();
                        let new_top = stepped_scroll_top(cur_y, -step_px, max_scroll);
                        up_handle.set_offset(point(px(0.0), px(-new_top)));
                        cx.notify();
                    }),
                )
                .child("^"),
        )
        .child(
            div()
                .id((id_prefix, 2usize))
                .flex_1()
                .w_full()
                .bg(rgb(BG_ROOT))
                .hover(|s| if can_scroll { s.cursor_pointer() } else { s })
                .child(
                    canvas(
                        move |bounds, _, _| {
                            paint_bounds_cell.set(bounds);
                            bounds
                        },
                        move |bounds, _, window, _| {
                            let bx: f32 = bounds.origin.x.into();
                            let by: f32 = bounds.origin.y.into();
                            let bw: f32 = bounds.size.width.into();
                            let bh: f32 = bounds.size.height.into();
                            if bw <= 2.0 || bh <= 2.0 {
                                return;
                            }

                            let metrics =
                                compute_thumb_metrics(bh, viewport_height, max_scroll, scroll_top);
                            let thumb_color = if can_scroll {
                                thumb_accent_hex
                            } else {
                                BORDER_STRONG
                            };
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(px(bx + 3.0), px(by + metrics.thumb_y_offset + 1.0)),
                                    size(
                                        px((bw - 6.0).max(4.0)),
                                        px((metrics.thumb_height - 2.0).max(6.0)),
                                    ),
                                ),
                                rgb(thumb_color),
                            ));
                        },
                    )
                    .size_full(),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_, ev: &gpui::MouseDownEvent, _, cx| {
                        apply_scroll_from_cursor(
                            &track_down_handle,
                            track_down_bounds.get(),
                            ev.position.y,
                            viewport_height,
                            max_scroll,
                        );
                        cx.notify();
                    }),
                )
                .on_mouse_move(cx.listener(move |this, ev: &gpui::MouseMoveEvent, _, cx| {
                    if !this.viewer_state.is_dragging
                        && ev.pressed_button == Some(MouseButton::Left)
                    {
                        apply_scroll_from_cursor(
                            &track_move_handle,
                            track_move_bounds.get(),
                            ev.position.y,
                            viewport_height,
                            max_scroll,
                        );
                        cx.notify();
                    }
                })),
        )
        .child(
            div()
                .id((id_prefix, 3usize))
                .h(px(ARROW_BTN_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .border_t_1()
                .border_color(rgb(BORDER_SUBTLE))
                .text_xs()
                .text_color(rgb(if can_scroll { TEXT_SECONDARY } else { TEXT_DIM }))
                .hover(|s| if can_scroll { s.bg(rgb(BG_HOVER)).cursor_pointer() } else { s })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_, _, _, cx| {
                        let cur_y: f32 = down_handle.offset().y.into();
                        let new_top = stepped_scroll_top(cur_y, step_px, max_scroll);
                        down_handle.set_offset(point(px(0.0), px(-new_top)));
                        cx.notify();
                    }),
                )
                .child("v"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_thumb_metrics_bounds() {
        let non_scrollable = compute_thumb_metrics(150.0, 182.0, 0.0, 0.0);
        assert_eq!(non_scrollable.thumb_height, 150.0);
        assert_eq!(non_scrollable.travel, 0.0);

        let top = compute_thumb_metrics(150.0, 182.0, 818.0, 0.0);
        assert!((top.thumb_height - 27.3).abs() < 0.2);
        assert_eq!(top.thumb_y_offset, 0.0);

        let bottom = compute_thumb_metrics(150.0, 182.0, 818.0, 818.0);
        assert!((bottom.thumb_y_offset + bottom.thumb_height - 150.0).abs() < 1e-4);
    }

    #[test]
    fn test_stepped_and_track_scroll_clamping() {
        assert_eq!(stepped_scroll_top(0.0, -78.0, 500.0), 0.0);
        assert_eq!(stepped_scroll_top(-100.0, 78.0, 500.0), 178.0);
        assert_eq!(stepped_scroll_top(-480.0, 78.0, 500.0), 500.0);

        let mid = scroll_top_from_track_y(100.0, 150.0, 175.0, 182.0, 818.0);
        assert!((mid - 409.0).abs() < 1.0);
    }
}
