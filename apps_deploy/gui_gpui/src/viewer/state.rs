use crate::model::DisplayDetection;
use gpui::{px, Bounds, MouseButton, Pixels, Point};

pub const MIN_ZOOM_SCALE: f32 = 0.10;
pub const MAX_ZOOM_SCALE: f32 = 40.0;

#[derive(Clone, Debug)]
pub struct ViewerState {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,

    pub is_dragging: bool,
    pub did_drag: bool,
    pub drag_button: Option<MouseButton>,
    pub drag_start: Point<Pixels>,
    pub start_offset_x: f32,
    pub start_offset_y: f32,

    pub cursor_img_pixel: Option<(f32, f32)>,
    pub hovered_detection_id: Option<usize>,
    pub selected_detection_id: Option<usize>,

    pub is_measure_mode: bool,
    pub measure_start: Option<(f32, f32)>,
    pub measure_current: Option<(f32, f32)>,
    pub is_measure_pinned: bool,

    pub is_grayscale: bool,
    pub show_labels: bool,
    pub show_minimap: bool,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            is_dragging: false,
            did_drag: false,
            drag_button: None,
            drag_start: Point::new(px(0.), px(0.)),
            start_offset_x: 0.0,
            start_offset_y: 0.0,
            cursor_img_pixel: None,
            hovered_detection_id: None,
            selected_detection_id: None,
            is_measure_mode: false,
            measure_start: None,
            measure_current: None,
            is_measure_pinned: false,
            is_grayscale: false,
            show_labels: true,
            show_minimap: true,
        }
    }
}

impl ViewerState {
    pub fn reset_view(&mut self) {
        self.scale = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.is_dragging = false;
        self.did_drag = false;
        self.drag_button = None;
        self.hovered_detection_id = None;
        self.measure_start = None;
        self.measure_current = None;
        self.is_measure_pinned = false;
    }

    #[inline]
    pub fn base_scale(viewport: Bounds<Pixels>, orig_w: f32, orig_h: f32) -> f32 {
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        if vw <= 1.0 || vh <= 1.0 || orig_w <= 0.0 || orig_h <= 0.0 {
            return 1.0;
        }
        (vw / orig_w).min(vh / orig_h)
    }

    /// Set zoom so 1 screen pixel equals 1 original satellite image pixel, keeping center locked
    pub fn set_native_1_to_1(&mut self, viewport: Bounds<Pixels>, orig_w: f32, orig_h: f32) {
        let base = Self::base_scale(viewport, orig_w, orig_h);
        if base > 0.0 {
            let target_scale = (1.0 / base).clamp(MIN_ZOOM_SCALE, MAX_ZOOM_SCALE);
            let factor = target_scale / self.scale;
            self.zoom_at_center(factor, viewport, orig_w, orig_h);
        }
    }

    /// Calculate displayed image dimensions and top-left screen position
    pub fn calc_image_rect(
        &self,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) -> (f32, f32, f32, f32) {
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        let vx: f32 = viewport.origin.x.into();
        let vy: f32 = viewport.origin.y.into();

        if vw <= 1.0 || vh <= 1.0 || orig_w <= 0.0 || orig_h <= 0.0 {
            return (vx, vy, vw.max(1.0), vh.max(1.0));
        }

        let final_scale = Self::base_scale(viewport, orig_w, orig_h) * self.scale;
        let img_w = orig_w * final_scale;
        let img_h = orig_h * final_scale;
        let img_x = vx + (vw - img_w) * 0.5 - self.offset_x;
        let img_y = vy + (vh - img_h) * 0.5 - self.offset_y;

        (img_x, img_y, img_w, img_h)
    }

    /// Keep image reachable while allowing any edge pixel to be centered in the viewport
    pub fn clamp_offset(&mut self, viewport: Bounds<Pixels>, orig_w: f32, orig_h: f32) {
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        if vw <= 1.0 || vh <= 1.0 || orig_w <= 0.0 || orig_h <= 0.0 {
            return;
        }
        let final_scale = Self::base_scale(viewport, orig_w, orig_h) * self.scale;
        let max_x = (orig_w * final_scale * 0.5).max(32.0);
        let max_y = (orig_h * final_scale * 0.5).max(32.0);
        self.offset_x = self.offset_x.clamp(-max_x, max_x);
        self.offset_y = self.offset_y.clamp(-max_y, max_y);
    }

    /// Convert screen cursor position to image pixel coordinates (`clamped = false` returns None outside image)
    fn map_screen_to_pixel(
        &self,
        cursor: Point<Pixels>,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
        clamped: bool,
    ) -> Option<(f32, f32)> {
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        if vw <= 1.0 || vh <= 1.0 || orig_w <= 0.0 || orig_h <= 0.0 {
            return None;
        }
        let (cx, cy): (f32, f32) = (cursor.x.into(), cursor.y.into());
        let (img_x, img_y, img_w, img_h) = self.calc_image_rect(viewport, orig_w, orig_h);
        let (rel_x, rel_y) = (cx - img_x, cy - img_y);
        if !clamped && (rel_x < 0.0 || rel_x > img_w || rel_y < 0.0 || rel_y > img_h) {
            return None;
        }
        Some((
            ((rel_x / img_w) * orig_w).clamp(0.0, orig_w),
            ((rel_y / img_h) * orig_h).clamp(0.0, orig_h),
        ))
    }

    pub fn screen_to_pixel(
        &self,
        cursor: Point<Pixels>,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) -> Option<(f32, f32)> {
        self.map_screen_to_pixel(cursor, viewport, orig_w, orig_h, false)
    }

    pub fn screen_to_pixel_clamped(
        &self,
        cursor: Point<Pixels>,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) -> Option<(f32, f32)> {
        self.map_screen_to_pixel(cursor, viewport, orig_w, orig_h, true)
    }

    /// Convert image pixel coordinates to screen coordinates
    pub fn pixel_to_screen(
        &self,
        px_x: f32,
        px_y: f32,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) -> (f32, f32) {
        let (img_x, img_y, img_w, img_h) = self.calc_image_rect(viewport, orig_w, orig_h);
        (img_x + (px_x / orig_w) * img_w, img_y + (px_y / orig_h) * img_h)
    }

    /// Sub-pixel invariant cursor zoom: keeps the exact image pixel under `cursor` stationary
    pub fn zoom_at_cursor(
        &mut self,
        factor: f32,
        cursor: Point<Pixels>,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) {
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        let vx: f32 = viewport.origin.x.into();
        let vy: f32 = viewport.origin.y.into();

        let old_scale = self.scale;
        let new_scale = (old_scale * factor).clamp(MIN_ZOOM_SCALE, MAX_ZOOM_SCALE);
        let r = new_scale / old_scale;

        if vw > 1.0 && vh > 1.0 {
            let cx: f32 = cursor.x.into();
            let cy: f32 = cursor.y.into();
            self.offset_x = r * self.offset_x + (r - 1.0) * (cx - (vx + vw * 0.5));
            self.offset_y = r * self.offset_y + (r - 1.0) * (cy - (vy + vh * 0.5));
        }

        self.scale = new_scale;
        self.clamp_offset(viewport, orig_w, orig_h);
    }

    /// Zoom anchored at the center of the current viewport
    pub fn zoom_at_center(&mut self, factor: f32, viewport: Bounds<Pixels>, orig_w: f32, orig_h: f32) {
        let vx: f32 = viewport.origin.x.into();
        let vy: f32 = viewport.origin.y.into();
        let vw: f32 = viewport.size.width.into();
        let vh: f32 = viewport.size.height.into();
        self.zoom_at_cursor(
            factor,
            Point::new(px(vx + vw * 0.5), px(vy + vh * 0.5)),
            viewport,
            orig_w,
            orig_h,
        );
    }

    /// Center and adaptively magnify the viewport directly onto a detection bounding box
    pub fn focus_on_detection(
        &mut self,
        det: &DisplayDetection,
        viewport: Bounds<Pixels>,
        orig_w: f32,
        orig_h: f32,
    ) {
        let base = Self::base_scale(viewport, orig_w, orig_h);
        let det_span = det.width().max(det.height()).max(12.0);
        self.scale = ((140.0 / det_span) / base).clamp(3.0, 16.0);

        let final_scale = base * self.scale;
        let img_w = orig_w * final_scale;
        let img_h = orig_h * final_scale;
        let (det_cx, det_cy) = det.center();

        self.offset_x = (det_cx / orig_w) * img_w - img_w * 0.5;
        self.offset_y = (det_cy / orig_h) * img_h - img_h * 0.5;
        self.clamp_offset(viewport, orig_w, orig_h);
        self.selected_detection_id = Some(det.id);
    }
}
