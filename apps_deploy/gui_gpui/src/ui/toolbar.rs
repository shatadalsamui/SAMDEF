use crate::app::SamdefApp;
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton, SharedString};

impl SamdefApp {
    pub fn render_toolbar(&self, cx: &Context<Self>) -> impl IntoElement {
        let zoom_pct = (self.viewer_state.scale * 100.0).round() as i32;
        let visible_count = self.count_visible();

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(38.0))
            .px_3()
            .bg(rgb(BG_SIDEBAR))
            .border_b_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1p5()
                    .child(action_btn("btn-reset", "Fit [R]").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.viewer_state.reset_view();
                            cx.notify();
                        }),
                    ))
                    .child(action_btn("btn-1to1", "1:1 [0]").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            let (w, h) = this.image_dims();
                            this.viewer_state.set_native_1_to_1(this.vp_bounds(), w, h);
                            cx.notify();
                        }),
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .rounded_sm()
                            .bg(rgb(BG_CARD))
                            .border_1()
                            .border_color(rgb(BORDER_SUBTLE))
                            .child(
                                div()
                                    .id("btn-zoom-out")
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let (w, h) = this.image_dims();
                                            this.viewer_state
                                                .zoom_at_center(0.80, this.vp_bounds(), w, h);
                                            cx.notify();
                                        }),
                                    )
                                    .child("-"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(ACCENT_CYAN))
                                    .border_l_1()
                                    .border_r_1()
                                    .border_color(rgb(BORDER_SUBTLE))
                                    .child(format!("{}%", zoom_pct)),
                            )
                            .child(
                                div()
                                    .id("btn-zoom-in")
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let (w, h) = this.image_dims();
                                            this.viewer_state
                                                .zoom_at_center(1.25, this.vp_bounds(), w, h);
                                            cx.notify();
                                        }),
                                    )
                                    .child("+"),
                            ),
                    )
                    .child(
                        toggle_btn(
                            "btn-ruler",
                            "Ruler [M]",
                            self.viewer_state.is_measure_mode,
                            ACCENT_CYAN,
                            0x0e3b43,
                            ACCENT_CYAN,
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.viewer_state.is_measure_mode =
                                    !this.viewer_state.is_measure_mode;
                                this.viewer_state.measure_start = None;
                                this.viewer_state.measure_current = None;
                                this.viewer_state.is_measure_pinned = false;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        toggle_btn(
                            "btn-bw",
                            "B/W [G]",
                            self.viewer_state.is_grayscale,
                            ACCENT_AMBER,
                            0x3b2d0e,
                            ACCENT_AMBER,
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.viewer_state.is_grayscale = !this.viewer_state.is_grayscale;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        toggle_btn(
                            "btn-minimap",
                            "Radar",
                            self.viewer_state.show_minimap,
                            ACCENT_BLUE,
                            0x132742,
                            ACCENT_BLUE,
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.viewer_state.show_minimap = !this.viewer_state.show_minimap;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        toggle_btn(
                            "btn-table",
                            format!("Table [T] ({})", visible_count),
                            self.show_table_drawer,
                            ACCENT_GREEN,
                            0x0d3326,
                            ACCENT_GREEN,
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.show_table_drawer = !this.show_table_drawer;
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .text_xs()
                    .child(if self.is_loading {
                        div()
                            .text_color(rgb(ACCENT_AMBER))
                            .child("Loading GeoTIFF...")
                    } else {
                        div()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(SharedString::from(self.status_message.clone()))
                    }),
            )
    }
}
