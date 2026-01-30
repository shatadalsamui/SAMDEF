use ndarray::ArrayView2;
use serde::Serialize;
use std::cmp::Ordering;

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
    pub source_tile: String,
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

/// Standard Non-Maximum Suppression
pub fn non_maximum_suppression(detections: &mut Vec<Detection>, iou_threshold: f32) {
    if detections.is_empty() { return; }
    
    // Sort by confidence (High -> Low)
    detections.sort_unstable_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(Ordering::Equal));

    let mut i = 0;
    while i < detections.len() {
        let mut j = i + 1;
        while j < detections.len() {
            if detections[i].class_id == detections[j].class_id && 
               calculate_iou(&detections[i].bbox, &detections[j].bbox) > iou_threshold {
                detections.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

/// Parses the Raw Output from the ONNX Model
/// Format: [Batch, 300, 6] -> [x_min, y_min, x_max, y_max, confidence, class_id]
pub fn parse_output(output: ArrayView2<f32>, tile_filename: &str) -> Vec<Detection> {
    let mut detections = Vec::new();
    const IMG_SIZE: f32 = 896.0;
    const MAX_COORD: f32 = IMG_SIZE - 1.0;
    
    // CONFIDENCE THRESHOLDS (0.0 to 1.0)
    const CLASS_THRESHOLDS: [f32; 6] = [0.1, 0.1, 0.1, 0.1, 0.25, 0.25];

    for proposal in output.rows() {
        // Raw values: [v0, v1, v2, v3, confidence, class_id]
        let v0 = proposal[0];
        let v1 = proposal[1];
        let v2 = proposal[2];
        let v3 = proposal[3];
        let confidence = proposal[4]; // Already a probability
        let class_id = proposal[5] as usize;

        if class_id < CLASS_THRESHOLDS.len() && confidence > CLASS_THRESHOLDS[class_id] {
            // 1. Handle Normalization: If values are small (<= 1.0), scale to 896 pixels
            // Use a small epsilon (1.01) to handle floating point overflow at edges
            let (mut x_min, mut y_min, mut x_max, mut y_max) = if v2 <= 1.01 {
                // Model outputs normalized coords (0-1). Scale to tile pixels; clamp to avoid edge spill.
                (
                    v0 * MAX_COORD,
                    v1 * MAX_COORD,
                    v2 * MAX_COORD,
                    v3 * MAX_COORD,
                )
            } else {
                (v0, v1, v2, v3)
            };

            // Clamp to tile bounds and enforce a minimum 1px extent to reduce corner drift.
            x_min = x_min.clamp(0.0, MAX_COORD);
            y_min = y_min.clamp(0.0, MAX_COORD);
            x_max = x_max.clamp(0.0, MAX_COORD);
            y_max = y_max.clamp(0.0, MAX_COORD);
            if x_max <= x_min { x_max = (x_min + 1.0).min(MAX_COORD); }
            if y_max <= y_min { y_max = (y_min + 1.0).min(MAX_COORD); }

            detections.push(Detection {
                bbox: BoundingBox { x_min, y_min, x_max, y_max },
                class_id,
                confidence,
                source_tile: tile_filename.to_string(),
            });
        }
    }
    detections
}
