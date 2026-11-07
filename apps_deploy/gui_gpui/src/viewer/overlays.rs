use crate::geo::{
    calculate_bearing_degrees, haversine_distance_meters, is_valid_geotransform, pixel_to_geo,
    GeoTransform,
};
use crate::model::{class_hsla, class_name};
use crate::viewer::state::ViewerState;
use gpui::{
    fill, point, px, rgb, rgba, size, App, Background, BorderStyle, Bounds, Edges, PaintQuad,
    PathBuilder, Pixels, SharedString, TextRun, Window,
};

pub struct BadgeItem {
    pub x: f32,
    pub y: f32,
    pub class_id: usize,
    pub confidence: f32,
    pub is_highlighted: bool,
}

/// Layer 3: Magnified class & confidence pill badges with GPU-shaped glyphs
pub fn paint_badges_layer(
    window: &mut Window,
    cx: &mut App,
    viewport: Bounds<Pixels>,
    badges: &[BadgeItem],
) {
    if badges.is_empty() {
        return;
    }

    let vx: f32 = viewport.origin.x.into();
    let vy: f32 = viewport.origin.y.into();
    let vw: f32 = viewport.size.width.into();
    let vh: f32 = viewport.size.height.into();
    let font = window.text_style().font();

    window.paint_layer(viewport, |window| {
        for item in badges {
            let label = format!(
                "{} {:.0}%",
                class_name(item.class_id),
                item.confidence * 100.0
            );
            let text_color = if item.is_highlighted {
                gpui::white()
            } else {
                class_hsla(item.class_id, 1.0)
            };

            let shaped = window.text_system().shape_line(
                SharedString::from(label.clone()),
                px(10.5),
                &[TextRun {
                    len: label.len(),
                    font: font.clone(),
                    color: text_color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );

            let text_w: f32 = shaped.width.into();
            let tag_w = text_w + 10.0;
            let tag_h = 16.0;

            let max_tag_x = (vx + vw - tag_w - 2.0).max(vx + 2.0);
            let tag_x = item.x.clamp(vx + 2.0, max_tag_x);
            let tag_y = if item.y - tag_h - 2.0 >= vy + 2.0 {
                item.y - tag_h - 2.0
            } else {
                (item.y + 2.0).min(vy + vh - tag_h - 2.0)
            };

            let bg_color: Background = if item.is_highlighted {
                rgba(0x09090bee).into()
            } else {
                rgba(0x09090bd8).into()
            };

            window.paint_quad(PaintQuad {
                bounds: Bounds::new(point(px(tag_x), px(tag_y)), size(px(tag_w), px(tag_h))),
                corner_radii: (3.).into(),
                background: bg_color,
                border_widths: Edges::all(px(1.0)),
                border_color: class_hsla(item.class_id, 0.95),
                border_style: BorderStyle::Solid,
            });

            let _ = shaped.paint(point(px(tag_x + 5.0), px(tag_y + 1.0)), px(14.0), window, cx);
        }
    });
}

/// Layer 4 & Layer 5: Geodesic ruler dashed vector line, crosshairs, and Haversine readout pill
pub fn paint_ruler_layers(
    window: &mut Window,
    cx: &mut App,
    viewport: Bounds<Pixels>,
    orig_w: f32,
    orig_h: f32,
    geo_transform: &GeoTransform,
    state: &ViewerState,
) {
    if !state.is_measure_mode {
        return;
    }
    let (Some(start_px), Some(end_px)) = (state.measure_start, state.measure_current) else {
        return;
    };

    let vx: f32 = viewport.origin.x.into();
    let vy: f32 = viewport.origin.y.into();
    let vw: f32 = viewport.size.width.into();
    let vh: f32 = viewport.size.height.into();
    let (sx1, sy1) = state.pixel_to_screen(start_px.0, start_px.1, viewport, orig_w, orig_h);
    let (sx2, sy2) = state.pixel_to_screen(end_px.0, end_px.1, viewport, orig_w, orig_h);

    window.paint_layer(viewport, |window| {
        let mut builder = PathBuilder::stroke(px(2.0)).dash_array(&[px(6.0), px(3.0)]);
        builder.move_to(point(px(sx1), px(sy1)));
        builder.line_to(point(px(sx2), px(sy2)));
        if let Ok(path) = builder.build() {
            window.paint_path(path, rgb(0x06b6d4));
        }

        let crosshair_color: Background = rgb(0x22d3ee).into();
        for (cx_pt, cy_pt) in [(sx1, sy1), (sx2, sy2)] {
            window.paint_quad(fill(
                Bounds::new(point(px(cx_pt - 5.0), px(cy_pt - 1.0)), size(px(10.0), px(2.0))),
                crosshair_color,
            ));
            window.paint_quad(fill(
                Bounds::new(point(px(cx_pt - 1.0), px(cy_pt - 5.0)), size(px(2.0), px(10.0))),
                crosshair_color,
            ));
        }
    });

    let label = format_ruler_measurement(start_px, end_px, geo_transform);
    let font = window.text_style().font();
    window.paint_layer(viewport, |window| {
        let shaped = window.text_system().shape_line(
            SharedString::from(label.clone()),
            px(11.0),
            &[TextRun {
                len: label.len(),
                font,
                color: rgb(0x67e8f9).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );

        let text_w: f32 = shaped.width.into();
        let pill_w = text_w + 14.0;
        let pill_h = 20.0;
        let mid_x = ((sx1 + sx2) * 0.5 - pill_w * 0.5)
            .clamp(vx + 4.0, (vx + vw - pill_w - 4.0).max(vx + 4.0));
        let mid_y = ((sy1 + sy2) * 0.5 - pill_h - 6.0)
            .clamp(vy + 4.0, (vy + vh - pill_h - 4.0).max(vy + 4.0));

        window.paint_quad(PaintQuad {
            bounds: Bounds::new(point(px(mid_x), px(mid_y)), size(px(pill_w), px(pill_h))),
            corner_radii: (4.).into(),
            background: rgba(0x0f172aee).into(),
            border_widths: Edges::all(px(1.2)),
            border_color: rgb(0x06b6d4).into(),
            border_style: BorderStyle::Solid,
        });

        let _ = shaped.paint(point(px(mid_x + 7.0), px(mid_y + 2.0)), px(16.0), window, cx);
    });
}

pub fn format_ruler_measurement(
    start_px: (f32, f32),
    end_px: (f32, f32),
    gt: &GeoTransform,
) -> String {
    let dx = end_px.0 - start_px.0;
    let dy = end_px.1 - start_px.1;
    let px_dist = dx.hypot(dy);

    if is_valid_geotransform(gt) {
        let (lon1, lat1) = pixel_to_geo(gt, start_px.0 as f64, start_px.1 as f64);
        let (lon2, lat2) = pixel_to_geo(gt, end_px.0 as f64, end_px.1 as f64);
        let dist_m = haversine_distance_meters(lon1, lat1, lon2, lat2);
        let bearing = calculate_bearing_degrees(lon1, lat1, lon2, lat2);
        if dist_m >= 1000.0 {
            format!("{:.2} km | {:.1}° ({:.0} px)", dist_m / 1000.0, bearing, px_dist)
        } else {
            format!("{:.1} m | {:.1}° ({:.0} px)", dist_m, bearing, px_dist)
        }
    } else {
        let angle_deg = (dy.atan2(dx).to_degrees() + 90.0 + 360.0) % 360.0;
        format!("{:.1} px | {:.1}°", px_dist, angle_deg)
    }
}

