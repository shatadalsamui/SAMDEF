use crate::app::SamdefApp;
use crate::geo::{format_lat_lon, is_valid_geotransform};
use crate::model::{class_name, class_rgb, DisplayDetection};
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, DefiniteLength, IntoElement, MouseButton};

impl SamdefApp {
    pub fn render_inspector(
        &self,
        detection: DisplayDetection,
        is_pinned: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let (cx_lon, cy_lat) = detection.geo_center(&self.geo_transform);
        let (w_m, h_m) = detection.ground_dimensions_m(&self.geo_transform);
        let area_m2 = detection.ground_area_sqm(&self.geo_transform);
        let has_geo = is_valid_geotransform(&self.geo_transform);

        let [r, g, b] = class_rgb(detection.class_id);
        let hex_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let conf_pct = (detection.confidence * 100.0).round() as i32;

        div()
            .id("inspector-hud")
            .occlude()
            .absolute()
            .top(px(12.0))
            .right(px(14.0))
            .w(px(278.0))
            .p_3()
            .rounded_md()
            .bg(rgb(BG_CARD))
            .border_1()
            .border_color(rgb(if is_pinned { hex_color } else { BORDER_STRONG }))
            .shadow_lg()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .child(div().size_3().rounded_full().bg(rgb(hex_color)))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child(class_name(detection.class_id)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_DIM))
                                    .child(format!("#{}", detection.id)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_xs()
                                    .bg(rgb(if is_pinned { 0x1e3a8a } else { BG_INPUT }))
                                    .text_xs()
                                    .text_color(rgb(if is_pinned { ACCENT_BLUE } else { TEXT_DIM }))
                                    .child(if is_pinned { "PINNED" } else { "HOVER" }),
                            )
                            .children(is_pinned.then(|| {
                                div()
                                    .id("btn-close-inspector")
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_xs()
                                    .text_xs()
                                    .text_color(rgb(TEXT_SECONDARY))
                                    .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.viewer_state.selected_detection_id = None;
                                            cx.notify();
                                        }),
                                    )
                                    .child("X")
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .text_xs()
                            .child(div().text_color(rgb(TEXT_SECONDARY)).child("Confidence"))
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(ACCENT_GREEN))
                                    .child(format!("{}%", conf_pct)),
                            ),
                    )
                    .child(
                        div()
                            .h(px(4.0))
                            .w_full()
                            .rounded_full()
                            .bg(rgb(BG_INPUT))
                            .child(
                                div()
                                    .h_full()
                                    .w(DefiniteLength::Fraction(detection.confidence))
                                    .rounded_full()
                                    .bg(rgb(ACCENT_GREEN)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .rounded_sm()
                    .bg(rgb(BG_INPUT))
                    .text_xs()
                    .child(kv_row(
                        "Pixel BBox",
                        format!("{:.0} x {:.0} px", detection.width(), detection.height()),
                        TEXT_PRIMARY,
                    ))
                    .child(kv_row(
                        "Bounds [X,Y]",
                        format!(
                            "{:.0},{:.0}..{:.0},{:.0}",
                            detection.x_min, detection.y_min, detection.x_max, detection.y_max
                        ),
                        TEXT_SECONDARY,
                    ))
                    .children(has_geo.then(|| {
                        kv_row(
                            "Ground Size",
                            format!("{:.1}m x {:.1}m", w_m, h_m),
                            ACCENT_CYAN,
                        )
                    }))
                    .children(has_geo.then(|| {
                        kv_row("Footprint", format!("{:.1} m2", area_m2), ACCENT_CYAN)
                    }))
                    .children(has_geo.then(|| {
                        kv_row("Geo Center", format_lat_lon(cx_lon, cy_lat), TEXT_PRIMARY)
                    })),
            )
            .children(is_pinned.then(|| {
                div()
                    .id("btn-focus-target")
                    .w_full()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(ACCENT_BLUE))
                    .hover(|s| s.bg(rgb(0x2563eb)).cursor_pointer())
                    .text_center()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(gpui::white())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let (orig_w, orig_h) = this.image_dims();
                            this.viewer_state.focus_on_detection(
                                &detection,
                                this.vp_bounds(),
                                orig_w,
                                orig_h,
                            );
                            cx.notify();
                        }),
                    )
                    .child("Center & Zoom on Target")
            }))
    }
}
