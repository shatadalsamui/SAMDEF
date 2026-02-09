// Class ID to name mapping (fixed color per class):
// 0: long truck         (Neon Pink)
// 1: boxy truck         (Neon Green)
// 2: small vehicle      (Neon Cyan)
// 3: building           (Neon Yellow)
// 4: container          (Neon Magenta)
// 5: construction vehicle (Neon Aqua)
// 6: tank               (Neon Orange)
// 7: container lot      (Neon Blue-Green)
use anyhow::Result;
use ab_glyph::{FontRef, PxScale};
use image::{Rgb};
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use turbojpeg::{Compressor, Subsamp, PixelFormat, Image, OutputBuf};
use tiff::decoder::Decoder;

// Neon color palette for 8 classes
const NEON_COLORS: [Rgb<u8>; 8] = [
    Rgb([255, 0, 128]),   // Neon Pink
    Rgb([57, 255, 20]),   // Neon Green
    Rgb([0, 255, 255]),   // Neon Cyan
    Rgb([255, 255, 0]),   // Neon Yellow
    Rgb([255, 0, 255]),   // Neon Magenta
    Rgb([0, 255, 128]),   // Neon Aqua
    Rgb([255, 110, 0]),   // Neon Orange
    Rgb([0, 255, 200]),   // Neon Blue-Green
];

// --- CONFIGURATION ---
const TIFF_DIR: &str = "/home/shatadal/SAMDEF_DATA/val_images";
const JSON_DIR: &str = "/home/shatadal/SAMDEF/raw_data/inference/results";
const OUTPUT_DIR: &str = "/home/shatadal/SAMDEF/raw_data/inference/annotated";
const FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
// ---------------------

#[derive(Debug, Deserialize, Clone)]
struct BBox {
    x_min: f32, y_min: f32, x_max: f32, y_max: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct Detection {
    bbox: BBox,
    class_id: usize,
    confidence: f32,
}

fn main() -> Result<()> {
    fs::create_dir_all(OUTPUT_DIR)?;
    println!(" Visualizer Running...");

    // Try load font
    let font_data = fs::read(FONT_PATH).unwrap_or(Vec::new());
    let font = FontRef::try_from_slice(&font_data).ok();

    for entry in fs::read_dir(JSON_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.to_string_lossy().ends_with("_manifest.json") {
            process_map(&path, &font)?;
        }
    }
    println!(" Done. Check: {}", OUTPUT_DIR);
    Ok(())
}

fn process_map(json_path: &Path, font: &Option<FontRef>) -> Result<()> {
    let filename = json_path.file_name().unwrap().to_string_lossy();
    let tiff_id = filename.replace("_manifest.json", "");
    let detections: Vec<Detection> = serde_json::from_reader(BufReader::new(File::open(json_path)?))?;

    let image_path = format!("{}/{}.tif", TIFF_DIR, tiff_id);
    if !Path::new(&image_path).exists() {
        println!("Image {} does not exist", image_path);
        return Ok(());
    }

    let file = File::open(&image_path)?;
    let mut decoder = Decoder::new(&file)?;
    let (width, height) = decoder.dimensions()?;
    let _color_type = decoder.colortype()?;
    let _planar_config = decoder.get_tag(tiff::tags::Tag::PlanarConfiguration)?;
    let chunk_count = if decoder.get_chunk_type() == tiff::decoder::ChunkType::Strip {
        decoder.strip_count()?
    } else {
        decoder.tile_count()?
    };
    let mut data = Vec::new();
    for i in 0..chunk_count {
        let chunk_data = decoder.read_chunk(i)?;
        match chunk_data {
            tiff::decoder::DecodingResult::U8(chunk) => data.extend_from_slice(&chunk),
            _ => {
                println!("Unsupported chunk format for {}", image_path);
                return Ok(());
            }
        }
    }
    let size = (width * height) as usize;
    let mut image = if data.len() == size * 3 {
        // Planar configuration: RRR...GGG...BBB...
        let mut rgb_data = Vec::with_capacity(size * 3);
        for i in 0..size {
            rgb_data.push(data[i]); // R
            rgb_data.push(data[i + size]); // G
            rgb_data.push(data[i + 2 * size]); // B
        }
        image::RgbImage::from_raw(width, height, rgb_data).unwrap()
    } else {
        println!("Unexpected data length: expected {}, got {}", size * 3, data.len());
        return Ok(());
    };

    let white = Rgb([255, 255, 255]);

    for det in detections {
        // Round to nearest pixel and clamp to image bounds to avoid corner drift.
        let mut x = det.bbox.x_min.round() as i32;
        let mut y = det.bbox.y_min.round() as i32;
        let mut w = (det.bbox.x_max - det.bbox.x_min).round() as u32;
        let mut h = (det.bbox.y_max - det.bbox.y_min).round() as u32;

        // Enforce minimum size of 1x1 and clamp box within the image.
        if w == 0 { w = 1; }
        if h == 0 { h = 1; }
        if x < 0 { x = 0; }
        if y < 0 { y = 0; }
        if x as u32 + w > width { w = width.saturating_sub(x as u32).max(1); }
        if y as u32 + h > height { h = height.saturating_sub(y as u32).max(1); }

        // Use fixed neon color for each class (see mapping above)
        let color = NEON_COLORS[det.class_id % NEON_COLORS.len()];

        // Draw Thin Box (1px)
        draw_hollow_rect_mut(&mut image, Rect::at(x, y).of_size(w, h), color);

        // // --- LABEL DRAWING BLOCK: comment out this block to hide class/conf labels ---
        // if let Some(f) = font {
        //     let label = format!("{}|{:.1}", det.class_id, det.confidence);
        //     let font_size = 10.0;
        //     let scale = PxScale { x: font_size, y: font_size };
        //     let label_width = (label.len() as f32 * font_size * 0.6).ceil() as u32;
        //     let label_height = (font_size * 1.2).ceil() as u32;
        //     let mut label_y = y - label_height as i32 - 2;
        //     if label_y < 0 { label_y = y + h as i32 + 2; }
        //     let mut label_x = x;
        //     if label_x < 0 { label_x = 0; }
        //     if (label_x as u32 + label_width) > width { label_x = width.saturating_sub(label_width) as i32; }
        //     let bg_rect = Rect::at(label_x, label_y).of_size(label_width, label_height);
        //     let bg_color = Rgb([
        //         color[0].saturating_add(60).min(255),
        //         color[1].saturating_add(60).min(255),
        //         color[2].saturating_add(60).min(255),
        //     ]);
        //     imageproc::drawing::draw_filled_rect_mut(&mut image, bg_rect, bg_color);
        //     draw_text_mut(&mut image, Rgb([0, 0, 0]), label_x, label_y, scale, &f, &label);
        // }
        // // --- END LABEL DRAWING BLOCK ---
    }
    let output_name = format!("{}_annotated.jpg", tiff_id);

    let mut compressor = Compressor::new()?;
    compressor.set_quality(100)?;
    compressor.set_subsamp(Subsamp::None)?; // 4:4:4 chroma subsampling for high-fidelity edges
    let tj_img = Image {
        pixels: image.as_raw().as_slice(),
        width: width as usize,
        height: height as usize,
        format: PixelFormat::RGB,
        pitch: width as usize * 3,
    };

    let mut jpeg_data = OutputBuf::new_owned();
    compressor.compress(tj_img, &mut jpeg_data)?;
    fs::write(format!("{}/{}", OUTPUT_DIR, output_name), &jpeg_data)?;
    Ok(())
}