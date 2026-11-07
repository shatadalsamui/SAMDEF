use crate::app::SamdefApp;
use crate::ui::theme::*;
use gpui::{
    canvas, div, fill, point, prelude::*, px, rgb, size, Bounds, Context, IntoElement, MouseButton,
    Pixels, Point,
};

impl SamdefApp {
    fn set_conf_from_pos(&mut self, window_pos: Point<Pixels>) {
        let sb = self.conf_slider_bounds.get();
        let sw: f32 = sb.size.width.into();
        if sw <= 1.0 {
            return;
        }
        let sx: f32 = sb.origin.x.into();
        let wx: f32 = window_pos.x.into();
        self.confidence_threshold = ((wx - sx) / sw).clamp(0.01, 0.99);
    }

    pub fn render_confidence_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let thresh = self.confidence_threshold;
        let slider_bounds_cell = self.conf_slider_bounds.clone();

        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .pt_2()
            .border_t_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .text_xs()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_SECONDARY))
                            .child("CONFIDENCE THRESHOLD"),
                    )
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(ACCENT_GREEN))
                            .child(format!("{:.0}%", thresh * 100.0)),
                    ),
            )
            // Interactive drag/click slider track
            .child(
                div()
                    .id("conf-slider-track")
                    .h(px(14.0))
                    .w_full()
                    .rounded_sm()
                    .bg(rgb(BG_INPUT))
                    .border_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .overflow_hidden()
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                slider_bounds_cell.set(bounds);
                                bounds
                            },
                            move |bounds, _, window, _| {
                                let bx: f32 = bounds.origin.x.into();
                                let by: f32 = bounds.origin.y.into();
                                let bw: f32 = bounds.size.width.into();
                                let bh: f32 = bounds.size.height.into();
                                let fill_w = (bw * thresh).clamp(2.0, bw);

                                window.paint_quad(fill(
                                    Bounds::new(point(px(bx), px(by)), size(px(fill_w), px(bh))),
                                    rgb(ACCENT_GREEN),
                                ));
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(px((bx + fill_w - 2.0).max(bx)), px(by)),
                                        size(px(3.0), px(bh)),
                                    ),
                                    gpui::white(),
                                ));
                            },
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, ev: &gpui::MouseDownEvent, _, cx| {
                            this.set_conf_from_pos(ev.position);
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, ev: &gpui::MouseMoveEvent, _, cx| {
                        if !this.viewer_state.is_dragging
                            && ev.pressed_button == Some(MouseButton::Left)
                        {
                            this.set_conf_from_pos(ev.position);
                            cx.notify();
                        }
                    })),
            )
            // Crisp ASCII stepper buttons (-5%, -1%, +1%, +5%)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(
                        [
                            ("btn-conf-m5", "-5%", -0.05_f32),
                            ("btn-conf-m1", "-1%", -0.01_f32),
                            ("btn-conf-p1", "+1%", 0.01_f32),
                            ("btn-conf-p5", "+5%", 0.05_f32),
                        ]
                        .into_iter()
                        .map(|(id, label, delta)| {
                            action_btn(id, label)
                                .flex_1()
                                .text_center()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.confidence_threshold =
                                            (this.confidence_threshold + delta).clamp(0.01, 0.99);
                                        cx.notify();
                                    }),
                                )
                        }),
                    ),
            )
            // Presets
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children([0.10, 0.20, 0.35, 0.50, 0.75, 0.90].into_iter().map(|val| {
                        let is_curr = (thresh - val).abs() < 0.02;
                        div()
                            .id(("conf-preset", (val * 100.0) as usize))
                            .flex_1()
                            .py_0p5()
                            .rounded_xs()
                            .bg(rgb(if is_curr { ACCENT_GREEN } else { BG_INPUT }))
                            .text_center()
                            .text_xs()
                            .text_color(rgb(if is_curr { 0x000000 } else { TEXT_DIM }))
                            .hover(|s| s.cursor_pointer())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.confidence_threshold = val;
                                    cx.notify();
                                }),
                            )
                            .child(format!("{:.0}%", val * 100.0))
                    })),
            )
    }
}
