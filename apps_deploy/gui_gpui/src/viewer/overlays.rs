use crate::geo::{
    calculate_bearing_degrees, calculate_polygon_metrics, haversine_distance_meters,
    is_valid_geotransform, pixel_to_geo, GeoTransform,
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
            paint_crosshair(window, cx_pt, cy_pt, crosshair_color);
        }
    });

    let label = format_ruler_measurement(start_px, end_px, geo_transform);
    let _ = (vx, vy);
    paint_measurement_pill(
        window,
        cx,
        viewport,
        (sx1 + sx2) * 0.5,
        (sy1 + sy2) * 0.5,
        label,
        0x67e8f9,
        0x06b6d4,
    );
}

/// Layer 6: Interactive multi-vertex geodesic polygon footprint fill, outline, crosshairs, and area pill
pub fn paint_footprint_layers(
    window: &mut Window,
    cx: &mut App,
    viewport: Bounds<Pixels>,
    orig_w: f32,
    orig_h: f32,
    geo_transform: &GeoTransform,
    state: &ViewerState,
) {
    if !state.is_footprint_mode {
        return;
    }
    let pts = state.active_footprint_points();
    if pts.is_empty() {
        return;
    }

    let screen_pts: Vec<(f32, f32)> = pts
        .iter()
        .map(|&(px_x, px_y)| state.pixel_to_screen(px_x, px_y, viewport, orig_w, orig_h))
        .collect();

    window.paint_layer(viewport, |window| {
        if screen_pts.len() >= 3 {
            let mut fill_builder = PathBuilder::fill();
            fill_builder.move_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
            for &(sx, sy) in &screen_pts[1..] {
                fill_builder.line_to(point(px(sx), px(sy)));
            }
            fill_builder.close();
            if let Ok(path) = fill_builder.build() {
                window.paint_path(path, rgba(0xf59e0b2e));
            }
        }

        if screen_pts.len() >= 2 {
            let mut stroke_builder = PathBuilder::stroke(px(2.0)).dash_array(&[px(6.0), px(3.0)]);
            stroke_builder.move_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
            for &(sx, sy) in &screen_pts[1..] {
                stroke_builder.line_to(point(px(sx), px(sy)));
            }
            if screen_pts.len() >= 3 {
                stroke_builder.close();
            }
            if let Ok(path) = stroke_builder.build() {
                window.paint_path(path, rgb(0xf59e0b));
            }
        }

        let crosshair_color: Background = rgb(0xfbbf24).into();
        for &(px_x, px_y) in &state.footprint_points {
            let (sx, sy) = state.pixel_to_screen(px_x, px_y, viewport, orig_w, orig_h);
            paint_crosshair(window, sx, sy, crosshair_color);
        }

        if !state.is_footprint_pinned && screen_pts.len() >= 3 {
            window.paint_quad(PaintQuad {
                bounds: Bounds::new(
                    point(px(screen_pts[0].0 - 6.0), px(screen_pts[0].1 - 6.0)),
                    size(px(12.0), px(12.0)),
                ),
                corner_radii: (2.).into(),
                background: gpui::transparent_black().into(),
                border_widths: Edges::all(px(1.5)),
                border_color: rgb(0xfde68a).into(),
                border_style: BorderStyle::Solid,
            });
        }
    });

    let inv_n = 1.0 / screen_pts.len() as f32;
    let anchor_x = screen_pts.iter().map(|p| p.0).sum::<f32>() * inv_n;
    let anchor_y = screen_pts.iter().map(|p| p.1).sum::<f32>() * inv_n;
    let label = format_footprint_measurement(&pts, state.is_footprint_pinned, geo_transform);
    paint_measurement_pill(window, cx, viewport, anchor_x, anchor_y, label, 0xfde68a, 0xf59e0b);
}

#[inline]
fn paint_crosshair(window: &mut Window, cx_pt: f32, cy_pt: f32, color: Background) {
    window.paint_quad(fill(
        Bounds::new(point(px(cx_pt - 5.0), px(cy_pt - 1.0)), size(px(10.0), px(2.0))),
        color,
    ));
    window.paint_quad(fill(
        Bounds::new(point(px(cx_pt - 1.0), px(cy_pt - 5.0)), size(px(2.0), px(10.0))),
        color,
    ));
}

#[allow(clippy::too_many_arguments)]
fn paint_measurement_pill(
    window: &mut Window,
    cx: &mut App,
    viewport: Bounds<Pixels>,
    anchor_x: f32,
    anchor_y: f32,
    label: String,
    text_hex: u32,
    border_hex: u32,
) {
    let vx: f32 = viewport.origin.x.into();
    let vy: f32 = viewport.origin.y.into();
    let vw: f32 = viewport.size.width.into();
    let vh: f32 = viewport.size.height.into();
    let font = window.text_style().font();

    window.paint_layer(viewport, |window| {
        let shaped = window.text_system().shape_line(
            SharedString::from(label.clone()),
            px(11.0),
            &[TextRun {
                len: label.len(),
                font,
                color: rgb(text_hex).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );

        let text_w: f32 = shaped.width.into();
        let pill_w = text_w + 14.0;
        let pill_h = 20.0;
        let mid_x = (anchor_x - pill_w * 0.5)
            .clamp(vx + 4.0, (vx + vw - pill_w - 4.0).max(vx + 4.0));
        let mid_y = (anchor_y - pill_h - 6.0)
            .clamp(vy + 4.0, (vy + vh - pill_h - 4.0).max(vy + 4.0));

        window.paint_quad(PaintQuad {
            bounds: Bounds::new(point(px(mid_x), px(mid_y)), size(px(pill_w), px(pill_h))),
            corner_radii: (4.).into(),
            background: rgba(0x0f172aee).into(),
            border_widths: Edges::all(px(1.2)),
            border_color: rgb(border_hex).into(),
            border_style: BorderStyle::Solid,
        });

        let _ = shaped.paint(point(px(mid_x + 7.0), px(mid_y + 2.0)), px(16.0), window, cx);
    });
}

pub fn format_footprint_measurement(
    points: &[(f32, f32)],
    is_pinned: bool,
    gt: &GeoTransform,
) -> String {
    match points.len() {
        0 => "Click vertices to trace footprint".to_string(),
        1 => "Click next corner (Right-click clears)".to_string(),
        2 => {
            let seg = format_ruler_measurement(points[0], points[1], gt);
            format!("{} | Click 3rd+ pt", seg)
        }
        n => {
            if let Some(m) = calculate_polygon_metrics(points, gt) {
                let state_tag = if is_pinned { "" } else { " | DblClick/Enter close" };
                if is_valid_geotransform(gt) {
                    let area_str = if m.area_sqm >= 1_000_000.0 {
                        format!("{:.3} km2", m.area_sqm / 1_000_000.0)
                    } else {
                        format!("{:.1} m2", m.area_sqm)
                    };
                    let perim_str = if m.perimeter_m >= 1000.0 {
                        format!("{:.2} km", m.perimeter_m / 1000.0)
                    } else {
                        format!("{:.1} m", m.perimeter_m)
                    };
                    format!("Area {} | Perim {} ({} pts){}", area_str, perim_str, n, state_tag)
                } else {
                    format!(
                        "Area {:.0} px2 | Perim {:.0} px ({} pts){}",
                        m.pixel_area, m.pixel_perimeter, n, state_tag
                    )
                }
            } else {
                format!("{} pts", n)
            }
        }
    }
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

