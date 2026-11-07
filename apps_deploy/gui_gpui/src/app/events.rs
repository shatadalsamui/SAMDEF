use crate::app::{ActiveInput, SamdefApp};
use gpui::{
    Context, KeystrokeEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ScrollDelta, ScrollWheelEvent, Window,
};

impl SamdefApp {
    #[inline]
    fn hit_tolerance_px(&self) -> f32 {
        let (orig_w, orig_h) = self.image_dims();
        let (_, _, img_w, _) = self
            .viewer_state
            .calc_image_rect(self.vp_bounds(), orig_w, orig_h);
        let px_per_src = (img_w / orig_w.max(1.0)).max(0.01);
        3.0 / px_per_src
    }

    pub fn on_viewport_scroll(&mut self, ev: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let factor = match ev.delta {
            ScrollDelta::Lines(p) => {
                if p.y.abs() < 1e-4 {
                    return;
                }
                let step = (p.y.signum() * p.y.abs().clamp(0.25, 3.0)) * 0.10;
                step.exp()
            }
            ScrollDelta::Pixels(p) => {
                let py: f32 = p.y.into();
                if py.abs() < 0.05 {
                    return;
                }
                (py * 0.0045).clamp(-0.35, 0.35).exp()
            }
        };

        let (orig_w, orig_h) = self.image_dims();
        self.viewer_state
            .zoom_at_cursor(factor, ev.position, self.vp_bounds(), orig_w, orig_h);
        self.viewer_state.cursor_img_pixel =
            self.viewer_state
                .screen_to_pixel(ev.position, self.vp_bounds(), orig_w, orig_h);
        cx.notify();
    }

    pub fn on_viewport_mouse_down(
        &mut self,
        button: MouseButton,
        ev: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) {
        self.active_input = None;
        let (orig_w, orig_h) = self.image_dims();
        let vp = self.vp_bounds();

        if button == MouseButton::Right
            && self.viewer_state.is_measure_mode
            && self.viewer_state.measure_start.is_some()
        {
            self.viewer_state.measure_start = None;
            self.viewer_state.measure_current = None;
            self.viewer_state.is_measure_pinned = false;
            cx.notify();
            return;
        }

        if button == MouseButton::Left && self.viewer_state.is_measure_mode {
            let pt = self
                .viewer_state
                .screen_to_pixel_clamped(ev.position, vp, orig_w, orig_h);
            if self.viewer_state.measure_start.is_some() && !self.viewer_state.is_measure_pinned {
                self.viewer_state.measure_current = pt;
                self.viewer_state.is_measure_pinned = true;
            } else {
                self.viewer_state.measure_start = pt;
                self.viewer_state.measure_current = pt;
                self.viewer_state.is_measure_pinned = false;
            }
            cx.notify();
            return;
        }

        self.viewer_state.is_dragging = true;
        self.viewer_state.did_drag = false;
        self.viewer_state.drag_button = Some(button);
        self.viewer_state.drag_start = ev.position;
        self.viewer_state.start_offset_x = self.viewer_state.offset_x;
        self.viewer_state.start_offset_y = self.viewer_state.offset_y;
    }

    pub fn on_viewport_mouse_move(&mut self, ev: &MouseMoveEvent, cx: &mut Context<Self>) {
        let (orig_w, orig_h) = self.image_dims();
        let vp = self.vp_bounds();
        let pixel = self
            .viewer_state
            .screen_to_pixel(ev.position, vp, orig_w, orig_h);
        self.viewer_state.cursor_img_pixel = pixel;

        if self.viewer_state.is_dragging {
            let cur_x: f32 = ev.position.x.into();
            let cur_y: f32 = ev.position.y.into();
            let start_x: f32 = self.viewer_state.drag_start.x.into();
            let start_y: f32 = self.viewer_state.drag_start.y.into();
            let dx = cur_x - start_x;
            let dy = cur_y - start_y;

            if dx.hypot(dy) > 3.0 {
                self.viewer_state.did_drag = true;
            }
            if self.viewer_state.did_drag {
                self.viewer_state.offset_x = self.viewer_state.start_offset_x - dx;
                self.viewer_state.offset_y = self.viewer_state.start_offset_y - dy;
                self.viewer_state.clamp_offset(vp, orig_w, orig_h);
            }
            cx.notify();
            return;
        }

        if self.viewer_state.is_measure_mode
            && self.viewer_state.measure_start.is_some()
            && !self.viewer_state.is_measure_pinned
        {
            self.viewer_state.measure_current =
                self.viewer_state
                    .screen_to_pixel_clamped(ev.position, vp, orig_w, orig_h);
            cx.notify();
            return;
        }

        let tol = self.hit_tolerance_px();
        self.viewer_state.hovered_detection_id = pixel.and_then(|p| {
            self.viewer_state
                .hit_test(
                    p,
                    &self.detections,
                    &self.class_visible,
                    self.confidence_threshold,
                    tol,
                )
                .map(|d| d.id)
        });
        cx.notify();
    }

    pub fn on_viewport_mouse_up(
        &mut self,
        button: MouseButton,
        ev: &MouseUpEvent,
        cx: &mut Context<Self>,
    ) {
        let was_click = self.viewer_state.is_dragging
            && self.viewer_state.drag_button == Some(MouseButton::Left)
            && button == MouseButton::Left
            && !self.viewer_state.did_drag;

        self.viewer_state.is_dragging = false;
        self.viewer_state.drag_button = None;
        self.viewer_state.did_drag = false;

        if was_click {
            let (orig_w, orig_h) = self.image_dims();
            let vp = self.vp_bounds();
            let tol = self.hit_tolerance_px();

            if let Some(pixel) = self
                .viewer_state
                .screen_to_pixel(ev.position, vp, orig_w, orig_h)
                && let Some(&det) = self.viewer_state.hit_test(
                    pixel,
                    &self.detections,
                    &self.class_visible,
                    self.confidence_threshold,
                    tol,
                )
            {
                if ev.click_count >= 2 {
                    self.viewer_state
                        .focus_on_detection(&det, vp, orig_w, orig_h);
                } else {
                    self.viewer_state.selected_detection_id = Some(det.id);
                }
            } else if ev.click_count >= 2 {
                self.viewer_state.reset_view();
            } else {
                self.viewer_state.selected_detection_id = None;
            }
        }
        cx.notify();
    }

    pub fn on_viewport_mouse_up_out(&mut self, cx: &mut Context<Self>) {
        if self.viewer_state.is_dragging {
            self.viewer_state.is_dragging = false;
            self.viewer_state.drag_button = None;
            self.viewer_state.did_drag = false;
            cx.notify();
        }
    }

    pub fn handle_keystroke(
        &mut self,
        ev: &KeystrokeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if ev.keystroke.modifiers.control && ev.keystroke.key == "q" {
            cx.quit();
            return;
        }
        if ev.keystroke.modifiers.control && ev.keystroke.key == "f" {
            self.active_input = Some(ActiveInput::FileSearch);
            self.file_dropdown_open = true;
            cx.notify();
            return;
        }
        if self.handle_input_keystroke(ev, cx) {
            return;
        }
        let (orig_w, orig_h) = self.image_dims();
        let vp = self.vp_bounds();
        match ev.keystroke.key.as_str() {
            "f11" => window.toggle_fullscreen(),
            "/" => {
                self.active_input = Some(ActiveInput::FileSearch);
                self.file_dropdown_open = true;
            }
            "w" | "up" => {
                self.viewer_state.offset_y -= 48.0;
                self.viewer_state.clamp_offset(vp, orig_w, orig_h);
            }
            "s" | "down" => {
                self.viewer_state.offset_y += 48.0;
                self.viewer_state.clamp_offset(vp, orig_w, orig_h);
            }
            "a" | "left" => {
                self.viewer_state.offset_x -= 48.0;
                self.viewer_state.clamp_offset(vp, orig_w, orig_h);
            }
            "d" | "right" => {
                self.viewer_state.offset_x += 48.0;
                self.viewer_state.clamp_offset(vp, orig_w, orig_h);
            }
            "+" | "=" => self.viewer_state.zoom_at_center(1.25, vp, orig_w, orig_h),
            "-" | "_" => self.viewer_state.zoom_at_center(0.80, vp, orig_w, orig_h),
            "0" => self.viewer_state.set_native_1_to_1(vp, orig_w, orig_h),
            "r" | "space" => self.viewer_state.reset_view(),
            "escape" => {
                self.viewer_state.is_measure_mode = false;
                self.viewer_state.measure_start = None;
                self.viewer_state.measure_current = None;
                self.viewer_state.selected_detection_id = None;
                self.show_table_drawer = false;
            }
            "n" | "]" => return self.next_file(cx),
            "p" | "[" => return self.prev_file(cx),
            "l" => self.viewer_state.show_labels = !self.viewer_state.show_labels,
            "m" => {
                self.viewer_state.is_measure_mode = !self.viewer_state.is_measure_mode;
                self.viewer_state.measure_start = None;
                self.viewer_state.measure_current = None;
                self.viewer_state.is_measure_pinned = false;
            }
            "t" => self.show_table_drawer = !self.show_table_drawer,
            "g" => self.viewer_state.is_grayscale = !self.viewer_state.is_grayscale,
            k @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8") => {
                if let Ok(num) = k.parse::<usize>() {
                    self.class_visible[num - 1] = !self.class_visible[num - 1];
                }
            }
            _ => return,
        }
        cx.notify();
    }
}

