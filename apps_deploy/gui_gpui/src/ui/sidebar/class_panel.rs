use crate::app::SamdefApp;
use crate::model::{class_name, class_rgb};
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};

impl SamdefApp {
    pub fn render_class_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let class_counts = self.class_counts();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .pt_2()
            .border_t_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_SECONDARY))
                            .child("ANNOTATION CONTROLS"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1p5()
                            .child(
                                action_btn("btn-select-all", "Select All")
                                    .flex_1()
                                    .text_center()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.class_visible = [true; 8];
                                            cx.notify();
                                        }),
                                    ),
                            )
                            .child(
                                action_btn("btn-clear-all", "Clear All")
                                    .flex_1()
                                    .text_center()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.class_visible = [false; 8];
                                            cx.notify();
                                        }),
                                    ),
                            )
                            .child(
                                action_btn("btn-invert-classes", "Invert")
                                    .flex_1()
                                    .text_center()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            for v in &mut this.class_visible {
                                                *v = !*v;
                                            }
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("btn-toggle-labels")
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(rgb(BG_CARD))
                            .border_1()
                            .border_color(rgb(BORDER_SUBTLE))
                            .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.viewer_state.show_labels = !this.viewer_state.show_labels;
                                    cx.notify();
                                }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child("Show Class & Conf Badges [L]"),
                            )
                            .child(
                                div()
                                    .size_3()
                                    .rounded_xs()
                                    .border_1()
                                    .border_color(rgb(BORDER_STRONG))
                                    .bg(rgb(if self.viewer_state.show_labels {
                                        ACCENT_BLUE
                                    } else {
                                        BG_INPUT
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pt_1p5()
                    .border_t_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_SECONDARY))
                            .child("TARGET CLASSES [1-8]")
                            .child(div().text_color(rgb(TEXT_DIM)).child("Solo")),
                    )
                    .children((0..8).map(|i| {
                        let is_active = self.class_visible[i];
                        let count = class_counts[i];
                        let [r, g, b] = class_rgb(i);
                        let hex_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

                        div()
                            .id(("class-row", i))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .bg(rgb(if is_active { BG_CARD } else { 0x131316 }))
                            .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.class_visible[i] = !this.class_visible[i];
                                            cx.notify();
                                        }),
                                    )
                                    .child(
                                        div()
                                            .size_3()
                                            .rounded_xs()
                                            .border_1()
                                            .border_color(rgb(BORDER_STRONG))
                                            .bg(rgb(if is_active { ACCENT_BLUE } else { BG_INPUT })),
                                    )
                                    .child(
                                        div()
                                            .w(px(10.0))
                                            .h(px(10.0))
                                            .rounded_xs()
                                            .bg(rgb(hex_color)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(if is_active {
                                                TEXT_PRIMARY
                                            } else {
                                                TEXT_DIM
                                            }))
                                            .child(class_name(i)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(if count > 0 {
                                                TEXT_PRIMARY
                                            } else {
                                                TEXT_DIM
                                            }))
                                            .child(format!("{}", count)),
                                    )
                                    .child(
                                        div()
                                            .id(("btn-solo", i))
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded_xs()
                                            .bg(rgb(BG_INPUT))
                                            .border_1()
                                            .border_color(rgb(BORDER_SUBTLE))
                                            .text_xs()
                                            .text_color(rgb(TEXT_DIM))
                                            .hover(|s| s.text_color(rgb(ACCENT_CYAN)).cursor_pointer())
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    if this.class_visible[i]
                                                        && this
                                                            .class_visible
                                                            .iter()
                                                            .filter(|&&v| v)
                                                            .count()
                                                            == 1
                                                    {
                                                        this.class_visible = [true; 8];
                                                    } else {
                                                        this.class_visible = [false; 8];
                                                        this.class_visible[i] = true;
                                                    }
                                                    cx.notify();
                                                }),
                                            )
                                            .child("S"),
                                    ),
                            )
                    })),
            )
    }
}

