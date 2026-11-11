use crate::modules::io::virtual_tiler::TILE_SIZE;
use crate::modules::processing::boundary_nms::{absorb_truncated_fragments, run_hybrid_grid_nms};
pub use crate::modules::processing::geometry::{
    compute_local_trunc_mask, finalize_trunc_mask_for_image, BoundingBox, Detection, RawDetection,
};
use crate::modules::processing::seam_merger::merge_seam_truncated_boxes;
use ndarray::ArrayView2;
use rayon::prelude::*;
use std::collections::HashMap;

/// Builds a bounded Rayon pool isolated from Stage 1's global preprocessing pool
/// so post-processing bursts never starve GPU batch preparation.
pub fn create_nms_pool() -> rayon::ThreadPool {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let nms_threads = (cores / 3).clamp(2, 4);
    rayon::ThreadPoolBuilder::new()
        .num_threads(nms_threads)
        .thread_name(|idx| format!("samdef-nms-{}", idx))
        .build()
        .expect("Failed to build isolated NMS thread pool")
}

fn group_by_class(raw_dets: Vec<RawDetection>) -> HashMap<usize, Vec<RawDetection>> {
    let mut class_map: HashMap<usize, Vec<RawDetection>> = HashMap::new();
    for d in raw_dets {
        class_map.entry(d.detection.class_id).or_default().push(d);
    }
    class_map
}

/// Full 3-stage boundary-aware detection resolution executed per scene.
pub fn resolve_tiled_detections(
    mut raw_dets: Vec<RawDetection>,
    img_width: usize,
    img_height: usize,
    iou_threshold: f32,
) -> Vec<Detection> {
    if raw_dets.is_empty() {
        return Vec::new();
    }

    finalize_trunc_mask_for_image(&mut raw_dets, img_width, img_height);

    group_by_class(raw_dets)
        .into_par_iter()
        .flat_map(|(_, class_dets)| {
            let merged = merge_seam_truncated_boxes(class_dets);
            let absorbed = absorb_truncated_fragments(merged);
            run_hybrid_grid_nms(absorbed, iou_threshold)
        })
        .collect()
}

/// Runs `resolve_tiled_detections` inside an isolated Rayon pool so Stage 1's global pool is never starved.
pub fn resolve_tiled_detections_in_pool(
    pool: &rayon::ThreadPool,
    raw_dets: Vec<RawDetection>,
    img_width: usize,
    img_height: usize,
    iou_threshold: f32,
) -> Vec<Detection> {
    pool.install(|| resolve_tiled_detections(raw_dets, img_width, img_height, iou_threshold))
}

/// Legacy single-slice Spatial Grid NMS entry point upgraded to multi-cell indexing.
#[allow(dead_code)]
pub fn non_maximum_suppression(detections: &mut Vec<Detection>, iou_threshold: f32) {
    if detections.is_empty() {
        return;
    }

    let raw = std::mem::take(detections)
        .into_iter()
        .map(|d| RawDetection::from_detection(d, (0, 0), 0))
        .collect::<Vec<_>>();

    *detections = group_by_class(raw)
        .into_par_iter()
        .flat_map(|(_, class_dets)| run_hybrid_grid_nms(class_dets, iou_threshold))
        .collect();
}

/// Parses the Raw Output from the ONNX Model
/// Format: [Batch, 5000, 6] -> [x_min, y_min, x_max, y_max, confidence, class_id]
pub fn parse_output(output: ArrayView2<f32>) -> Vec<Detection> {
    let mut detections = Vec::new();
    const IMG_SIZE: f32 = TILE_SIZE as f32;
    const MAX_COORD: f32 = IMG_SIZE - 1.0;

    // CONFIDENCE THRESHOLDS (0.0 to 1.0) for 15 classes:
    // 0: Light-Vehicle, 1: Boxy-Truck, 2: Long-Trucks, 3: Small-Boat, 4: Large-Ship,
    // 5: Fixed-Wing, 6: Helicopter, 7: Building, 8: Other-Structure, 9: Storage-Tank,
    // 10: Container-Lot, 11: Construction-Site, 12: Railway, 13: Engineering-Machinery, 14: Tower-Pylon
    const CLASS_THRESHOLDS: [f32; 15] = [0.01; 15];

    for proposal in output.rows() {
        let confidence = proposal[4];
        let class_id = proposal[5] as usize;

        if class_id < CLASS_THRESHOLDS.len() && confidence > CLASS_THRESHOLDS[class_id] {
            let x_min = proposal[0].clamp(0.0, MAX_COORD);
            let y_min = proposal[1].clamp(0.0, MAX_COORD);
            let mut x_max = proposal[2].clamp(0.0, MAX_COORD);
            let mut y_max = proposal[3].clamp(0.0, MAX_COORD);

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
