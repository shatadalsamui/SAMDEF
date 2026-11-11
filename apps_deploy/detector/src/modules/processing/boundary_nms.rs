use crate::modules::processing::geometry::{
    build_spatial_grid, calculate_overlap_metrics, cell_bounds, Detection, RawDetection,
};
use std::cmp::Ordering;

const ABSORB_IOS_MIN: f32 = 0.60;
const ABSORB_AREA_RATIO: f32 = 1.10;
const ABSORB_CONF_FLOOR: f32 = 0.15;
const ABSORB_CONF_RATIO: f32 = 0.45;
const TRUNC_SCORE_FACTOR: f32 = 0.70;
const IOS_TRUNC_SUPPRESS: f32 = 0.55;
const IOS_CROSS_TILE_SUPPRESS: f32 = 0.75;

/// Pass 2: Drops truncated cut slivers absorbed inside complete non-truncated detections,
/// preventing priority inversion without letting low-confidence noise suppress real hits.
pub fn absorb_truncated_fragments(items: Vec<RawDetection>) -> Vec<RawDetection> {
    if items.len() < 2 || !items.iter().any(|d| d.trunc_mask != 0) {
        return items;
    }

    let grid = build_spatial_grid(items.iter().enumerate().map(|(i, d)| (i, &d.detection.bbox)));
    let n = items.len();
    let mut absorbed = vec![false; n];
    let mut visited = vec![0usize; n];
    let mut epoch = 0usize;

    for i in 0..n {
        if items[i].trunc_mask == 0 {
            continue;
        }
        let area_i = items[i].detection.bbox.area();
        let min_parent_conf = ABSORB_CONF_FLOOR.max(items[i].detection.confidence * ABSORB_CONF_RATIO);

        epoch += 1;
        let (gx0, gx1, gy0, gy1) = cell_bounds(&items[i].detection.bbox);
        'cell_scan: for gx in gx0..=gx1 {
            for gy in gy0..=gy1 {
                if let Some(indices) = grid.get(&(gx, gy)) {
                    for &j in indices {
                        if j == i || visited[j] == epoch {
                            continue;
                        }
                        visited[j] = epoch;
                        let parent = &items[j];
                        if parent.trunc_mask != 0
                            || (!parent.was_merged && parent.tile_origin == items[i].tile_origin)
                            || parent.detection.confidence < min_parent_conf
                            || parent.detection.bbox.area() <= area_i * ABSORB_AREA_RATIO
                        {
                            continue;
                        }
                        let metrics = calculate_overlap_metrics(
                            &items[i].detection.bbox,
                            &parent.detection.bbox,
                        );
                        if metrics.ios >= ABSORB_IOS_MIN {
                            absorbed[i] = true;
                            break 'cell_scan;
                        }
                    }
                }
            }
        }
    }

    items
        .into_iter()
        .enumerate()
        .filter_map(|(i, d)| (!absorbed[i]).then_some(d))
        .collect()
}

/// Pass 3: Multi-cell spatial grid NMS supporting IoU and cross-tile containment suppression.
pub fn run_hybrid_grid_nms(mut items: Vec<RawDetection>, iou_threshold: f32) -> Vec<Detection> {
    items.sort_unstable_by(|a, b| {
        let sa = if a.trunc_mask != 0 {
            a.detection.confidence * TRUNC_SCORE_FACTOR
        } else {
            a.detection.confidence
        };
        let sb = if b.trunc_mask != 0 {
            b.detection.confidence * TRUNC_SCORE_FACTOR
        } else {
            b.detection.confidence
        };
        sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
    });

    let n = items.len();
    let grid = build_spatial_grid(items.iter().enumerate().map(|(i, d)| (i, &d.detection.bbox)));
    let mut suppressed = vec![false; n];
    let mut visited = vec![0usize; n];
    let mut epoch = 0usize;
    let mut kept = Vec::new();

    for i in 0..n {
        if suppressed[i] {
            continue;
        }
        let current = &items[i];
        kept.push(current.detection.clone());

        epoch += 1;
        let (gx0, gx1, gy0, gy1) = cell_bounds(&current.detection.bbox);
        for gx in gx0..=gx1 {
            for gy in gy0..=gy1 {
                if let Some(indices) = grid.get(&(gx, gy)) {
                    for &j in indices {
                        if j <= i || suppressed[j] || visited[j] == epoch {
                            continue;
                        }
                        visited[j] = epoch;
                        let candidate = &items[j];
                        let metrics = calculate_overlap_metrics(
                            &current.detection.bbox,
                            &candidate.detection.bbox,
                        );

                        let suppress = metrics.iou > iou_threshold
                            || (candidate.trunc_mask != 0 && metrics.ios > IOS_TRUNC_SUPPRESS)
                            || (current.tile_origin != candidate.tile_origin
                                && metrics.ioa_second > IOS_CROSS_TILE_SUPPRESS);

                        if suppress {
                            suppressed[j] = true;
                        }
                    }
                }
            }
        }
    }

    kept
}

