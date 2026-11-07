use crate::app::SamdefApp;
use crate::ui::theme::*;
use crate::viewer::paint_viewer;
use gpui::{canvas, div, prelude::*, rgb, Context, MouseButton, Render, Window};

impl Render for SamdefApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let titlebar = self.render_titlebar(window, cx);
        let resize_strips = Self::render_resize_strips(window);
        let sidebar = self.render_sidebar(cx);
        let toolbar = self.render_toolbar(cx);
        let status_bar = self.render_status_bar();

        let active_det = self
            .viewer_state
            .selected_detection_id
            .and_then(|id| {
                self.detections
                    .iter()
                    .find(|d| d.id == id)
                    .copied()
                    .map(|d| (d, true))
            })
            .or_else(|| {
                self.viewer_state.hovered_detection_id.and_then(|id| {
                    self.detections
                        .iter()
                        .find(|d| d.id == id)
                        .copied()
                        .map(|d| (d, false))
                })
            });

        let inspector_element = active_det.map(|(d, is_pinned)| self.render_inspector(d, is_pinned, cx));
        let minimap_element = self
            .viewer_state
            .show_minimap
            .then(|| self.render_minimap(cx));
        let table_element = self
            .show_table_drawer
            .then(|| self.render_table_drawer(cx));

        let loaded_img = self.current_image.clone();
        let detections = self.detections.clone();
        let class_vis = self.class_visible;
        let conf_thresh = self.confidence_threshold;
        let geo_transform = self.geo_transform;
        let state_copy = self.viewer_state.clone();
        let vp_bounds_cell = self.viewport_bounds.clone();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(BG_ROOT))
            .border_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(titlebar)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .flex()
                    .flex_row()
                    .child(sidebar)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(toolbar)
                            .child(
                                div()
                                    .id("viewport-container")
                                    .flex_1()
                                    .relative()
                                    .overflow_hidden()
                                    .bg(rgb(0x070709))
                                    .child(
                                        div()
                                            .id("viewport-canvas-interactive")
                                            .size_full()
                                            .child(
                                                canvas(
                                                    move |bounds, _, _| {
                                                        vp_bounds_cell.set(bounds);
                                                        bounds
                                                    },
                                                    move |bounds, _, window, cx| {
                                                        paint_viewer(
                                                            window,
                                                            cx,
                                                            bounds,
                                                            loaded_img.as_ref(),
                                                            &detections,
                                                            &class_vis,
                                                            conf_thresh,
                                                            &geo_transform,
                                                            &state_copy,
                                                        );
                                                    },
                                                )
                                                .size_full(),
                                            )
                                            .on_scroll_wheel(cx.listener(|this, ev, _, cx| {
                                                this.on_viewport_scroll(ev, cx);
                                            }))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_down(
                                                        MouseButton::Left,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_down(
                                                MouseButton::Right,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_down(
                                                        MouseButton::Right,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_down(
                                                MouseButton::Middle,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_down(
                                                        MouseButton::Middle,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_move(cx.listener(|this, ev, _, cx| {
                                                this.on_viewport_mouse_move(ev, cx);
                                            }))
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_up(
                                                        MouseButton::Left,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_up(
                                                MouseButton::Right,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_up(
                                                        MouseButton::Right,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_up(
                                                MouseButton::Middle,
                                                cx.listener(|this, ev, _, cx| {
                                                    this.on_viewport_mouse_up(
                                                        MouseButton::Middle,
                                                        ev,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .on_mouse_up_out(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.on_viewport_mouse_up_out(cx);
                                                }),
                                            )
                                            .on_mouse_up_out(
                                                MouseButton::Right,
                                                cx.listener(|this, _, _, cx| {
                                                    this.on_viewport_mouse_up_out(cx);
                                                }),
                                            ),
                                    )
                                    .children(inspector_element)
                                    .children(minimap_element)
                                    .children(table_element),
                            )
                            .child(status_bar),
                    ),
            )
            .children(resize_strips)
    }
}

