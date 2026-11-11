use crate::modules::io::virtual_tiler::TILE_SIZE;
use crate::modules::processing::geometry::{
    iou_1d, BoundingBox, Detection, RawDetection, TRUNC_BOTTOM, TRUNC_LEFT, TRUNC_RIGHT, TRUNC_TOP,
};
use std::cmp::Ordering;

const SEAM_MERGE_GAP: f32 = 16.0;
const ORTHO_IOU_MIN: f32 = 0.60;
const MIN_MERGE_CONF: f32 = 0.10;
const MERGE_CONF_RATIO: f32 = 0.30;

#[derive(Copy, Clone)]
enum SeamAxis {
    Horizontal,
    Vertical,
}

impl SeamAxis {
    #[inline]
    fn flags(self) -> (u8, u8, u8, u8, u8) {
        match self {
            SeamAxis::Horizontal => (
                TRUNC_RIGHT,
                TRUNC_LEFT,
                TRUNC_LEFT,
                TRUNC_RIGHT,
                TRUNC_TOP | TRUNC_BOTTOM,
            ),
            SeamAxis::Vertical => (
                TRUNC_BOTTOM,
                TRUNC_TOP,
                TRUNC_TOP,
                TRUNC_BOTTOM,
                TRUNC_LEFT | TRUNC_RIGHT,
            ),
        }
    }

    #[inline]
    fn origin(self, tile_origin: (i32, i32)) -> i32 {
        match self {
            SeamAxis::Horizontal => tile_origin.0,
            SeamAxis::Vertical => tile_origin.1,
        }
    }

    #[inline]
    fn spans(self, b: &BoundingBox) -> (f32, f32, f32, f32) {
        match self {
            SeamAxis::Horizontal => (b.x_min, b.x_max, b.y_min, b.y_max),
            SeamAxis::Vertical => (b.y_min, b.y_max, b.x_min, b.x_max),
        }
    }
}

fn try_merge_along_axis(
    left: &RawDetection,
    right: &RawDetection,
    axis: SeamAxis,
) -> Option<(f32, RawDetection)> {
    let (cut_neg, cut_pos, keep_outer_neg, keep_outer_pos, keep_perp) = axis.flags();
    if (left.trunc_mask & cut_neg) == 0 || (right.trunc_mask & cut_pos) == 0 {
        return None;
    }

    let left_orig = axis.origin(left.tile_origin);
    let right_orig = axis.origin(right.tile_origin);
    if right_orig <= left_orig || right_orig > left_orig + TILE_SIZE as i32 {
        return None;
    }

    let c_left = left.detection.confidence;
    let c_right = right.detection.confidence;
    let min_c = c_left.min(c_right);
    let max_c = c_left.max(c_right);
    if min_c < MIN_MERGE_CONF || min_c < max_c * MERGE_CONF_RATIO {
        return None;
    }

    let (l_min, l_max, l_omin, l_omax) = axis.spans(&left.detection.bbox);
    let (r_min, r_max, r_omin, r_omax) = axis.spans(&right.detection.bbox);

    // Merged span must extend outward across both tiles and share the overlap boundary.
    if l_min >= r_min || r_max <= l_max || r_min > l_max + SEAM_MERGE_GAP {
        return None;
    }
    if l_max < (right_orig as f32) - SEAM_MERGE_GAP
        || r_min > ((left_orig + TILE_SIZE as i32) as f32) + SEAM_MERGE_GAP
    {
        return None;
    }

    let ortho_iou = iou_1d(l_omin, l_omax, r_omin, r_omax);
    if ortho_iou < ORTHO_IOU_MIN {
        return None;
    }

    let wa = c_left.max(1e-4);
    let wb = c_right.max(1e-4);
    let wsum = wa + wb;
    let m_min = l_min.min(r_min);
    let m_max = l_max.max(r_max);
    let m_omin = (l_omin * wa + r_omin * wb) / wsum;
    let m_omax = (l_omax * wa + r_omax * wb) / wsum;

    let (bbox, tile_origin) = match axis {
        SeamAxis::Horizontal => (
            BoundingBox {
                x_min: m_min,
                y_min: m_omin,
                x_max: m_max,
                y_max: m_omax,
            },
            (
                right.tile_origin.0,
                if wa >= wb {
                    left.tile_origin.1
                } else {
                    right.tile_origin.1
                },
            ),
        ),
        SeamAxis::Vertical => (
            BoundingBox {
                x_min: m_omin,
                y_min: m_min,
                x_max: m_omax,
                y_max: m_max,
            },
            (
                if wa >= wb {
                    left.tile_origin.0
                } else {
                    right.tile_origin.0
                },
                right.tile_origin.1,
            ),
        ),
    };

    let trunc_mask = (left.trunc_mask & keep_outer_neg)
        | (right.trunc_mask & keep_outer_pos)
        | ((left.trunc_mask | right.trunc_mask) & keep_perp);

    Some((
        ortho_iou + 0.1 * min_c,
        RawDetection {
            detection: Detection {
                bbox,
                class_id: left.detection.class_id,
                confidence: max_c,
            },
            tile_origin,
            trunc_mask,
            was_merged: true,
        },
    ))
}

fn merge_axis_pass(mut items: Vec<RawDetection>, axis: SeamAxis) -> (Vec<RawDetection>, bool) {
    items.sort_unstable_by(|a, b| {
        axis.origin(a.tile_origin)
            .cmp(&axis.origin(b.tile_origin))
            .then_with(|| {
                axis.spans(&a.detection.bbox)
                    .0
                    .partial_cmp(&axis.spans(&b.detection.bbox).0)
                    .unwrap_or(Ordering::Equal)
            })
    });

    let (cut_neg, cut_pos, _, _, _) = axis.flags();
    let n = items.len();
    let right_candidates: Vec<usize> = (0..n)
        .filter(|&idx| {
            (items[idx].trunc_mask & cut_pos) != 0
                && items[idx].detection.confidence >= MIN_MERGE_CONF
        })
        .collect();

    if right_candidates.is_empty() {
        return (items, false);
    }

    let mut merged_away = vec![false; n];
    let mut absorbed_right = vec![false; n];
    let mut any_merged = false;

    for i in 0..n {
        if merged_away[i]
            || (items[i].trunc_mask & cut_neg) == 0
            || items[i].detection.confidence < MIN_MERGE_CONF
        {
            continue;
        }

        let mut best: Option<(usize, f32, RawDetection)> = None;
        for &j in &right_candidates {
            if j == i || merged_away[j] || absorbed_right[j] {
                continue;
            }
            if let Some((score, merged)) = try_merge_along_axis(&items[i], &items[j], axis) {
                if best.as_ref().map_or(true, |(_, bs, _)| score > *bs) {
                    best = Some((j, score, merged));
                }
            }
        }

        if let Some((best_j, _, merged)) = best {
            items[best_j] = merged;
            absorbed_right[best_j] = true;
            merged_away[i] = true;
            any_merged = true;
        }
    }

    if !any_merged {
        return (items, false);
    }

    let kept = items
        .into_iter()
        .enumerate()
        .filter_map(|(i, d)| (!merged_away[i]).then_some(d))
        .collect();
    (kept, true)
}

/// Pass 1: Iteratively fuses objects cut across overlapping horizontal and vertical seams.
pub fn merge_seam_truncated_boxes(mut items: Vec<RawDetection>) -> Vec<RawDetection> {
    if items.len() < 2 {
        return items;
    }
    for _ in 0..4 {
        let (next, changed_h) = merge_axis_pass(items, SeamAxis::Horizontal);
        let (next, changed_v) = merge_axis_pass(next, SeamAxis::Vertical);
        items = next;
        if !changed_h && !changed_v {
            break;
        }
    }
    items
}

