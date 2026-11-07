use crate::app::SamdefApp;
use crate::geo::{
    estimate_gsd_meters, format_dms, format_lat_lon, is_valid_geotransform, pixel_to_geo,
};
use crate::io::{export_csv, export_geojson, export_summary_report};
use crate::ui::theme::*;
use gpui::{div, prelude::*, rgb, Context, IntoElement, MouseButton};
use std::path::PathBuf;

impl SamdefApp {
    pub fn render_telemetry_export(&self, cx: &Context<Self>) -> impl IntoElement {
        let has_geo = is_valid_geotransform(&self.geo_transform);
        let gsd = estimate_gsd_meters(&self.geo_transform);
        let visible_count = self.count_visible();
        let total_count = self.detections.len();
        let img_dims = self
            .current_image
            .as_ref()
            .map(|i| (i.width, i.height))
            .unwrap_or((0, 0));

        let (cursor_lon, cursor_lat) = match self.viewer_state.cursor_img_pixel {
            Some((px, py)) if has_geo => pixel_to_geo(&self.geo_transform, px as f64, py as f64),
            _ => (0.0, 0.0),
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .pt_2()
            .border_t_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_SECONDARY))
                    .child("GEOGRAPHIC & TELEMETRY HUD"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .rounded_sm()
                    .bg(rgb(BG_CARD))
                    .border_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .text_xs()
                    .child(kv_row(
                        "Scene Size",
                        format!("{} x {} px", img_dims.0, img_dims.1),
                        TEXT_PRIMARY,
                    ))
                    .child(kv_row(
                        "Active Targets",
                        format!("{} / {}", visible_count, total_count),
                        ACCENT_GREEN,
                    ))
                    .child(kv_row(
                        "Magnification",
                        format!("{:.0}%", self.viewer_state.scale * 100.0),
                        ACCENT_CYAN,
                    ))
                    .child(kv_row(
                        "Cursor Pixel",
                        match self.viewer_state.cursor_img_pixel {
                            Some((x, y)) => format!("X:{:.0} Y:{:.0}", x, y),
                            None => "Outside Image".to_string(),
                        },
                        TEXT_PRIMARY,
                    ))
                    .children(has_geo.then(|| {
                        kv_row(
                            "WGS-84 Lon/Lat",
                            if self.viewer_state.cursor_img_pixel.is_some() {
                                format_lat_lon(cursor_lon, cursor_lat)
                            } else {
                                "Outside Bounds".to_string()
                            },
                            TEXT_PRIMARY,
                        )
                    }))
                    .children(
                        (has_geo && self.viewer_state.cursor_img_pixel.is_some()).then(|| {
                            kv_row(
                                "DMS Grid",
                                format_dms(cursor_lon, cursor_lat),
                                TEXT_SECONDARY,
                            )
                        }),
                    )
                    .children(has_geo.then(|| {
                        kv_row("Est. GSD", format!("{:.2} m/px", gsd), ACCENT_AMBER)
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_SECONDARY))
                    .child("EXPORT INTELLIGENCE PRODUCTS"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1p5()
                    .child(
                        action_btn("btn-export-geojson", "GeoJSON")
                            .flex_1()
                            .text_center()
                            .text_color(rgb(ACCENT_CYAN))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let path = PathBuf::from("samdef_export.geojson");
                                    let name = this
                                        .current_json_path
                                        .as_ref()
                                        .and_then(|p| p.file_stem())
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("scene");
                                    this.status_message = match export_geojson(
                                        &path,
                                        &this.detections,
                                        &this.geo_transform,
                                        name,
                                    ) {
                                        Ok(()) => format!("Exported {}", path.display()),
                                        Err(e) => format!("Export error: {}", e),
                                    };
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        action_btn("btn-export-csv", "CSV Table")
                            .flex_1()
                            .text_center()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let path = PathBuf::from("samdef_detections.csv");
                                    this.status_message = match export_csv(
                                        &path,
                                        &this.detections,
                                        &this.geo_transform,
                                    ) {
                                        Ok(()) => format!("Exported {}", path.display()),
                                        Err(e) => format!("Export error: {}", e),
                                    };
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        action_btn("btn-export-report", "Report")
                            .flex_1()
                            .text_center()
                            .text_color(rgb(ACCENT_GREEN))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let path = PathBuf::from("samdef_intelligence_report.md");
                                    let name = this
                                        .current_json_path
                                        .as_ref()
                                        .and_then(|p| p.file_stem())
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("scene");
                                    let vis_cnt = this.count_visible();
                                    this.status_message = match export_summary_report(
                                        &path,
                                        &this.detections,
                                        &this.geo_transform,
                                        name,
                                        vis_cnt,
                                        this.confidence_threshold,
                                    ) {
                                        Ok(()) => format!("Exported {}", path.display()),
                                        Err(e) => format!("Export error: {}", e),
                                    };
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }
}

