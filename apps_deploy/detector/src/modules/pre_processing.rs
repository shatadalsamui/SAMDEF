use anyhow::Result;
use rayon::prelude::*;
use turbojpeg::{decompress, PixelFormat};
use crate::InferenceTask;

pub fn preprocess_image(image_data: &[u8]) -> Result<Vec<f32>> {
    // Direct Decompression: Use turbojpeg to decompress the bytes into a u8 vector.
    // Pixel Format: Use RGB. Official Ultralytics YOLO models are trained on RGB.
    let header = turbojpeg::read_header(image_data)?;
    let (width, height) = (header.width as usize, header.height as usize);
    assert_eq!(width, 896, "Image width must be 896");
    assert_eq!(height, 896, "Image height must be 896");

    let pixels: Vec<u8> = decompress(image_data, PixelFormat::RGB)?.pixels;

    // Planar Transposition (The Core Fix) & Normalization
    let mut tensor = vec![0.0f32; 3 * width * height];
    let plane_size = width * height;

    // Loop through the pixels. Instead of a flat copy, place the R value at index i,
    // the G value at index i + (896*896), and the B value at index i + 2*(896*896).
    // Divide each value by 255.0 during this transposition to scale the input to the 0.0 - 1.0 range.
    pixels
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
    let mut batch = Vec::with_capacity(tensors.len() * 3 * 896 * 896);
    for tensor in tensors {
        batch.extend(tensor);
    }
    Ok(batch)
}