use crate::model::{class_color, class_name, DisplayDetection};
use crate::viewer::state::calc_image_size;
use crate::viewer::ViewerState;
use iced::advanced::image;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text;
use iced::alignment;
use iced::{Color, Rectangle};

/// Render the layers:
/// Layer 1: The real satellite TIFF image
/// Layer 2: Dynamic vector bounding boxes directly in the viewer layer (zero texture allocation)
/// Layer 3: Optional confidence badges when zoomed in
#[allow(clippy::too_many_arguments)]
pub fn render_viewer<Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    handle: &image::Handle,
    state: &ViewerState,
    detections: &[DisplayDetection],
    class_visible: &[bool; 8],
    confidence_threshold: f32,
    show_labels: bool,
) where
    Renderer: image::Renderer<Handle = image::Handle>
        + text::Renderer<Font = iced::Font>
        + renderer::Renderer,
{
    let image_size = calc_image_size(renderer, handle, state, bounds.size());

    // Exact image top-left position in screen coordinates
    let img_x = bounds.x + (bounds.width - image_size.width) / 2.0 - state.current_offset.x;
    let img_y = bounds.y + (bounds.height - image_size.height) / 2.0 - state.current_offset.y;

    let iced::Size {
        width: orig_w,
        height: orig_h,
    } = renderer.dimensions(handle);
    let scale_x = image_size.width / (orig_w as f32);
    let scale_y = image_size.height / (orig_h as f32);

    let img_rect = Rectangle {
        x: img_x,
        y: img_y,
        width: image_size.width,
        height: image_size.height,
    };

    // LAYER 1: The real satellite TIFF image (drawn in base viewport layer)
    renderer.with_layer(bounds, |renderer| {
        image::Renderer::draw(renderer, handle.clone(), img_rect);
    });

    // LAYER 2: Dynamic transparent annotation layer on top (only the boxes)
    let any_class_visible = class_visible.iter().any(|&v| v);
    if any_class_visible {
        let show_badge = show_labels && (state.scale >= 1.5 || scale_x >= 0.35);

        // Adaptive line thickness: thin (1.0px) when zoomed out, bolder (1.5-2.0px) when zoomed in
        let border_thickness = if state.scale < 1.2 { 1.0 } else if state.scale < 3.0 { 1.5 } else { 2.0 };

        renderer.with_layer(bounds, |renderer| {
            for det in detections {
                if det.class_id >= 8
                    || !class_visible[det.class_id]
                    || det.confidence < confidence_threshold
                {
                    continue;
                }

                let box_x = img_x + (det.x_min * scale_x);
                let box_y = img_y + (det.y_min * scale_y);
                let box_w = (det.x_max - det.x_min) * scale_x;
                let box_h = (det.y_max - det.y_min) * scale_y;

                // Frustum cull boxes outside current view
                if box_x + box_w < bounds.x
                    || box_x > bounds.x + bounds.width
                    || box_y + box_h < bounds.y
                    || box_y > bounds.y + bounds.height
                {
                    continue;
                }

                let color = class_color(det.class_id);

                // Draw bounding box: hollow with 4 solid edges, center 100% transparent
                draw_hollow_box(renderer, box_x, box_y, box_w, box_h, border_thickness, color);

                // Optional label badge when zoomed in
                if show_badge && box_w >= 24.0 {
                    let label = format!(
                        "{} {:.0}%",
                        class_name(det.class_id),
                        det.confidence * 100.0
                    );
                    let tag_w = (label.len() as f32) * 6.5 + 8.0;
                    let tag_h = 14.0;
                    let tag_y = (box_y - tag_h).max(img_y);

                    draw_solid_rect(
                        renderer,
                        Rectangle {
                            x: box_x,
                            y: tag_y,
                            width: tag_w,
                            height: tag_h,
                        },
                        Color { a: 0.90, ..color },
                    );

                    renderer.fill_text(text::Text {
                        content: &label,
                        bounds: Rectangle {
                            x: box_x + 4.0,
                            y: tag_y + 1.5,
                            width: tag_w,
                            height: tag_h,
                        },
                        size: 9.5,
                        line_height: text::LineHeight::Relative(1.0),
                        color: Color::BLACK,
                        font: renderer.default_font(),
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                        shaping: text::Shaping::Basic,
                    });
                }
            }
        });
    }
}

#[inline]
fn draw_hollow_box<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    thickness: f32,
    color: Color,
) {
    if w < 3.0 || h < 3.0 {
        draw_solid_rect(
            renderer,
            Rectangle {
                x,
                y,
                width: w.max(2.0),
                height: h.max(2.0),
            },
            color,
        );
        return;
    }
    let t = thickness.min(w / 2.0).min(h / 2.0);
    // Top, bottom, left, right edges
    draw_solid_rect(renderer, Rectangle { x, y, width: w, height: t }, color);
    draw_solid_rect(renderer, Rectangle { x, y: y + h - t, width: w, height: t }, color);
    draw_solid_rect(
        renderer,
        Rectangle {
            x,
            y: y + t,
            width: t,
            height: (h - 2.0 * t).max(0.0),
        },
        color,
    );
    draw_solid_rect(
        renderer,
        Rectangle {
            x: x + w - t,
            y: y + t,
            width: t,
            height: (h - 2.0 * t).max(0.0),
        },
        color,
    );
}

#[inline]
fn draw_solid_rect<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    color: Color,
) {
    renderer.fill_quad(
        Quad {
            bounds,
            border_radius: [0.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
        color,
    );
}
