use crate::model::DisplayDetection;
use crate::viewer::state::ViewerState;

impl ViewerState {
    /// Perform spatial hit test to find which detection was clicked or hovered.
    /// Prioritizes the smallest bounding box when targets are nested (e.g. car next to building).
    pub fn hit_test<'a>(
        &self,
        pixel_coord: (f32, f32),
        detections: &'a [DisplayDetection],
        class_visible: &[bool; 8],
        threshold: f32,
        pixel_tolerance: f32,
    ) -> Option<&'a DisplayDetection> {
        let (px, py) = pixel_coord;
        let pad = pixel_tolerance.max(0.0);
        let mut best: Option<&'a DisplayDetection> = None;
        let mut min_area = f32::MAX;

        for d in detections {
            if d.class_id >= 8 || !class_visible[d.class_id] || d.confidence < threshold {
                continue;
            }
            if px >= d.x_min - pad
                && px <= d.x_max + pad
                && py >= d.y_min - pad
                && py <= d.y_max + pad
            {
                let area = d.area();
                if area < min_area {
                    min_area = area;
                    best = Some(d);
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use crate::model::DisplayDetection;
    use crate::viewer::state::{ViewerState, MAX_ZOOM_SCALE, MIN_ZOOM_SCALE};
    use gpui::{px, Bounds, Point};

    #[test]
    fn test_viewer_zoom_and_clamp() {
        let mut state = ViewerState::default();
        let viewport = Bounds {
            origin: Point::new(px(0.0), px(0.0)),
            size: gpui::size(px(1000.0), px(1000.0)),
        };
        let cursor = Point::new(px(500.0), px(500.0));
        assert_eq!(state.scale, 1.0);

        state.zoom_at_cursor(1.2, cursor, viewport, 1000.0, 1000.0);
        assert!(state.scale > 1.0);

        state.scale = 39.0;
        state.zoom_at_cursor(2.0, cursor, viewport, 1000.0, 1000.0);
        assert_eq!(state.scale, MAX_ZOOM_SCALE);

        state.scale = 0.12;
        state.zoom_at_cursor(0.1, cursor, viewport, 1000.0, 1000.0);
        assert_eq!(state.scale, MIN_ZOOM_SCALE);

        state.reset_view();
        assert_eq!(state.scale, 1.0);
    }

    #[test]
    fn test_zoom_at_cursor_preserves_pixel_under_cursor() {
        let mut state = ViewerState {
            scale: 1.4,
            offset_x: 75.0,
            offset_y: -42.0,
            ..Default::default()
        };
        let viewport = Bounds {
            origin: Point::new(px(350.0), px(40.0)),
            size: gpui::size(px(1080.0), px(860.0)),
        };
        let cursor = Point::new(px(712.0), px(319.0));
        let (orig_w, orig_h) = (3195.0, 3215.0);

        let before = state
            .screen_to_pixel(cursor, viewport, orig_w, orig_h)
            .expect("Cursor starts inside image");
        state.zoom_at_cursor(2.35, cursor, viewport, orig_w, orig_h);
        let after = state
            .screen_to_pixel(cursor, viewport, orig_w, orig_h)
            .expect("Cursor remains inside image");

        assert!((before.0 - after.0).abs() < 1e-2);
        assert!((before.1 - after.1).abs() < 1e-2);
    }

    #[test]
    fn test_screen_pixel_projection() {
        let state = ViewerState {
            scale: 2.0,
            offset_x: 50.0,
            offset_y: 100.0,
            ..Default::default()
        };
        let viewport = Bounds {
            origin: Point::new(px(200.0), px(100.0)),
            size: gpui::size(px(800.0), px(600.0)),
        };

        let (sx, sy) = state.pixel_to_screen(50.0, 50.0, viewport, 1000.0, 1000.0);
        let (back_x, back_y) = state
            .screen_to_pixel(Point::new(px(sx), px(sy)), viewport, 1000.0, 1000.0)
            .expect("Inside bounds");

        assert!((back_x - 50.0).abs() < 1e-2);
        assert!((back_y - 50.0).abs() < 1e-2);
    }

    #[test]
    fn test_hit_testing_priority() {
        let state = ViewerState::default();
        let dets = vec![
            DisplayDetection {
                id: 1,
                x_min: 0.0,
                y_min: 0.0,
                x_max: 200.0,
                y_max: 200.0,
                class_id: 3,
                confidence: 0.9,
            },
            DisplayDetection {
                id: 2,
                x_min: 50.0,
                y_min: 50.0,
                x_max: 80.0,
                y_max: 80.0,
                class_id: 2,
                confidence: 0.95,
            },
        ];
        let class_visible = [true; 8];

        let hit = state.hit_test((60.0, 60.0), &dets, &class_visible, 0.5, 0.0);
        assert_eq!(hit.map(|d| d.id), Some(2));

        let hit_high = state.hit_test((60.0, 60.0), &dets, &class_visible, 0.96, 0.0);
        assert!(hit_high.is_none());

        let mut vis_filtered = [true; 8];
        vis_filtered[2] = false;
        let hit_fallback = state.hit_test((60.0, 60.0), &dets, &vis_filtered, 0.5, 0.0);
        assert_eq!(hit_fallback.map(|d| d.id), Some(1));
    }
}

