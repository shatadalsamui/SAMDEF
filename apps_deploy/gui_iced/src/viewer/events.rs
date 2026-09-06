use crate::viewer::state::{calc_image_size, calc_pixel_coord, StatusCallback};
use crate::viewer::ViewerState;
use iced::advanced::image;
use iced::advanced::mouse;
use iced::advanced::Shell;
use iced::event::{self, Event};
use iced::{Point, Rectangle, Vector};

/// Handles mouse pan and zoom events for the viewer
#[allow(clippy::too_many_arguments)]
pub fn handle_viewer_event<'a, Message>(
    state: &mut ViewerState,
    event: Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    renderer: &impl image::Renderer<Handle = image::Handle>,
    handle: &image::Handle,
    min_scale: f32,
    max_scale: f32,
    scale_step: f32,
    on_status: &Option<StatusCallback<'a, Message>>,
    shell: &mut Shell<'_, Message>,
) -> event::Status {
    let iced::Size { width: orig_w, height: orig_h } = renderer.dimensions(handle);
    let image_size = calc_image_size(renderer, handle, state, bounds.size());

    match event {
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            let Some(cursor_pos) = cursor.position_in(bounds) else {
                return event::Status::Ignored;
            };
            let global_cursor = cursor.position().unwrap_or(Point::ORIGIN);

            match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    let previous_scale = state.scale;
                    if (y < 0.0 && previous_scale > min_scale) || (y > 0.0 && previous_scale < max_scale) {
                        state.scale = (if y > 0.0 {
                            state.scale * (1.0 + scale_step)
                        } else {
                            state.scale / (1.0 + scale_step)
                        })
                        .clamp(min_scale, max_scale);

                        let new_image_size = calc_image_size(renderer, handle, state, bounds.size());
                        let factor = state.scale / previous_scale - 1.0;
                        let cursor_to_center = global_cursor - bounds.center();
                        let adjustment = cursor_to_center * factor + state.current_offset * factor;

                        state.current_offset = Vector::new(
                            if new_image_size.width > bounds.width {
                                state.current_offset.x + adjustment.x
                            } else {
                                0.0
                            },
                            if new_image_size.height > bounds.height {
                                state.current_offset.y + adjustment.y
                            } else {
                                0.0
                            },
                        );

                        if let Some(f) = on_status {
                            let pixel = calc_pixel_coord(
                                cursor_pos,
                                bounds.size(),
                                new_image_size,
                                state.current_offset,
                                orig_w as f32,
                                orig_h as f32,
                            );
                            shell.publish(f(state.scale, pixel));
                        }
                    }
                }
            }
            event::Status::Captured
        }

        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            let Some(cursor_pos) = cursor.position_in(bounds) else {
                return event::Status::Ignored;
            };
            state.cursor_grabbed_at = Some(cursor_pos);
            state.starting_offset = state.current_offset;
            event::Status::Captured
        }

        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            if state.cursor_grabbed_at.is_some() {
                state.cursor_grabbed_at = None;
                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }

        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            if let Some(origin) = state.cursor_grabbed_at {
                let rel_pos = cursor.position_in(bounds).unwrap_or(position);
                let delta = rel_pos - origin;

                let hidden_width = (image_size.width - bounds.width / 2.0).max(0.0).round();
                let hidden_height = (image_size.height - bounds.height / 2.0).max(0.0).round();

                let x = if bounds.width < image_size.width {
                    (state.starting_offset.x - delta.x).clamp(-hidden_width, hidden_width)
                } else {
                    0.0
                };
                let y = if bounds.height < image_size.height {
                    (state.starting_offset.y - delta.y).clamp(-hidden_height, hidden_height)
                } else {
                    0.0
                };
                state.current_offset = Vector::new(x, y);

                let is_dragging = true;
                notify_status(
                    on_status,
                    cursor.position_in(bounds),
                    bounds,
                    image_size,
                    state,
                    orig_w as f32,
                    orig_h as f32,
                    is_dragging,
                    shell,
                );
                event::Status::Captured
            } else {
                let is_dragging = false;
                notify_status(
                    on_status,
                    cursor.position_in(bounds),
                    bounds,
                    image_size,
                    state,
                    orig_w as f32,
                    orig_h as f32,
                    is_dragging,
                    shell,
                );
                event::Status::Ignored
            }
        }

        _ => event::Status::Ignored,
    }
}

#[allow(clippy::too_many_arguments)]
fn notify_status<'a, Message>(
    on_status: &Option<StatusCallback<'a, Message>>,
    cursor_pos: Option<Point>,
    bounds: Rectangle,
    image_size: iced::Size,
    state: &mut ViewerState,
    orig_w: f32,
    orig_h: f32,
    is_dragging: bool,
    shell: &mut Shell<'_, Message>,
) {
    if let (Some(f), Some(pos)) = (on_status, cursor_pos) {
        let pixel = calc_pixel_coord(
            pos,
            bounds.size(),
            image_size,
            state.current_offset,
            orig_w,
            orig_h,
        );
        if is_dragging || pixel != state.last_reported_pixel {
            state.last_reported_pixel = pixel;
            shell.publish(f(state.scale, pixel));
        }
    }
}
