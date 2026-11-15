use anyhow::{anyhow, Result};
use gpui::RenderImage;
use image::{Frame, ImageBuffer, Rgba};
use smallvec::SmallVec;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

#[derive(Clone)]
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub render_image: Arc<RenderImage>,
}

fn bgra_to_loaded_image(width: u32, height: u32, bgra: Vec<u8>) -> Result<LoadedImage> {
    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, bgra)
        .ok_or_else(|| anyhow!("Failed to allocate BGRA ImageBuffer"))?;
    let render_image = Arc::new(RenderImage::new(SmallVec::from_elem(Frame::new(buffer), 1)));
    Ok(LoadedImage {
        width,
        height,
        render_image,
    })
}

fn append_tiff_chunk(dst: &mut Vec<u8>, chunk_data: DecodingResult, max_bytes: Option<usize>) -> Result<()> {
    let limit = max_bytes.unwrap_or(usize::MAX);
    match chunk_data {
        DecodingResult::U8(chunk) => {
            let take = chunk.len().min(limit);
            dst.extend_from_slice(&chunk[..take]);
        }
        DecodingResult::U16(chunk) => {
            let take = chunk.len().min(limit);
            dst.extend(chunk[..take].iter().map(|&v| (v >> 8) as u8));
        }
        _ => return Err(anyhow!("Unsupported TIFF chunk format")),
    }
    Ok(())
}

/// Load a TIFF or image file and produce a GPUI-compatible `RenderImage` in BGRA format.
pub fn load_tiff_or_image<P: AsRef<Path>>(path: P) -> Result<LoadedImage> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(anyhow!("Image file does not exist: {}", path_ref.display()));
    }

    // Try high-performance custom TIFF decoder first
    match load_tiff_file(path_ref) {
        Ok(loaded) => Ok(loaded),
        Err(e) => {
            eprintln!(
                "Fast TIFF custom decode failed ({}), falling back to image::open",
                e
            );
            let mut img = image::open(path_ref)?.to_rgba8();
            // Convert RGBA to BGRA for GPUI
            for pixel in img.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
            let (w, h) = img.dimensions();
            bgra_to_loaded_image(w, h, img.into_raw())
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
                let remaining = size.saturating_sub(plane.len());
                append_tiff_chunk(plane, decoder.read_chunk(chunk_idx)?, Some(remaining))?;
            }
        }

        if planes[0].len() == size && planes[1].len() == size && planes[2].len() == size {
            let mut bgra = Vec::with_capacity(size * 4);
            let r_plane = &planes[0];
            let g_plane = &planes[1];
            let b_plane = &planes[2];

            // Interleave into BGRA for GPUI
            for i in 0..size {
                bgra.push(b_plane[i]); // B
                bgra.push(g_plane[i]); // G
                bgra.push(r_plane[i]); // R
                bgra.push(255);        // A
            }

            return bgra_to_loaded_image(width, height, bgra);
        }
    }

    // Case 2: Interleaved Chunky RGB or Grayscale
    let mut raw_bytes = Vec::with_capacity(size * 3);
    for i in 0..chunk_count {
        append_tiff_chunk(&mut raw_bytes, decoder.read_chunk(i)?, None)?;
    }

    let mut bgra = Vec::with_capacity(size * 4);
    if raw_bytes.len() >= size * 3 {
        for chunk in raw_bytes.chunks_exact(3).take(size) {
            bgra.push(chunk[2]); // B
            bgra.push(chunk[1]); // G
            bgra.push(chunk[0]); // R
            bgra.push(255);      // A
        }
    } else if raw_bytes.len() >= size {
        for &p in raw_bytes.iter().take(size) {
            bgra.push(p);
            bgra.push(p);
            bgra.push(p);
            bgra.push(255);
        }
    } else {
        return Err(anyhow!("Unexpected data length in TIFF: {}", raw_bytes.len()));
    }

    bgra_to_loaded_image(width, height, bgra)
}
