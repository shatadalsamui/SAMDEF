use crate::app::SamdefApp;
use crate::geo::{
    estimate_gsd_meters, format_dms, format_lat_lon, is_valid_geotransform, pixel_to_geo,
};
use crate::ui::theme::*;
use crate::viewer::format_ruler_measurement;
use gpui::{div, prelude::*, px, rgb, IntoElement};

impl SamdefApp {
    /// Bottom QGIS-style GIS status bar showing live cursor coordinates, GSD, and ruler telemetry
    pub fn render_status_bar(&self) -> impl IntoElement {
        let has_geo = is_valid_geotransform(&self.geo_transform);
        let gsd = estimate_gsd_meters(&self.geo_transform);
        let visible = self.count_visible();
        let total = self.detections.len();

        let (px_str, geo_str, dms_str) = match self.viewer_state.cursor_img_pixel {
            Some((x, y)) => {
                let p_s = format!("X: {:.0}  Y: {:.0} px", x, y);
                if has_geo {
                    let (lon, lat) = pixel_to_geo(&self.geo_transform, x as f64, y as f64);
                    (p_s, format_lat_lon(lon, lat), format_dms(lon, lat))
                } else {
                    (p_s, "Non-Georeferenced".to_string(), String::new())
                }
            }
            None => (
                "Cursor Outside Scene".to_string(),
                "--".to_string(),
                String::new(),
            ),
        };

        let ruler_text = if self.viewer_state.is_measure_mode
            && let (Some(s), Some(e)) = (
                self.viewer_state.measure_start,
                self.viewer_state.measure_current,
            ) {
            Some(format_ruler_measurement(s, e, &self.geo_transform))
        } else {
            None
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(26.0))
            .px_3()
            .bg(rgb(BG_SIDEBAR))
            .border_t_1()
            .border_color(rgb(BORDER_SUBTLE))
            .text_xs()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(div().text_color(rgb(TEXT_PRIMARY)).child(px_str))
                    .child(div().text_color(rgb(BORDER_STRONG)).child("|"))
                    .child(div().text_color(rgb(ACCENT_CYAN)).child(geo_str))
                    .children((!dms_str.is_empty()).then(|| {
                        div()
                            .text_color(rgb(TEXT_DIM))
                            .child(format!("({})", dms_str))
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .children(ruler_text.map(|r| {
                        div()
                            .px_1p5()
                            .rounded_xs()
                            .bg(rgb(0x0e3b43))
                            .text_color(rgb(ACCENT_CYAN))
                            .child(format!("Ruler: {}", r))
                    }))
                    .children(has_geo.then(|| {
                        div()
                            .text_color(rgb(ACCENT_AMBER))
                            .child(format!("GSD {:.2} m/px", gsd))
                    }))
                    .child(
                        div()
                            .text_color(rgb(ACCENT_GREEN))
                            .child(format!("Targets: {} / {}", visible, total)),
                    )
                    .child(
                        div()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(format!("Zoom: {:.0}%", self.viewer_state.scale * 100.0)),
                    ),
            )
    }
}

