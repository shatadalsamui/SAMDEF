use crate::app::{ActiveInput, SamdefApp};
use crate::geo::is_valid_geotransform;
use crate::io::{export_csv, export_geojson};
use crate::model::{class_name, class_rgb, DisplayDetection};
use crate::ui::theme::*;
use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use std::path::PathBuf;

impl SamdefApp {
    pub fn render_table_drawer(&self, cx: &Context<Self>) -> impl IntoElement {
        let has_geo = is_valid_geotransform(&self.geo_transform);
        let search_lower = self.table_search_query.trim().to_lowercase();

        let visible_dets: Vec<DisplayDetection> = self
            .detections
            .iter()
            .filter(|d| {
                if d.class_id >= 8
                    || !self.class_visible[d.class_id]
                    || d.confidence < self.confidence_threshold
                {
                    return false;
                }
                if !search_lower.is_empty() {
                    let name = class_name(d.class_id).to_lowercase();
                    if !name.contains(&search_lower)
                        && !format!("{}", d.id).contains(&search_lower)
                    {
                        return false;
                    }
                }
                true
            })
            .copied()
            .collect();

        let total_matches = visible_dets.len();
        let selected_id = self.viewer_state.selected_detection_id;
        let gt = self.geo_transform;

        div()
            .id("table-drawer-panel")
            .occlude()
            .absolute()
            .bottom(px(0.0))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(240.0))
            .bg(rgb(BG_SIDEBAR))
            .border_t_1()
            .border_color(rgb(BORDER_STRONG))
            .shadow_lg()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(34.0))
                    .px_4()
                    .bg(rgb(BG_CARD))
                    .border_b_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child("TARGET EXPLOITATION TABLE"),
                            )
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_xs()
                                    .bg(rgb(BG_INPUT))
                                    .text_xs()
                                    .text_color(rgb(ACCENT_GREEN))
                                    .child(format!("{} visible", total_matches)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                text_input_box(
                                    "input-table-search",
                                    &self.table_search_query,
                                    "Filter class or #ID...",
                                    self.active_input == Some(ActiveInput::TableSearch),
                                    ACCENT_CYAN,
                                )
                                .w(px(165.0))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.active_input = Some(ActiveInput::TableSearch);
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(action_btn("btn-tbl-geojson", "Export GeoJSON").on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let path = PathBuf::from("samdef_export.geojson");
                                    let name = this
                                        .current_json_path
                                        .as_ref()
                                        .and_then(|p| p.file_stem())
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("scene");
                                    let _ = export_geojson(
                                        &path,
                                        &this.detections,
                                        &this.geo_transform,
                                        name,
                                    );
                                    this.status_message = format!("Exported {}", path.display());
                                    cx.notify();
                                }),
                            ))
                            .child(action_btn("btn-tbl-csv", "Export CSV").on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let path = PathBuf::from("samdef_detections.csv");
                                    let _ =
                                        export_csv(&path, &this.detections, &this.geo_transform);
                                    this.status_message = format!("Exported {}", path.display());
                                    cx.notify();
                                }),
                            ))
                            .child(action_btn("btn-close-table", "Close [T]").on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.show_table_drawer = false;
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(24.0))
                    .px_4()
                    .bg(rgb(BG_INPUT))
                    .border_b_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_DIM))
                    .child(div().w(px(60.0)).child("ID"))
                    .child(div().w(px(160.0)).child("TARGET CLASS"))
                    .child(div().w(px(90.0)).child("CONFIDENCE"))
                    .child(div().w(px(120.0)).child("PIXEL SIZE"))
                    .child(div().w(px(195.0)).child("GEOGRAPHIC (LON, LAT)"))
                    .child(div().w(px(105.0)).child("FOOTPRINT"))
                    .child(div().w(px(70.0)).text_right().child("FOCUS")),
            )
            .child(
                div()
                    .id("table-body-scroll")
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .flex_1()
                    .children(visible_dets.into_iter().map(|d| {
                        let is_sel = selected_id == Some(d.id);
                        let [r, g, b] = class_rgb(d.class_id);
                        let hex_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                        let (lon, lat) = d.geo_center(&gt);
                        let area = d.ground_area_sqm(&gt);

                        div()
                            .id(("row-det", d.id))
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(26.0))
                            .px_4()
                            .border_b_1()
                            .border_color(rgb(BORDER_SUBTLE))
                            .bg(rgb(if is_sel { 0x1e293b } else { BG_SIDEBAR }))
                            .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
                            .text_xs()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let (orig_w, orig_h) = this.image_dims();
                                    this.viewer_state.focus_on_detection(
                                        &d,
                                        this.vp_bounds(),
                                        orig_w,
                                        orig_h,
                                    );
                                    cx.notify();
                                }),
                            )
                            .child(div().w(px(60.0)).text_color(rgb(TEXT_DIM)).child(format!("#{}", d.id)))
                            .child(
                                div()
                                    .w(px(160.0))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(div().size_2p5().rounded_full().bg(rgb(hex_color)))
                                    .child(div().text_color(rgb(TEXT_PRIMARY)).child(class_name(d.class_id))),
                            )
                            .child(
                                div()
                                    .w(px(90.0))
                                    .text_color(rgb(ACCENT_GREEN))
                                    .child(format!("{:.1}%", d.confidence * 100.0)),
                            )
                            .child(
                                div()
                                    .w(px(120.0))
                                    .text_color(rgb(TEXT_SECONDARY))
                                    .child(format!("{:.0}x{:.0} px", d.width(), d.height())),
                            )
                            .child(
                                div()
                                    .w(px(195.0))
                                    .text_color(rgb(TEXT_SECONDARY))
                                    .child(if has_geo {
                                        format!("{:.5}, {:.5}", lon, lat)
                                    } else {
                                        "N/A".to_string()
                                    }),
                            )
                            .child(
                                div()
                                    .w(px(105.0))
                                    .text_color(rgb(ACCENT_CYAN))
                                    .child(if has_geo {
                                        format!("{:.1} m2", area)
                                    } else {
                                        format!("{:.0} px2", d.area())
                                    }),
                            )
                            .child(
                                div()
                                    .w(px(70.0))
                                    .text_right()
                                    .text_color(rgb(ACCENT_BLUE))
                                    .child("Focus ->"),
                            )
                    })),
            )
    }
}
