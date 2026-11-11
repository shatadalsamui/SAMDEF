use anyhow::Result;
use crossbeam::channel::Sender;
use gdal::Dataset;
use log::{info, warn};
use std::path::Path;

use crate::modules::data::buffer_pool::TileBytePool;
use crate::modules::data::task::{InferenceTask, PipelineMessage};

pub const TILE_SIZE: usize = 1152;
const STRIDE: usize = 922; // ~ 20 percent overlap (1152 * 0.8 = 921.6)

///  send PipelineMessage::Process for each tile directly in planar [R|G|B] CHW layout
pub fn process_geotiff(
    source_path_str: &str,
    msg_sender: Sender<PipelineMessage>,
    byte_pool: &TileBytePool,
) -> Result<()> {
    // makes the images file path usable
    let clean_path_str = source_path_str
        .trim_matches(|c| c == '"' || c == '\'' || c == '\n' || c == '\r' || c == ' ');
    let path = Path::new(clean_path_str);
    info!("Processing GeoTIFF (streaming): {:?}", path);

    // canonicalize the source path of the image for consistency
    let canonical_path = match path.canonicalize() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => clean_path_str.to_string(),
    };

    // open the image as dataset using the gdal library
    let dataset = Dataset::open(path)?;
    let (width, height) = dataset.raster_size();
    let geo_transform = dataset
        .geo_transform()
        .unwrap_or([0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);// get the spatial meta data

    let band1 = dataset.rasterband(1)?; // R
    let band2 = dataset.rasterband(2)?; // G
    let band3 = dataset.rasterband(3)?; // B

    //iterates over the image for each tile and read the R G B data and converts them into a single array
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

            let window_offset = (tile_x as isize, tile_y as isize);// for position of the tile in the image
            let window_size = (TILE_SIZE, TILE_SIZE);
            let buffer_size = (TILE_SIZE, TILE_SIZE);

            let len = TILE_SIZE * TILE_SIZE;
            let mut planar_data = byte_pool.acquire();
            if planar_data.len() != len * 3 {
                planar_data.resize(len * 3, 0);
            }

            band1.read_into_slice::<u8>(
                window_offset,
                window_size,
                buffer_size,
                &mut planar_data[..len],
                None,
            )?;
            band2.read_into_slice::<u8>(
                window_offset,
                window_size,
                buffer_size,
                &mut planar_data[len..2 * len],
                None,
            )?;
            band3.read_into_slice::<u8>(
                window_offset,
                window_size,
                buffer_size,
                &mut planar_data[2 * len..3 * len],
                None,
            )?;

            let task = InferenceTask {
                image_data: planar_data,
                source_path: canonical_path.clone(),
                global_offset_x: tile_x as i32,
                global_offset_y: tile_y as i32,
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
