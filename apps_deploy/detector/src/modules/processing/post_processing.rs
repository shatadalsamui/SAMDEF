use half::f16;
use ndarray::ArrayView2;
use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Detection {
    pub bbox: BoundingBox,
    pub class_id: usize,
    pub confidence: f32,
}

/// Helper: Calculate Intersection over Union (IoU)
fn calculate_iou(box_a: &BoundingBox, box_b: &BoundingBox) -> f32 {
    let x_a = box_a.x_min.max(box_b.x_min);
    let y_a = box_a.y_min.max(box_b.y_min);
    let x_b = box_a.x_max.min(box_b.x_max);
    let y_b = box_a.y_max.min(box_b.y_max);

    let inter_area = (x_b - x_a).max(0.0) * (y_b - y_a).max(0.0);

    let box_a_area = (box_a.x_max - box_a.x_min) * (box_a.y_max - box_a.y_min);
    let box_b_area = (box_b.x_max - box_b.x_min) * (box_b.y_max - box_b.y_min);

    let union_area = box_a_area + box_b_area - inter_area;
    if union_area > 0.0 {
        inter_area / union_area
    } else {
        0.0
    }
}

/// Effectively O(n) Spatial Grid NMS with 3x3 Neighborhood Overlap
pub fn non_maximum_suppression(detections: &mut Vec<Detection>, iou_threshold: f32) {
    use rayon::prelude::*;
    use std::cmp::Ordering;
    use std::collections::HashMap;

    if detections.is_empty() {
        return;
    }

    // Grid size must be larger than your largest object (e.g., 1000px)
    const GRID_SIZE: f32 = 1000.0;

    // 1. Group by class to allow parallel execution
    let all_detections = std::mem::take(detections);
    let mut class_map: HashMap<usize, Vec<Detection>> = HashMap::new();
    for d in all_detections {
        class_map.entry(d.class_id).or_default().push(d);
    }

    // 2. Process classes in parallel using Rayon
    *detections = class_map
        .into_par_iter()
        .flat_map(|(_, mut class_dets)| {
            // Sort by confidence (High to Low): O(n log n)
            class_dets.sort_unstable_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(Ordering::Equal)
            });

            let n = class_dets.len();
            let mut suppressed = vec![false; n];
            let mut kept = Vec::new();

            // 3. Build Spatial Grid for this class
            // Maps (grid_x, grid_y) -> Indices of boxes in that cell
            let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
            for (idx, det) in class_dets.iter().enumerate() {
                let gx = (det.bbox.x_min / GRID_SIZE).floor() as i32;
                let gy = (det.bbox.y_min / GRID_SIZE).floor() as i32;
                grid.entry((gx, gy)).or_default().push(idx);
            }

            // 4. Run NMS with 3x3 Neighborhood Search
            for i in 0..n {
                if suppressed[i] {
                    continue;
                }
                let current_det = &class_dets[i];
                kept.push(current_det.clone());

                let gx = (current_det.bbox.x_min / GRID_SIZE).floor() as i32;
                let gy = (current_det.bbox.y_min / GRID_SIZE).floor() as i32;

                // Check 3x3 neighborhood (9 cells) to handle overlaps/boundaries
                for nx in (gx - 1)..=(gx + 1) {
                    for ny in (gy - 1)..=(gy + 1) {
                        if let Some(indices) = grid.get(&(nx, ny)) {
                            for &j in indices {
                                // Only check lower-confidence boxes that aren't already suppressed
                                if j > i && !suppressed[j] {
                                    if calculate_iou(&current_det.bbox, &class_dets[j].bbox)
                                        > iou_threshold
                                    {
                                        suppressed[j] = true; // O(1) update
                                    }
                                }
                            }
                        }
                    }
                }
            }
            kept
        })
        .collect();
}

/// Parses the Raw Output from the ONNX Model
/// Format: [Batch, 1000, 6] -> [x_min, y_min, x_max, y_max, confidence, class_id]
pub fn parse_output(output: ArrayView2<f16>) -> Vec<Detection> {
    let mut detections = Vec::new();
    const IMG_SIZE: f32 = 896.0;
    const MAX_COORD: f32 = IMG_SIZE - 1.0;

    // CONFIDENCE THRESHOLDS (0.0 to 1.0)
    const CLASS_THRESHOLDS: [f32; 8] = [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];

    for proposal in output.rows() {
        let confidence = proposal[4].to_f32();
        let class_id = proposal[5].to_f32() as usize;

        if class_id < CLASS_THRESHOLDS.len() && confidence > CLASS_THRESHOLDS[class_id] {
            let x_min = proposal[0].to_f32().clamp(0.0, MAX_COORD);
            let y_min = proposal[1].to_f32().clamp(0.0, MAX_COORD);
            let mut x_max = proposal[2].to_f32().clamp(0.0, MAX_COORD);
            let mut y_max = proposal[3].to_f32().clamp(0.0, MAX_COORD);

            if x_max <= x_min {
                x_max = (x_min + 1.0).min(MAX_COORD);
            }
            if y_max <= y_min {
                y_max = (y_min + 1.0).min(MAX_COORD);
            }

            detections.push(Detection {
                bbox: BoundingBox {
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                },
                class_id,
                confidence,
            });
        }
    }
    detections
}
