use crate::modules::io::virtual_tiler::TILE_SIZE;
use serde::Serialize;
use std::collections::HashMap;

pub const TRUNC_LEFT: u8 = 0b0001;
pub const TRUNC_RIGHT: u8 = 0b0010;
pub const TRUNC_TOP: u8 = 0b0100;
pub const TRUNC_BOTTOM: u8 = 0b1000;

pub const EDGE_EPS: f32 = 4.0;
const GRID_SIZE: f32 = 1024.0;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl BoundingBox {
    #[inline]
    pub fn area(&self) -> f32 {
        (self.x_max - self.x_min).max(0.0) * (self.y_max - self.y_min).max(0.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Detection {
    pub bbox: BoundingBox,
    pub class_id: usize,
    pub confidence: f32,
}

/// Internal per-tile detection carrying origin and internal seam truncation metadata.
#[derive(Debug, Clone)]
pub struct RawDetection {
    pub detection: Detection,
    pub tile_origin: (i32, i32),
    pub trunc_mask: u8,
    pub was_merged: bool,
}

impl RawDetection {
    pub fn from_detection(detection: Detection, tile_origin: (i32, i32), trunc_mask: u8) -> Self {
        Self {
            detection,
            tile_origin,
            trunc_mask,
            was_merged: false,
        }
    }
}

pub struct OverlapMetrics {
    pub iou: f32,
    pub ios: f32,
    pub ioa_second: f32,
}

/// Computes local tile edge truncation bitmask before global offset translation.
pub fn compute_local_trunc_mask(local_bbox: &BoundingBox, tile_origin: (i32, i32)) -> u8 {
    let max_coord = (TILE_SIZE as f32) - 1.0 - EDGE_EPS;
    let mut mask = 0u8;
    if tile_origin.0 > 0 && local_bbox.x_min <= EDGE_EPS {
        mask |= TRUNC_LEFT;
    }
    if local_bbox.x_max >= max_coord {
        mask |= TRUNC_RIGHT;
    }
    if tile_origin.1 > 0 && local_bbox.y_min <= EDGE_EPS {
        mask |= TRUNC_TOP;
    }
    if local_bbox.y_max >= max_coord {
        mask |= TRUNC_BOTTOM;
    }
    mask
}

/// Clears right/bottom truncation bits when a detection touches the outer GeoTIFF border
/// rather than an internal tile overlap seam.
pub fn finalize_trunc_mask_for_image(
    raw_dets: &mut [RawDetection],
    img_width: usize,
    img_height: usize,
) {
    let outer_x = (img_width as f32) - 1.0 - EDGE_EPS;
    let outer_y = (img_height as f32) - 1.0 - EDGE_EPS;
    for raw in raw_dets {
        if raw.detection.bbox.x_max >= outer_x {
            raw.trunc_mask &= !TRUNC_RIGHT;
        }
        if raw.detection.bbox.y_max >= outer_y {
            raw.trunc_mask &= !TRUNC_BOTTOM;
        }
    }
}

/// Calculates IoU, IoS (inter / min area), and overlap fraction relative to `box_b`.
pub fn calculate_overlap_metrics(box_a: &BoundingBox, box_b: &BoundingBox) -> OverlapMetrics {
    let x_a = box_a.x_min.max(box_b.x_min);
    let y_a = box_a.y_min.max(box_b.y_min);
    let x_b = box_a.x_max.min(box_b.x_max);
    let y_b = box_a.y_max.min(box_b.y_max);

    let inter_area = (x_b - x_a).max(0.0) * (y_b - y_a).max(0.0);
    if inter_area <= 0.0 {
        return OverlapMetrics {
            iou: 0.0,
            ios: 0.0,
            ioa_second: 0.0,
        };
    }

    let area_a = box_a.area();
    let area_b = box_b.area();
    let union_area = area_a + area_b - inter_area;
    let min_area = area_a.min(area_b);

    OverlapMetrics {
        iou: if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        },
        ios: if min_area > 0.0 {
            inter_area / min_area
        } else {
            0.0
        },
        ioa_second: if area_b > 0.0 {
            inter_area / area_b
        } else {
            0.0
        },
    }
}

#[inline]
pub fn iou_1d(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    let inter = (a_max.min(b_max) - a_min.max(b_min)).max(0.0);
    let union = a_max.max(b_max) - a_min.min(b_min);
    if union > 0.0 {
        inter / union
    } else {
        0.0
    }
}

#[inline]
pub fn cell_bounds(bbox: &BoundingBox) -> (i32, i32, i32, i32) {
    if !(bbox.x_min.is_finite()
        && bbox.y_min.is_finite()
        && bbox.x_max.is_finite()
        && bbox.y_max.is_finite())
    {
        return (0, 0, 0, 0);
    }
    let gx0 = (bbox.x_min / GRID_SIZE).floor() as i32;
    let gx1 = ((bbox.x_max / GRID_SIZE).floor() as i32).max(gx0);
    let gy0 = (bbox.y_min / GRID_SIZE).floor() as i32;
    let gy1 = ((bbox.y_max / GRID_SIZE).floor() as i32).max(gy0);
    // Bound span so corrupted outlier coordinates cannot allocate excessive cells.
    (gx0, gx1.min(gx0 + 64), gy0, gy1.min(gy0 + 64))
}

pub fn build_spatial_grid<'a, I>(bboxes: I) -> HashMap<(i32, i32), Vec<usize>>
where
    I: Iterator<Item = (usize, &'a BoundingBox)>,
{
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (idx, bbox) in bboxes {
        let (gx0, gx1, gy0, gy1) = cell_bounds(bbox);
        for gx in gx0..=gx1 {
            for gy in gy0..=gy1 {
                grid.entry((gx, gy)).or_default().push(idx);
            }
        }
    }
    grid
}

