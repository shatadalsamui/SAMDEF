use crate::app::SamdefApp;
use crate::model::{class_hsla, CLASS_COUNT};
use crate::ui::theme::*;
use gpui::{
    canvas, div, fill, point, prelude::*, px, rgb, rgba, size, BorderStyle, Bounds, Corners, Edges,
    IntoElement, MouseButton, PaintQuad, Pixels, Point,
};

impl SamdefApp {
    fn pan_from_minimap_pos(&mut self, window_pos: Point<Pixels>, mini_w: f32, mini_h: f32) {
        let mm = self.minimap_bounds.get();
        let mw: f32 = mm.size.width.into();
        let mh: f32 = mm.size.height.into();
        if mw <= 1.0 || mh <= 1.0 {
            return;
        }
        let mx: f32 = mm.origin.x.into();
        let my: f32 = mm.origin.y.into();
        let wx: f32 = window_pos.x.into();
        let wy: f32 = window_pos.y.into();

        let rel_x = ((wx - mx) / mini_w.max(1.0)).clamp(0.0, 1.0);
        let rel_y = ((wy - my) / mini_h.max(1.0)).clamp(0.0, 1.0);

        let (orig_w, orig_h) = self.image_dims();
        let vp = self.vp_bounds();
        let (_, _, img_w, img_h) = self.viewer_state.calc_image_rect(vp, orig_w, orig_h);

        self.viewer_state.offset_x = rel_x * img_w - img_w * 0.5;
        self.viewer_state.offset_y = rel_y * img_h - img_h * 0.5;
        self.viewer_state.clamp_offset(vp, orig_w, orig_h);
    }

    pub fn render_minimap(&self, cx: &gpui::Context<Self>) -> impl IntoElement {
        let (orig_w, orig_h) = self.image_dims();
        let box_outer = 164.0_f32;
        let ratio = (box_outer - 16.0) / orig_w.max(orig_h).max(1.0);
        let mini_w = (orig_w * ratio).round().max(24.0);
        let mini_h = (orig_h * ratio).round().max(24.0);

        let render_image = self.current_image.as_ref().map(|i| i.render_image.clone());
        let vp = self.vp_bounds();
        let (img_x, img_y, img_w, img_h) = self.viewer_state.calc_image_rect(vp, orig_w, orig_h);
        let vx: f32 = vp.origin.x.into();
        let vy: f32 = vp.origin.y.into();
        let vw: f32 = vp.size.width.into();
        let vh: f32 = vp.size.height.into();

        let (frustum_x, frustum_y, frustum_w, frustum_h) = if vw > 1.0 && img_w > 1.0 {
            let x0 = (((vx - img_x) / img_w) * mini_w).clamp(0.0, mini_w);
            let y0 = (((vy - img_y) / img_h) * mini_h).clamp(0.0, mini_h);
            let x1 = (((vx + vw - img_x) / img_w) * mini_w).clamp(0.0, mini_w);
            let y1 = (((vy + vh - img_y) / img_h) * mini_h).clamp(0.0, mini_h);
            (x0, y0, (x1 - x0).max(3.0), (y1 - y0).max(3.0))
        } else {
            (0.0, 0.0, mini_w, mini_h)
        };

        let dets = self.detections.clone();
        let class_vis = self.class_visible;
        let thresh = self.confidence_threshold;
        let mm_bounds_cell = self.minimap_bounds.clone();
        let bottom_offset = if self.show_table_drawer { 252.0 } else { 14.0 };

        div()
            .id("minimap-card")
            .occlude()
            .absolute()
            .bottom(px(bottom_offset))
            .right(px(14.0))
            .w(px(box_outer))
            .h(px(box_outer))
            .rounded_md()
            .bg(rgb(BG_CARD))
            .border_1()
            .border_color(rgb(BORDER_STRONG))
            .shadow_md()
            .p_1p5()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("minimap-canvas-hit")
                    .w(px(mini_w))
                    .h(px(mini_h))
                    .bg(rgb(0x070709))
                    .overflow_hidden()
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                mm_bounds_cell.set(bounds);
                                bounds
                            },
                            move |bounds, _, window, _| {
                                let bx: f32 = bounds.origin.x.into();
                                let by: f32 = bounds.origin.y.into();

                                // Layer 1: Satellite thumbnail
                                window.paint_layer(bounds, |window| {
                                    if let Some(img_data) = render_image {
                                        let _ = window.paint_image(
                                            bounds,
                                            Corners::default(),
                                            img_data,
                                            0,
                                            false,
                                        );
                                    }
                                });

                                // Layer 2: Radar neon detection blips + cyan viewport frustum
                                window.paint_layer(bounds, |window| {
                                    for d in &dets {
                                        if d.class_id < CLASS_COUNT
                                            && class_vis[d.class_id]
                                            && d.confidence >= thresh
                                        {
                                            let (cx_p, cy_p) = d.center();
                                            let dx = bx + (cx_p / orig_w) * mini_w;
                                            let dy = by + (cy_p / orig_h) * mini_h;
                                            window.paint_quad(fill(
                                                Bounds::new(
                                                    point(px(dx), px(dy)),
                                                    size(px(1.6), px(1.6)),
                                                ),
                                                class_hsla(d.class_id, 0.85),
                                            ));
                                        }
                                    }

                                    window.paint_quad(PaintQuad {
                                        bounds: Bounds::new(
                                            point(px(bx + frustum_x), px(by + frustum_y)),
                                            size(px(frustum_w), px(frustum_h)),
                                        ),
                                        corner_radii: (0.).into(),
                                        background: rgba(0x06b6d426).into(),
                                        border_widths: Edges::all(px(1.5)),
                                        border_color: rgb(ACCENT_CYAN).into(),
                                        border_style: BorderStyle::Solid,
                                    });
                                });
                            },
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &gpui::MouseDownEvent, _, cx| {
                            this.pan_from_minimap_pos(ev.position, mini_w, mini_h);
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, ev: &gpui::MouseMoveEvent, _, cx| {
                        if !this.viewer_state.is_dragging
                            && ev.pressed_button == Some(MouseButton::Left)
                        {
                            this.pan_from_minimap_pos(ev.position, mini_w, mini_h);
                            cx.notify();
                        }
                    })),
            )
    }
}
