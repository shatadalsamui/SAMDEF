use anyhow::Result;
use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use tiff::decoder::Decoder;

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
    println!("🚀 Visualizer Running...");

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
    println!("✅ Done. Check: {}", OUTPUT_DIR);
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
    let color_type = decoder.colortype()?;
    let planar_config = decoder.get_tag(tiff::tags::Tag::PlanarConfiguration)?;
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
        RgbImage::from_raw(width, height, rgb_data).unwrap()
    } else {
        println!("Unexpected data length: expected {}, got {}", size * 3, data.len());
        return Ok(());
    };

    let red = Rgb([255, 50, 50]);
    let green = Rgb([50, 255, 50]);
    let white = Rgb([255, 255, 255]);

    for det in detections {
        let x = det.bbox.x_min as i32;
        let y = det.bbox.y_min as i32;
        let w = (det.bbox.x_max - det.bbox.x_min) as u32;
        let h = (det.bbox.y_max - det.bbox.y_min) as u32;
        let color = if det.class_id == 1 { green } else { red };

        // Draw Thin Box (1px)
        draw_hollow_rect_mut(&mut image, Rect::at(x, y).of_size(w, h), color);

        // Draw Label
        if let Some(f) = font {
            let label = format!("{}|{:.1}", det.class_id, det.confidence);
            draw_text_mut(&mut image, white, x, y - 15, PxScale { x: 15.0, y: 15.0 }, &f, &label);
        }
    }
    let output_name = format!("{}_annotated.jpg", tiff_id);
    image.save(format!("{}/{}", OUTPUT_DIR, output_name))?;
    Ok(())
}