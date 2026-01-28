use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use gdal::Dataset;
use turbojpeg::{Compressor, Subsamp, PixelFormat};
use indicatif::ProgressBar;
use anyhow::Result;
use prost::Message;

// Import the data structures from image_util.rs
use crate::modules::image_util::{TileMetadata, InferenceManifest};
use crate::inference_manifest;

/// Slices a GeoTIFF into overlapping JPG tiles and generates a manifest.
/// - image_path: path to the input .tif/.tiff file
/// - output_dir: directory to save tiles and manifest
/// - tile_size: size of each tile (e.g., 896)
pub fn process_inference_image(
    image_path: &Path,
    tiles_output_dir: &str,
    manifest_output_dir: &str,
    tile_size: u32,
    stride: u32,
    jpeg_quality: i32,
    pb: Arc<Mutex<ProgressBar>>,
) -> Result<InferenceManifest> {
    // Open the GeoTIFF using GDAL
    let dataset = Dataset::open(image_path)?;
    let (width_usize, height_usize) = dataset.raster_size();
    let width = width_usize as u32;
    let height = height_usize as u32;
    let geo_transform = dataset.geo_transform()?;
    let geo_transform_arr = [
        geo_transform[0], geo_transform[1], geo_transform[2],
        geo_transform[3], geo_transform[4], geo_transform[5],
    ];

    // Prepare bands for RGB extraction
    let band_r = dataset.rasterband(1)?;
    let band_g = dataset.rasterband(2)?;
    let band_b = dataset.rasterband(3)?;

    // Stride and JPEG quality are now parameters, matching ingestor architecture

    let stem = image_path.file_stem().unwrap().to_str().unwrap();
    let mut tiles = Vec::new();

    // Move compressor and output buffer outside loops for reuse
    let mut compressor = Compressor::new()?;
    compressor.set_subsamp(Subsamp::None)?;
    compressor.set_quality(jpeg_quality)?;
    let mut output = turbojpeg::OutputBuf::new_owned();

    let mut local_progress = 0;

    let mut row_idx: u32 = 0;
    let mut y: u32 = 0;
    while y < height {
        let tile_y = if y + tile_size > height {
            height - tile_size
        } else {
            y
        };
        let tile_height = tile_size.min(height - tile_y);

        // Read a full horizontal strip for this row
        let strip_window = (0isize, tile_y as isize);
        let strip_size = (width as usize, tile_height as usize);
        let r_strip = band_r.read_as::<u8>(strip_window, strip_size, strip_size, None)?.data;
        let g_strip = band_g.read_as::<u8>(strip_window, strip_size, strip_size, None)?.data;
        let b_strip = band_b.read_as::<u8>(strip_window, strip_size, strip_size, None)?.data;

        let mut col_idx: u32 = 0;
        let mut x: u32 = 0;
        let mut last_tile_x: Option<u32> = None;
        while x < width {
            let tile_x = if x + tile_size > width {
                width - tile_size
            } else {
                x
            };
            let tile_width = tile_size.min(width - tile_x);

            // Skip if this tile_x is the same as the last one (duplicate)
            if Some(tile_x) == last_tile_x {
                if x + stride >= width {
                    break;
                }
                x += stride;
                col_idx += 1;
                continue;
            }
            last_tile_x = Some(tile_x);

            // Slice out the tile from the strip
            let mut tile_pixels = Vec::with_capacity((tile_width * tile_height * 3) as usize);
            for row in 0..tile_height as usize {
                let base = row * width as usize + tile_x as usize;
                let end = base + tile_width as usize;
                let r_row = &r_strip[base..end];
                let g_row = &g_strip[base..end];
                let b_row = &b_strip[base..end];
                for i in 0..tile_width as usize {
                    tile_pixels.push(r_row[i]);
                    tile_pixels.push(g_row[i]);
                    tile_pixels.push(b_row[i]);
                }
            }

            // Save JPG tile
            let filename = format!("{}_{}_{}.jpg", stem, row_idx, col_idx);
            let out_path = Path::new(tiles_output_dir).join(&filename);
            let image = turbojpeg::Image {
                pixels: &tile_pixels[..],
                width: tile_width as usize,
                pitch: tile_width as usize * 3,
                height: tile_height as usize,
                format: PixelFormat::RGB,
            };
            compressor.compress(image, &mut output)?;
            fs::write(&out_path, output.as_ref())?;

            tiles.push(TileMetadata {
                filename: filename.clone(),
                row_idx: row_idx as usize,
                col_idx: col_idx as usize,
                x_offset: tile_x,
                y_offset: tile_y,
                width: tile_width,
                height: tile_height,
            });

            local_progress += 1;
            if local_progress % 10 == 0 {
                pb.lock().unwrap().inc(10);
            }

            if x + stride >= width {
                break;
            }
            x += stride;
            col_idx += 1;
        }
        if y + stride >= height {
            break;
        }
        y += stride;
        row_idx += 1;
    }

    // Flush remaining progress
    if local_progress % 10 != 0 {
        pb.lock().unwrap().inc(local_progress % 10);
    }

    // Write manifest as protobuf
    let manifest = InferenceManifest {
        source_image: stem.to_string(),
        source_width: width,
        source_height: height,
        geo_transform: geo_transform_arr,
        tiles,
    };

    // Convert to protobuf types
    let pb_tiles: Vec<inference_manifest::TileMetadata> = manifest.tiles.iter().map(|t| {
        inference_manifest::TileMetadata {
            filename: t.filename.clone(),
            row_idx: t.row_idx as u32,
            col_idx: t.col_idx as u32,
            x_offset: t.x_offset,
            y_offset: t.y_offset,
            width: t.width,
            height: t.height,
        }
    }).collect();

    let pb_manifest = inference_manifest::InferenceManifest {
        source_image: manifest.source_image.clone(),
        source_width: manifest.source_width,
        source_height: manifest.source_height,
        geo_transform: manifest.geo_transform.to_vec(),
        tiles: pb_tiles,
    };

    let mut buf = Vec::new();
    pb_manifest.encode(&mut buf).map_err(|e| anyhow::anyhow!(e))?;
    let manifest_path = Path::new(manifest_output_dir).join(format!("{}_manifest.pb", stem));
    fs::write(manifest_path, buf)?;
    Ok(manifest)
}