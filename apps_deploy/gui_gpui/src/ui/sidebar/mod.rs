pub mod class_panel;
pub mod confidence_panel;
pub mod file_nav;
pub mod telemetry_export;

use crate::app::SamdefApp;
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, IntoElement};

impl SamdefApp {
    pub fn render_sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("sidebar-container")
            .w(px(336.0))
            .h_full()
            .bg(rgb(BG_SIDEBAR))
            .border_r_1()
            .border_color(rgb(BORDER_SUBTLE))
            .flex()
            .flex_col()
            .p_3()
            .gap_2p5()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .pb_2()
                    .border_b_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(div().size_2p5().rounded_xs().bg(rgb(ACCENT_BLUE)))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child("SAMDEF GIS VIEWER"),
                            )
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_xs()
                                    .bg(rgb(0x1e3a8a))
                                    .text_xs()
                                    .text_color(rgb(ACCENT_BLUE))
                                    .child("GPUI"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_DIM))
                            .child("Satellite Object Detection & Exploitation"),
                    ),
            )
            .child(self.render_file_nav(cx))
            .child(self.render_class_panel(cx))
            .child(self.render_confidence_panel(cx))
            .child(self.render_telemetry_export(cx))
    }
}

