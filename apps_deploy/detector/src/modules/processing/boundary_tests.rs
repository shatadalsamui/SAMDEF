use crate::modules::processing::geometry::{
    compute_local_trunc_mask, finalize_trunc_mask_for_image, BoundingBox, Detection, RawDetection,
    TRUNC_BOTTOM, TRUNC_LEFT, TRUNC_RIGHT, TRUNC_TOP,
};
use crate::modules::processing::post_processing::{create_nms_pool, resolve_tiled_detections_in_pool};

fn make_raw(
    coords: (f32, f32, f32, f32),
    class_id: usize,
    confidence: f32,
    tile_origin: (i32, i32),
    trunc_mask: u8,
) -> RawDetection {
    RawDetection::from_detection(
        Detection {
            bbox: BoundingBox {
                x_min: coords.0,
                y_min: coords.1,
                x_max: coords.2,
                y_max: coords.3,
            },
            class_id,
            confidence,
        },
        tile_origin,
        trunc_mask,
    )
}

fn resolve(dets: Vec<RawDetection>, w: usize, h: usize) -> Vec<Detection> {
    let pool = create_nms_pool();
    resolve_tiled_detections_in_pool(&pool, dets, w, h, 0.45)
}

#[test]
fn test_suppresses_cut_sliver_inside_full_box_low_iou() {
    let full = make_raw((1135.0, 500.0, 1185.0, 540.0), 0, 0.80, (922, 0), 0);
    let sliver = make_raw((1135.0, 500.0, 1152.0, 540.0), 0, 0.65, (0, 0), TRUNC_RIGHT);
    let out = resolve(vec![sliver, full], 2074, 1152);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.x_max - 1185.0).abs() < 1e-3);
}

#[test]
fn test_priority_inversion_complete_beats_higher_conf_cut_sliver() {
    let sliver = make_raw((1130.0, 400.0, 1152.0, 440.0), 1, 0.90, (0, 0), TRUNC_RIGHT);
    let full = make_raw((1130.0, 400.0, 1190.0, 440.0), 1, 0.55, (922, 0), 0);
    let out = resolve(vec![sliver, full], 2074, 1152);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.x_max - 1190.0).abs() < 1e-3);
    assert!((out[0].confidence - 0.55).abs() < 1e-3);
}

#[test]
fn test_noise_guardrail_protects_real_truncated_from_002_giant_box() {
    let real_trunc = make_raw((1100.0, 500.0, 1152.0, 540.0), 0, 0.85, (0, 0), TRUNC_RIGHT);
    let garbage = make_raw((950.0, 300.0, 1500.0, 800.0), 0, 0.02, (922, 0), 0);
    let out = resolve(vec![real_trunc, garbage], 2074, 1152);
    // Real truncated box must NOT be absorbed by a 0.02 confidence box; garbage stays untouched (0.0075 IoU).
    assert_eq!(out.len(), 2);
    assert!(out.iter().any(|d| (d.confidence - 0.85).abs() < 1e-3));
}

#[test]
fn test_solitary_truncated_object_preserved_for_recall() {
    let only_edge = make_raw((1120.0, 500.0, 1151.5, 540.0), 0, 0.78, (0, 0), TRUNC_RIGHT);
    let out = resolve(vec![only_edge], 2074, 1152);
    assert_eq!(out.len(), 1);
    assert!((out[0].confidence - 0.78).abs() < 1e-3);
}

#[test]
fn test_horizontal_seam_merge_for_oversized_ship() {
    let left_half = make_raw((800.0, 500.0, 1151.0, 580.0), 4, 0.75, (0, 0), TRUNC_RIGHT);
    let right_half = make_raw((922.0, 502.0, 1250.0, 578.0), 4, 0.68, (922, 0), TRUNC_LEFT);
    let out = resolve(vec![left_half, right_half], 2074, 1152);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.x_min - 800.0).abs() < 1e-3);
    assert!((out[0].bbox.x_max - 1250.0).abs() < 1e-3);
    assert!((out[0].confidence - 0.75).abs() < 1e-3);
}

#[test]
fn test_vertical_seam_merge_for_tall_object() {
    let top_half = make_raw((400.0, 750.0, 480.0, 1151.0), 7, 0.72, (0, 0), TRUNC_BOTTOM);
    let bot_half = make_raw((401.0, 922.0, 479.0, 1300.0), 7, 0.80, (0, 922), TRUNC_TOP);
    let out = resolve(vec![top_half, bot_half], 1152, 2074);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.y_min - 750.0).abs() < 1e-3);
    assert!((out[0].bbox.y_max - 1300.0).abs() < 1e-3);
    assert!((out[0].confidence - 0.80).abs() < 1e-3);
}

#[test]
fn test_four_way_corner_cut_reconstructs_single_building() {
    let q1 = make_raw((850.0, 850.0, 1151.0, 1151.0), 7, 0.70, (0, 0), TRUNC_RIGHT | TRUNC_BOTTOM);
    let q2 = make_raw((922.0, 850.0, 1250.0, 1151.0), 7, 0.74, (922, 0), TRUNC_LEFT | TRUNC_BOTTOM);
    let q3 = make_raw((850.0, 922.0, 1151.0, 1260.0), 7, 0.69, (0, 922), TRUNC_RIGHT | TRUNC_TOP);
    let q4 = make_raw((922.0, 922.0, 1250.0, 1260.0), 7, 0.78, (922, 922), TRUNC_LEFT | TRUNC_TOP);
    let out = resolve(vec![q1, q2, q3, q4], 2074, 2074);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.x_min - 850.0).abs() < 1e-3);
    assert!((out[0].bbox.y_min - 850.0).abs() < 1e-3);
    assert!((out[0].bbox.x_max - 1250.0).abs() < 1e-3);
    assert!((out[0].bbox.y_max - 1260.0).abs() < 1e-3);
}

#[test]
fn test_side_by_side_cars_at_seam_do_not_falsely_merge() {
    let car1_t1 = make_raw((1105.0, 400.0, 1149.0, 430.0), 0, 0.80, (0, 0), TRUNC_RIGHT);
    let car1_t2 = make_raw((1105.0, 400.0, 1149.0, 430.0), 0, 0.82, (922, 0), 0);
    let car2_t2 = make_raw((1156.0, 400.0, 1200.0, 430.0), 0, 0.84, (922, 0), 0);
    let out = resolve(vec![car1_t1, car1_t2, car2_t2], 2074, 1152);
    assert_eq!(out.len(), 2);
}

#[test]
fn test_wide_box_over_1024_px_deduplicates_across_grid_cells() {
    let wide = make_raw((900.0, 100.0, 2200.0, 500.0), 10, 0.88, (0, 0), 0);
    let dup = make_raw((2050.0, 110.0, 2190.0, 490.0), 10, 0.60, (1844, 0), TRUNC_RIGHT);
    let out = resolve(vec![wide, dup], 3000, 1152);
    assert_eq!(out.len(), 1);
    assert!((out[0].bbox.x_min - 900.0).abs() < 1e-3);
}

#[test]
fn test_outer_geotiff_edge_is_not_flagged_truncated() {
    let local = BoundingBox {
        x_min: 1088.0,
        y_min: 200.0,
        x_max: 1151.0,
        y_max: 260.0,
    };
    let mut raw = vec![make_raw(
        (2010.0, 200.0, 2073.0, 260.0),
        7,
        0.82,
        (922, 0),
        compute_local_trunc_mask(&local, (922, 0)),
    )];
    finalize_trunc_mask_for_image(&mut raw, 2074, 1152);
    assert_eq!(raw[0].trunc_mask, 0);
}
