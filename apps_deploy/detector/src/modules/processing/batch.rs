use anyhow::Result;
use ndarray::{Array4};
use ort::session::Session;
use half::f16;

use crate::modules::processing::post_processing::Detection;
use crate::modules::data::task::InferenceTask;

pub fn process_batch(
    session: &mut Session,
    batch: &[InferenceTask],
) -> Result<Vec<Vec<Detection>>> {
    let batch_len = batch.len();
    
    // Preprocess
    let input_data = crate::modules::processing::pre_processing::preprocess_batch(batch)?;
    let input_tensor = Array4::<f16>::from_shape_vec((batch_len, 3, 896, 896), input_data)?;

    // Inference
    let outputs = crate::modules::processing::inference::run_inference(session, input_tensor)?;

    // Postprocess
    let mut results_per_image = Vec::with_capacity(batch_len);
    for (i, single_output_dyn) in outputs.outer_iter().enumerate() {
        let task = &batch[i];
        if let Ok(single_output_2d) = single_output_dyn.into_dimensionality::<ndarray::Ix2>() {
            let output_view = single_output_2d;

            let mut detections =
                crate::modules::processing::post_processing::parse_output(output_view);

            for det in &mut detections {
                det.bbox.x_min = (det.bbox.x_min as f64 + task.global_offset_x as f64) as f32;
                det.bbox.y_min = (det.bbox.y_min as f64 + task.global_offset_y as f64) as f32;
                det.bbox.x_max = (det.bbox.x_max as f64 + task.global_offset_x as f64) as f32;
                det.bbox.y_max = (det.bbox.y_max as f64 + task.global_offset_y as f64) as f32;
            }
            results_per_image.push(detections);
        } else {
            results_per_image.push(Vec::new());
        }
    }

    Ok(results_per_image)
}