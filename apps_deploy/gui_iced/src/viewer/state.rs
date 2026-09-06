use iced::advanced::image;
use iced::{Point, Size, Vector};

pub type StatusCallback<'a, Message> = Box<dyn Fn(f32, Option<(f32, f32)>) -> Message + 'a>;

pub struct ViewerState {
    pub scale: f32,
    pub starting_offset: Vector,
    pub current_offset: Vector,
    pub cursor_grabbed_at: Option<Point>,
    pub last_reset: usize,
    pub last_reported_pixel: Option<(f32, f32)>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            starting_offset: Vector::default(),
            current_offset: Vector::default(),
            cursor_grabbed_at: None,
            last_reset: 0,
            last_reported_pixel: None,
        }
    }
}

/// Calculate displayed image size respecting aspect ratio and zoom scale
pub fn calc_image_size(
    renderer: &impl image::Renderer<Handle = image::Handle>,
    handle: &image::Handle,
    state: &ViewerState,
    bounds: Size,
) -> Size {
    let Size { width, height } = renderer.dimensions(handle);
    let dimensions = (width as f32, height as f32);

    let width_ratio = bounds.width / dimensions.0;
    let height_ratio = bounds.height / dimensions.1;
    let ratio = width_ratio.min(height_ratio);
    let base_scale = if ratio < 1.0 { ratio } else { 1.0 };
    let final_scale = base_scale * state.scale;

    Size::new(dimensions.0 * final_scale, dimensions.1 * final_scale)
}

/// Map cursor position in viewport to original image pixel coordinates
pub fn calc_pixel_coord(
    cursor_pos: Point,
    viewport_size: Size,
    image_size: Size,
    offset: Vector,
    orig_w: f32,
    orig_h: f32,
) -> Option<(f32, f32)> {
    let top_left_x = (viewport_size.width / 2.0) - (image_size.width / 2.0) - offset.x;
    let top_left_y = (viewport_size.height / 2.0) - (image_size.height / 2.0) - offset.y;

    let rel_x = cursor_pos.x - top_left_x;
    let rel_y = cursor_pos.y - top_left_y;

    if rel_x >= 0.0 && rel_x < image_size.width && rel_y >= 0.0 && rel_y < image_size.height {
        let px = (rel_x / image_size.width) * orig_w;
        let py = (rel_y / image_size.height) * orig_h;
        Some((px, py))
    } else {
        None
    }
}
