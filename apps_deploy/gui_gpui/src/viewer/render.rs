use crate::geo::GeoTransform;
use crate::io::LoadedImage;
use crate::model::{class_hsla, DisplayDetection, CLASS_COUNT};
use crate::viewer::overlays::{paint_badges_layer, paint_ruler_layers, BadgeItem};
use crate::viewer::state::ViewerState;
use gpui::{
    fill, point, px, rgba, size, App, Background, BorderStyle, Bounds, Corners, Edges, PaintQuad,
    Pixels, Window,
};

/// Paint all visual layers for the SAMDEF GIS viewer using strictly ordered GPU layers:
/// - Layer 1: Satellite GeoTIFF base raster (`PolychromeSprite`)
/// - Layer 2: Dynamic hollow neon vector bounding boxes & targeting brackets (`Quad` batch)
/// - Layer 3..5: Delegated to `overlays.rs` (Badges + Shaped Text + Geodesic Ruler)
#[allow(clippy::too_many_arguments)]
pub fn paint_viewer(
    window: &mut Window,
    cx: &mut App,
    viewport: Bounds<Pixels>,
    loaded_image: Option<&LoadedImage>,
    detections: &[DisplayDetection],
    class_visible: &[bool; CLASS_COUNT],
    confidence_threshold: f32,
    geo_transform: &GeoTransform,
    state: &ViewerState,
) {
    let vw: f32 = viewport.size.width.into();
    let vh: f32 = viewport.size.height.into();
    if vw <= 1.0 || vh <= 1.0 {
        return;
    }

    let Some(loaded_img) = loaded_image else {
        return;
    };

    let orig_w = loaded_img.width as f32;
    let orig_h = loaded_img.height as f32;
    if orig_w <= 0.0 || orig_h <= 0.0 {
        return;
    }

    let (img_x, img_y, img_w, img_h) = state.calc_image_rect(viewport, orig_w, orig_h);
    let scale_x = img_w / orig_w;
    let scale_y = img_h / orig_h;
    let vx: f32 = viewport.origin.x.into();
    let vy: f32 = viewport.origin.y.into();

    // =========================================================
    // LAYER 1: Base Satellite Raster Texture (Dedicated Layer)
    // =========================================================
    window.paint_layer(viewport, |window| {
        let img_bounds = Bounds::new(point(px(img_x), px(img_y)), size(px(img_w), px(img_h)));
        let _ = window.paint_image(
            img_bounds,
            Corners::default(),
            loaded_img.render_image.clone(),
            0,
            state.is_grayscale,
        );

        window.paint_quad(PaintQuad {
            bounds: img_bounds,
            corner_radii: (0.).into(),
            background: gpui::transparent_black().into(),
            border_widths: Edges::all(px(1.0)),
            border_color: rgba(0xffffff22).into(),
            border_style: BorderStyle::Solid,
        });
    });

    // =========================================================
    // LAYER 2: Dynamic Neon Vector Bounding Boxes & Brackets
    // =========================================================
    let base_thickness: f32 = if state.scale <= 1.15 {
        1.25
    } else if state.scale < 2.5 {
        1.55
    } else if state.scale < 6.0 {
        1.85
    } else {
        2.20
    };

    let show_auto_badges = state.show_labels && (state.scale >= 2.0 || scale_x >= 0.45);
    let mut badges: Vec<BadgeItem> = Vec::new();

    window.paint_layer(viewport, |window| {
        for det in detections {
            if det.class_id >= CLASS_COUNT
                || !class_visible[det.class_id]
                || det.confidence < confidence_threshold
            {
                continue;
            }

            let box_x = img_x + (det.x_min * scale_x);
            let box_y = img_y + (det.y_min * scale_y);
            let box_w = (det.x_max - det.x_min) * scale_x;
            let box_h = (det.y_max - det.y_min) * scale_y;

            if box_x + box_w < vx || box_x > vx + vw || box_y + box_h < vy || box_y > vy + vh {
                continue;
            }

            let is_selected = state.selected_detection_id == Some(det.id);
            let is_hovered = state.hovered_detection_id == Some(det.id);

            if !is_selected && !is_hovered && (box_w < 3.0 || box_h < 3.0) {
                window.paint_quad(fill(
                    Bounds::new(
                        point(px(box_x), px(box_y)),
                        size(px(box_w.max(2.5)), px(box_h.max(2.5))),
                    ),
                    class_hsla(det.class_id, 0.88),
                ));
                continue;
            }

            let draw_w = box_w.max(3.5);
            let draw_h = box_h.max(3.5);

            let (thickness, stroke_alpha, fill_alpha) = if is_selected {
                (2.6_f32, 1.0_f32, 0.20_f32)
            } else if is_hovered {
                (2.2_f32, 1.0_f32, 0.14_f32)
            } else {
                let max_hollow_t = (draw_w.min(draw_h) * 0.30).max(1.0);
                (base_thickness.min(max_hollow_t), 0.92_f32, 0.0_f32)
            };

            let stroke_color = class_hsla(det.class_id, stroke_alpha);
            window.paint_quad(PaintQuad {
                bounds: Bounds::new(point(px(box_x), px(box_y)), size(px(draw_w), px(draw_h))),
                corner_radii: (0.).into(),
                background: class_hsla(det.class_id, fill_alpha).into(),
                border_widths: Edges::all(px(thickness)),
                border_color: stroke_color,
                border_style: BorderStyle::Solid,
            });

            if is_selected || is_hovered {
                paint_corner_brackets(
                    window,
                    box_x,
                    box_y,
                    draw_w,
                    draw_h,
                    is_selected,
                    stroke_color.into(),
                );
            }

            if is_selected
                || is_hovered
                || (show_auto_badges && draw_w >= 26.0 && badges.len() < 120)
            {
                badges.push(BadgeItem {
                    x: box_x,
                    y: box_y,
                    class_id: det.class_id,
                    confidence: det.confidence,
                    is_highlighted: is_selected || is_hovered,
                });
            }
        }
    });

    paint_badges_layer(window, cx, viewport, &badges);
    paint_ruler_layers(window, cx, viewport, orig_w, orig_h, geo_transform, state);
}

fn paint_corner_brackets(
    window: &mut Window,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    is_selected: bool,
    fallback_color: Background,
) {
    let len = (w.min(h) * 0.35).clamp(4.0, 14.0);
    let t = if is_selected { 2.5 } else { 2.0 };
    let color: Background = if is_selected {
        gpui::white().into()
    } else {
        fallback_color
    };

    let rects = [
        (x - 1.0, y - 1.0, len, t),
        (x - 1.0, y - 1.0, t, len),
        (x + w - len + 1.0, y - 1.0, len, t),
        (x + w - t + 1.0, y - 1.0, t, len),
        (x - 1.0, y + h - t + 1.0, len, t),
        (x - 1.0, y + h - len + 1.0, t, len),
        (x + w - len + 1.0, y + h - t + 1.0, len, t),
        (x + w - t + 1.0, y + h - len + 1.0, t, len),
    ];

    for (rx, ry, rw, rh) in rects {
        window.paint_quad(fill(
            Bounds::new(point(px(rx), px(ry)), size(px(rw), px(rh))),
            color,
        ));
    }
}
