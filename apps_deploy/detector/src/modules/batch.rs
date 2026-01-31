use anyhow::Result;
use ndarray::Array;
use ort::session::Session;
use std::collections::HashMap;

use crate::modules::post_processing::Detection;
use crate::modules::task::InferenceTask;
use crate::modules::utils::extract_tiff_id;

pub fn process_batch(
    session: &mut Session,
    batch: &mut Vec<InferenceTask>,
    aggregator: &mut HashMap<String, Vec<Detection>>,
) -> Result<()> {
    let batch_len = batch.len();
    
    // Preprocess
    let input_data = crate::modules::pre_processing::preprocess_batch(batch)?;
    let input_tensor = Array::from_shape_vec(ndarray::IxDyn(&[batch_len, 3, 896, 896]), input_data)?;

    // Inference
    let outputs = crate::modules::inference::run_inference(session, input_tensor)?;

    // Postprocess
    for (i, single_output_dyn) in outputs.outer_iter().enumerate() {
        let task = &batch[i];
        if let Ok(single_output_2d) = single_output_dyn.into_dimensionality::<ndarray::Ix2>() {
            // No Transpose: The model output is already [300, 6]
            let output_view = single_output_2d;

            let mut detections =
                crate::modules::post_processing::parse_output(output_view, &task.tile_filename);

            for det in &mut detections {
                det.bbox.x_min += task.global_offset_x;
                det.bbox.y_min += task.global_offset_y;
                det.bbox.x_max += task.global_offset_x;
                det.bbox.y_max += task.global_offset_y;
            }

            let tiff_id = extract_tiff_id(&task.tile_filename);
            aggregator.entry(tiff_id).or_default().extend(detections);
        }
    }

    batch.clear();
    Ok(())
}