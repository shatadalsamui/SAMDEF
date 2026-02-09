use anyhow::Result;
use crossbeam::channel::Sender;
use gdal::Dataset;
use std::path::Path;
use log::{info, warn};

use crate::modules::data::task::InferenceTask;

const TILE_SIZE: usize = 896;
const STRIDE: usize = 716;

pub fn process_geotiff(
    path: &Path,
    task_sender: Sender<InferenceTask>,
) -> Result<()> {
    info!("Processing GeoTIFF: {:?}", path);

    let dataset = Dataset::open(path)?;
    let (width, height) = dataset.raster_size();
    let geo_transform = dataset.geo_transform()?;

    let band1 = dataset.rasterband(1)?; // R
    let band2 = dataset.rasterband(2)?; // G
    let band3 = dataset.rasterband(3)?; // B

    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let mut tile_x = x;
            let mut tile_y = y;

            // Shift-back strategy
            if tile_x + TILE_SIZE > width {
                tile_x = width - TILE_SIZE;
            }
            if tile_y + TILE_SIZE > height {
                tile_y = height - TILE_SIZE;
            }

            let window_offset = (tile_x as isize, tile_y as isize);
            let window_size = (TILE_SIZE, TILE_SIZE);
            let buffer_size = (TILE_SIZE, TILE_SIZE);

            let r_buffer = band1.read_as::<u8>(window_offset, window_size, buffer_size, None)?;
            let g_buffer = band2.read_as::<u8>(window_offset, window_size, buffer_size, None)?;
            let b_buffer = band3.read_as::<u8>(window_offset, window_size, buffer_size, None)?;

            let mut interleaved_data =
                Vec::with_capacity(TILE_SIZE * TILE_SIZE * 3);

            for i in 0..(TILE_SIZE * TILE_SIZE) {
                interleaved_data.push(r_buffer.data[i]);
                interleaved_data.push(g_buffer.data[i]);
                interleaved_data.push(b_buffer.data[i]);
            }
            
            let task = InferenceTask {
                image_data: interleaved_data,
                source_path: path.to_str().unwrap_or("").to_string(),
                global_offset_x: tile_x as i32,
                global_offset_y: tile_y as i32,
                geo_transform,
            };

            if let Err(e) = task_sender.send(task) {
                warn!("Failed to send task to inference channel: {}", e);
            }

            if x + STRIDE >= width {
                break;
            }
            x += STRIDE;
        }

        if y + STRIDE >= height {
            break;
        }
        y += STRIDE;
    }

    Ok(())
}