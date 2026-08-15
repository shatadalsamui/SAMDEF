// Class ID to name mapping (fixed color per class):
// 0: long truck         (Neon Pink)
// 1: boxy truck         (Neon Green)
// 2: small vehicle      (Neon Cyan)
// 3: building           (Neon Yellow)
// 4: container          (Neon Magenta)
// 5: construction vehicle (Neon Aqua)
// 6: tank               (Neon Orange)
// 7: container lot      (Neon Blue-Green)
use ab_glyph::FontRef;
use anyhow::Result;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use tiff::decoder::Decoder;

// Neon color palette for 8 classes
// Neon color palette for 8 classes
const NEON_COLORS: [Rgb<u8>; 8] = [
    Rgb([255, 0, 128]), // Neon Pink
    Rgb([57, 255, 20]), // Neon Green
    Rgb([0, 255, 255]), // Neon Cyan
    Rgb([255, 255, 0]), // Neon Yellow
    Rgb([255, 0, 255]), // Neon Magenta
    Rgb([0, 255, 128]), // Neon Aqua
    Rgb([255, 110, 0]), // Neon Orange
    Rgb([0, 255, 200]), // Neon Blue-Green
];

#[derive(Debug, Deserialize, Clone)]
struct BBox {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct Detection {
    bbox: BBox,
    class_id: usize,
    confidence: f32,
}

#[derive(Debug, Deserialize)]
struct FinalOutput {
    source_image: String,
    detections: Vec<Detection>,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let json_dir = std::env::var("JSON_DIR").expect("JSON_DIR must be set in .env");
    let output_dir = std::env::var("OUTPUT_DIR").expect("OUTPUT_DIR must be set in .env");
    let font_path = std::env::var("FONT_PATH").expect("FONT_PATH must be set in .env");

    fs::create_dir_all(&output_dir)?;
    println!(" Visualizer Running...");

    let font_data = fs::read(&font_path).unwrap_or_else(|_| Vec::new());
    let font = FontRef::try_from_slice(&font_data).ok();

    let entries: Vec<_> = fs::read_dir(&json_dir)?.filter_map(|e| e.ok()).collect();

    let start = Instant::now();

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.to_string_lossy().ends_with("_results.json") {
            if let Err(e) = process_map(&path, &font, &output_dir) {
                eprintln!("Error processing map for {:?}: {}", path, e);
            }
        }
    });

    let duration = start.elapsed();
    println!(" Done. Check your output directory.");
    println!("Total annotation time: {:.2?}", duration);
    Ok(())
}

fn process_map(json_path: &Path, font: &Option<FontRef>, output_dir: &str) -> Result<()> {
    let output: FinalOutput = serde_json::from_reader(BufReader::new(File::open(json_path)?))?;
    let detections = output.detections;
    let image_path = Path::new(&output.source_image);

    if !image_path.exists() {
        println!("Image {} does not exist", image_path.display());
        return Ok(());
    }

    let tiff_id = image_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let file = File::open(&image_path)?;
    let mut decoder = Decoder::new(file)?;
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
            tiff::decoder::DecodingResult::U16(chunk) => {
                // Convert U16 to U8 by bit shift
                let u8_chunk: Vec<u8> = chunk.into_iter().map(|p| (p >> 8) as u8).collect();
                data.extend_from_slice(&u8_chunk);
            }
            _ => {
                println!("Unsupported chunk format for {}", image_path.display());
                return Ok(());
            }
        }
    }
    let size = (width * height) as usize;
    let mut image = if data.len() == size * 3 {
        // Planar configuration: RRR...GGG...BBB... -> RGBRGB...
        let mut rgb_data = Vec::with_capacity(size * 3);
        for i in 0..size {
            rgb_data.push(data[i]); // R
            rgb_data.push(data[i + size]); // G
            rgb_data.push(data[i + 2 * size]); // B
        }
        image::RgbImage::from_raw(width, height, rgb_data).unwrap()
    } else if data.len() == size {
        // Grayscale
        let rgb_data: Vec<u8> = data.into_iter().flat_map(|p| vec![p, p, p]).collect();
        image::RgbImage::from_raw(width, height, rgb_data).unwrap()
    } else {
        println!(
            "Unexpected data length: expected {} or {}, got {}",
            size * 3,
            size,
            data.len()
        );
        return Ok(());
    };

    for det in detections {
        let mut x = det.bbox.x_min.round() as i32;
        let mut y = det.bbox.y_min.round() as i32;
        let mut w = (det.bbox.x_max - det.bbox.x_min).round() as u32;
        let mut h = (det.bbox.y_max - det.bbox.y_min).round() as u32;

        if w == 0 {
            w = 1;
        }
        if h == 0 {
            h = 1;
        }
        if x < 0 {
            x = 0;
        }
        if y < 0 {
            y = 0;
        }
        if x as u32 + w > width {
            w = width.saturating_sub(x as u32).max(1);
        }
        if y as u32 + h > height {
            h = height.saturating_sub(y as u32).max(1);
        }

        let color = NEON_COLORS[det.class_id % NEON_COLORS.len()];
        draw_hollow_rect_mut(&mut image, Rect::at(x, y).of_size(w, h), color);
    }

    let output_name = format!("{}_annotated.png", tiff_id);
    image.save(format!("{}/{}", output_dir, output_name))?;
    Ok(())
}
