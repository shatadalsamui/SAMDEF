use crate::modules::label_parser::Label;
use crate::modules::labeler::{labels_for_tile, prepare_labels};
use gdal::Dataset;
use std::fs;
use std::path::{Path, PathBuf};
use turbojpeg::{Compressor, PixelFormat, Subsamp};

pub fn find_tif_images(image_dir: &str) -> Vec<PathBuf> {
    fs::read_dir(image_dir)
        .expect("Failed to read image directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "tif" {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

pub fn tile_image(
    image_path: &Path,
    images_dir: &str,
    labels_dir: &str,
    tile_size: u32,
    stride: u32,
    original_name: &str,
    image_labels: &[Label],
) -> Result<(), Box<dyn std::error::Error>> {
    let dataset = Dataset::open(image_path)?;
    let (width, height) = dataset.raster_size();

    let band_r = dataset.rasterband(1)?;
    let band_g = dataset.rasterband(2)?;
    let band_b = dataset.rasterband(3)?;

    let parsed_labels = prepare_labels(image_labels);

    let mut compressor = Compressor::new()?;
    compressor.set_subsamp(Subsamp::None)?;

    // --- FIXED QUALITY ---
    compressor.set_quality(95)?;

    let tile_cap = (tile_size * tile_size * 3) as usize;
    let mut tile_pixels: Vec<u8> = vec![0u8; tile_cap];

    let mut y = 0;
    while y < height {
        let tile_y = if y + tile_size as usize > height {
            height - tile_size as usize
        } else {
            y
        };

        let strip_window = (0isize, tile_y as isize);
        let strip_size = (width, tile_size as usize);
        let r_buffer = band_r.read_as::<u8>(strip_window, strip_size, strip_size, None)?;
        let g_buffer = band_g.read_as::<u8>(strip_window, strip_size, strip_size, None)?;
        let b_buffer = band_b.read_as::<u8>(strip_window, strip_size, strip_size, None)?;

        let r_data = r_buffer.data();
        let g_data = g_buffer.data();
        let b_data = b_buffer.data();

        let mut x = 0;
        while x < width {
            let tile_x = if x + tile_size as usize > width {
                width - tile_size as usize
            } else {
                x
            };

            let tile_width = tile_size as usize;
            let tile_height = tile_size as usize;

            let mut pixel_idx = 0;
            for row in 0..tile_height {
                let base = row * width + tile_x;
                let end = base + tile_width;
                let r_row = &r_data[base..end];
                let g_row = &g_data[base..end];
                let b_row = &b_data[base..end];
                for i in 0..tile_width {
                    tile_pixels[pixel_idx] = r_row[i];
                    tile_pixels[pixel_idx + 1] = g_row[i];
                    tile_pixels[pixel_idx + 2] = b_row[i];
                    pixel_idx += 3;
                }
            }

            let row_idx = y / stride as usize;
            let col_idx = x / stride as usize;
            let tile_filename = format!(
                "{}/{}_{}_{}.jpg",
                images_dir, original_name, row_idx, col_idx
            );

            let image = turbojpeg::Image {
                pixels: &tile_pixels[..pixel_idx],
                width: tile_width,
                pitch: tile_width * 3,
                height: tile_height,
                format: PixelFormat::RGB,
            };
            let mut output = turbojpeg::OutputBuf::new_owned();
            compressor.compress(image, &mut output)?;
            std::fs::write(&tile_filename, output.as_ref())?;

            let label_content = labels_for_tile(&parsed_labels, tile_x, tile_y, tile_size as usize);
            let label_filename = format!(
                "{}/{}_{}_{}.txt",
                labels_dir, original_name, row_idx, col_idx
            );
            std::fs::write(&label_filename, label_content)?;

            if x + stride as usize >= width {
                break;
            }
            x += stride as usize;
        }

        if y + stride as usize >= height {
            break;
        }
        y += stride as usize;
    }
    Ok(())
}
