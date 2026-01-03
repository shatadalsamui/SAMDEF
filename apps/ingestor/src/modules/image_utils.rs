use std::fs;
use std::path::{Path, PathBuf};
use gdal::Dataset;
use image::{ImageBuffer, RgbImage};
use turbojpeg::{Compressor, Subsamp, PixelFormat};

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
    output_dir: &str,
    tile_size: u32,
    stride: u32,
    original_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open the image
    let dataset = Dataset::open(image_path)?;
    let raster_size = dataset.raster_size();
    let (width, height) = (raster_size.0, raster_size.1);

    // For RGB images, read 3 bands
    let band_r = dataset.rasterband(1)?;
    let band_g = dataset.rasterband(2)?;
    let band_b = dataset.rasterband(3)?;

    // No logs inside loops or per image
    // Reuse compressor to avoid per-tile setup cost
    let mut compressor = Compressor::new()?;
    compressor.set_subsamp(Subsamp::None)?;
    compressor.set_quality(70)?;

    let mut y = 0;
    while y < height {
        let tile_y = if y + tile_size as usize > height {
            height - tile_size as usize
        } else {
            y
        };
        let mut x = 0;
        while x < width {
            let tile_x = if x + tile_size as usize > width {
                width - tile_size as usize
            } else {
                x
            };

            let tile_width = tile_size as usize;
            let tile_height = tile_size as usize;

            // Read tile data for each band
            let buf_r = band_r.read_as::<u8>(
                (tile_x as isize, tile_y as isize),
                (tile_width, tile_height),
                (tile_width, tile_height),
                None,
            )?.data;
            let buf_g = band_g.read_as::<u8>(
                (tile_x as isize, tile_y as isize),
                (tile_width, tile_height),
                (tile_width, tile_height),
                None,
            )?.data;
            let buf_b = band_b.read_as::<u8>(
                (tile_x as isize, tile_y as isize),
                (tile_width, tile_height),
                (tile_width, tile_height),
                None,
            )?.data;

            // Combine bands into an RGB image
            let mut img_buf = Vec::with_capacity(tile_width * tile_height * 3);
            for i in 0..(tile_width * tile_height) {
                img_buf.push(buf_r[i]);
                img_buf.push(buf_g[i]);
                img_buf.push(buf_b[i]);
            }
            let img: RgbImage = ImageBuffer::from_vec(tile_width as u32, tile_height as u32, img_buf)
                .expect("Failed to create image buffer");

            // Save tile using turbojpeg for fastest encoding
            let row = tile_y / stride as usize;
            let col = tile_x / stride as usize;
            let tile_filename = format!("{}/{}_{}_{}.jpg", output_dir, original_name, row, col);
            let image = turbojpeg::Image {
                pixels: img.as_raw().as_slice(),
                width: img.width() as usize,
                pitch: img.width() as usize * 3,
                height: img.height() as usize,
                format: PixelFormat::RGB,
            };
            let mut output = turbojpeg::OutputBuf::new_owned();
            compressor.compress(image, &mut output)?;
            std::fs::write(&tile_filename, output.as_ref())?;

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