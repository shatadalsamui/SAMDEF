use anyhow::Result;
use rayon::prelude::*;
use crate::modules::data::task::InferenceTask;

const WIDTH: usize = 896;
const HEIGHT: usize = 896;

pub fn preprocess_image(image_data: &[u8]) -> Result<Vec<f32>> {
    // The image_data is already a Vec<u8> of interleaved RGB pixels.
    // We just need to perform planar transposition and normalization.
    assert_eq!(image_data.len(), WIDTH * HEIGHT * 3, "Image data size must be correct");

    // Planar Transposition (HWC to CHW) & Normalization
    let mut tensor = vec![0.0f32; 3 * WIDTH * HEIGHT];
    let plane_size = WIDTH * HEIGHT;

    // Loop through the pixels. Instead of a flat copy, place the R value at index i,
    // the G value at index i + (WIDTH*HEIGHT), and the B value at index i + 2*(WIDTH*HEIGHT).
    // Divide each value by 255.0 during this transposition to scale the input to the 0.0 - 1.0 range.
    image_data
        .chunks_exact(3)
        .enumerate()
        .for_each(|(i, pixel)| {
            tensor[i] = pixel[0] as f32 / 255.0; // R
            tensor[i + plane_size] = pixel[1] as f32 / 255.0; // G
            tensor[i + 2 * plane_size] = pixel[2] as f32 / 255.0; // B
        });
    Ok(tensor)
}

pub fn preprocess_batch(tasks: &[InferenceTask]) -> Result<Vec<f32>> {
    // Parallel process images
    let tensors: Vec<Vec<f32>> = tasks.par_iter()
        .map(|task| preprocess_image(&task.image_data))
        .collect::<Result<Vec<_>>>()?;

    // Flatten into [batch, 3, 896, 896]
    let mut batch = Vec::with_capacity(tensors.len() * 3 * WIDTH * HEIGHT);
    for tensor in tensors {
        batch.extend(tensor);
    }
    Ok(batch)
}