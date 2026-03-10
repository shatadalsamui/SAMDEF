use anyhow::Result;
use crossbeam::channel::Sender;
use gdal::Dataset;
use log::{info, warn};
use std::path::Path;

use crate::modules::data::task::{InferenceTask, PipelineMessage};

const TILE_SIZE: usize = 896;
const STRIDE: usize = 716;

/// Streaming mode: send PipelineMessage::Process for each tile
pub fn process_geotiff(source_path_str: &str, msg_sender: Sender<PipelineMessage>) -> Result<()> {
    let clean_path_str = source_path_str
        .trim_matches(|c| c == '"' || c == '\'' || c == '\n' || c == '\r' || c == ' ');
    let path = Path::new(clean_path_str);
    info!("Processing GeoTIFF (streaming): {:?}", path);

    // Canonicalize the source path for consistency
    let canonical_path = match path.canonicalize() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => clean_path_str.to_string(),
    };

    let dataset = Dataset::open(path)?;
    let (width, height) = dataset.raster_size();
    let geo_transform = dataset
        .geo_transform()
        .unwrap_or([0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

    let band1 = dataset.rasterband(1)?; // R
    let band2 = dataset.rasterband(2)?; // G
    let band3 = dataset.rasterband(3)?; // B

    let mut tile_count = 0;

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

            // Optimized: Pre-allocate and fill interleaved_data for zero-overhead
            let len = TILE_SIZE * TILE_SIZE;
            let mut interleaved_data = vec![0u8; len * 3];

            for i in 0..len {
                interleaved_data[i * 3] = r_buffer.data[i];
                interleaved_data[i * 3 + 1] = g_buffer.data[i];
                interleaved_data[i * 3 + 2] = b_buffer.data[i];
            }

            let task = InferenceTask {
                image_data: interleaved_data,
                source_path: source_path_str.to_string(),
                global_offset_x: tile_x as i32,
                global_offset_y: tile_y as i32,
                geo_transform,
            };

            if let Err(_e) = msg_sender.send(PipelineMessage::Process(task)) {
                warn!("Failed to send PipelineMessage::Process to inference channel");
            }

            tile_count += 1;

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

    // Send EndOfFile with expected_tiles and canonicalized path
    if let Err(_e) = msg_sender.send(PipelineMessage::EndOfFile {
        source_path: canonical_path,
        geo_transform,
        width: width as u32,
        height: height as u32,
        expected_tiles: tile_count,
    }) {
        warn!("Failed to send PipelineMessage::EndOfFile");
    }

    Ok(())
}
