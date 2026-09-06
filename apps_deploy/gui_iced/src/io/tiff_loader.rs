use anyhow::{anyhow, Result};
use iced::widget::image::Handle;
use std::fs::File;
use std::path::{Path, PathBuf};
use tiff::decoder::Decoder;
use tiff::tags::Tag;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LoadedImage {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub handle: Handle,
}

/// Load a TIFF or image file and produce an iced Handle::from_pixels (RGBA).
pub fn load_tiff_or_image<P: AsRef<Path>>(path: P) -> Result<LoadedImage> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(anyhow!("File does not exist: {}", path_ref.display()));
    }

    // Try specialized fast TIFF decoder first
    match load_tiff_file(path_ref) {
        Ok(loaded) => Ok(loaded),
        Err(e) => {
            eprintln!("TIFF custom decode failed ({}), falling back to image::open", e);
            let img = image::open(path_ref)?.to_rgba8();
            let (w, h) = img.dimensions();
            let handle = Handle::from_pixels(w, h, img.into_raw());
            Ok(LoadedImage {
                path: path_ref.to_path_buf(),
                width: w,
                height: h,
                handle,
            })
        }
    }
}

fn load_tiff_file(path: &Path) -> Result<LoadedImage> {
    let file = File::open(path)?;
    let mut decoder = Decoder::new(file)?;
    let (width, height) = decoder.dimensions()?;
    let size = (width * height) as usize;

    let is_planar = match decoder.get_tag(Tag::PlanarConfiguration) {
        Ok(tiff::decoder::ifd::Value::Unsigned(v)) => v == 2,
        Ok(tiff::decoder::ifd::Value::Short(v)) => v == 2,
        Ok(tiff::decoder::ifd::Value::UnsignedBig(v)) => v == 2,
        _ => false,
    };

    let chunk_count = if decoder.get_chunk_type() == tiff::decoder::ChunkType::Strip {
        decoder.strip_count()?
    } else {
        decoder.tile_count()?
    };

    // Case 1: Separate Planar Strips (Red strips, Green strips, Blue strips)
    if is_planar && chunk_count >= 3 && chunk_count % 3 == 0 {
        let strips_per_plane = chunk_count as usize / 3;
        let mut planes: [Vec<u8>; 3] = [
            Vec::with_capacity(size),
            Vec::with_capacity(size),
            Vec::with_capacity(size),
        ];

        for (p, plane) in planes.iter_mut().enumerate() {
            for s in 0..strips_per_plane {
                let chunk_idx = (p * strips_per_plane + s) as u32;
                let chunk_data = decoder.read_chunk(chunk_idx)?;
                match chunk_data {
                    tiff::decoder::DecodingResult::U8(chunk) => {
                        let remaining = size.saturating_sub(plane.len());
                        let to_take = chunk.len().min(remaining);
                        plane.extend_from_slice(&chunk[..to_take]);
                    }
                    tiff::decoder::DecodingResult::U16(chunk) => {
                        let remaining = size.saturating_sub(plane.len());
                        let to_take = chunk.len().min(remaining);
                        plane.extend(chunk[..to_take].iter().map(|&v| (v >> 8) as u8));
                    }
                    _ => return Err(anyhow!("Unsupported TIFF chunk format")),
                }
            }
        }

        if planes[0].len() == size && planes[1].len() == size && planes[2].len() == size {
            let mut rgba = Vec::with_capacity(size * 4);
            let r_plane = &planes[0];
            let g_plane = &planes[1];
            let b_plane = &planes[2];

            for i in 0..size {
                rgba.push(r_plane[i]);
                rgba.push(g_plane[i]);
                rgba.push(b_plane[i]);
                rgba.push(255);
            }

            let handle = Handle::from_pixels(width, height, rgba);
            return Ok(LoadedImage {
                path: path.to_path_buf(),
                width,
                height,
                handle,
            });
        }
    }

    // Case 2: Interleaved Chunky RGB or Grayscale
    let mut raw_bytes = Vec::with_capacity(size * 3);
    for i in 0..chunk_count {
        let chunk_data = decoder.read_chunk(i)?;
        match chunk_data {
            tiff::decoder::DecodingResult::U8(chunk) => raw_bytes.extend_from_slice(&chunk),
            tiff::decoder::DecodingResult::U16(chunk) => {
                raw_bytes.extend(chunk.iter().map(|&v| (v >> 8) as u8));
            }
            _ => return Err(anyhow!("Unsupported TIFF chunk format")),
        }
    }

    let mut rgba = Vec::with_capacity(size * 4);
    if raw_bytes.len() >= size * 3 {
        for chunk in raw_bytes.chunks_exact(3).take(size) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255);
        }
    } else if raw_bytes.len() >= size {
        for &p in raw_bytes.iter().take(size) {
            rgba.push(p);
            rgba.push(p);
            rgba.push(p);
            rgba.push(255);
        }
    } else {
        return Err(anyhow!("Unexpected data length in TIFF: {}", raw_bytes.len()));
    }

    let handle = Handle::from_pixels(width, height, rgba);
    Ok(LoadedImage {
        path: path.to_path_buf(),
        width,
        height,
        handle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_tiff() {
        let path = "/home/shatadal/SAMDEF_DATA/val_images/1038.tif";
        let loaded = load_tiff_or_image(path).expect("load_tiff_or_image failed");
        assert_eq!(loaded.width, 3195);
        assert_eq!(loaded.height, 3215);

        // Verify color channels are distinct
        let file = File::open(path).unwrap();
        let mut decoder = Decoder::new(file).unwrap();
        let (w, h) = decoder.dimensions().unwrap();
        assert_eq!(w, 3195);
        assert_eq!(h, 3215);
    }
}


