// Run 15 Class ID to Name mapping (15 classes):
// 0: Light-Vehicle          (Neon Cyan)
// 1: Boxy-Truck             (Neon Lime)
// 2: Long-Trucks            (Neon Emerald)
// 3: Small-Boat             (Electric Blue)
// 4: Large-Ship             (Deep Indigo)
// 5: Fixed-Wing             (Bright Yellow)
// 6: Helicopter             (Neon Orange)
// 7: Building               (Neon Red)
// 8: Other-Structure        (Coral)
// 9: Storage-Tank           (Neon Purple)
// 10: Container-Lot         (Amber/Gold)
// 11: Construction-Site     (Warm Ochre)
// 12: Railway               (Mint/Teal)
// 13: Engineering-Machinery (Hot Pink)
// 14: Tower-Pylon           (Lavender)
use ab_glyph::FontRef;
use anyhow::Result;
use image::Rgb;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use tiff::decoder::Decoder;

// Neon color palette for 15 classes (Run 15)
const NEON_COLORS: [Rgb<u8>; 15] = [
    Rgb([0, 229, 255]),   // 0: Light-Vehicle (Neon Cyan)
    Rgb([57, 255, 20]),   // 1: Boxy-Truck (Neon Lime)
    Rgb([0, 255, 136]),   // 2: Long-Trucks (Neon Emerald)
    Rgb([41, 121, 255]),  // 3: Small-Boat (Electric Blue)
    Rgb([101, 31, 255]),  // 4: Large-Ship (Deep Indigo)
    Rgb([255, 255, 0]),   // 5: Fixed-Wing (Bright Yellow)
    Rgb([255, 109, 0]),   // 6: Helicopter (Neon Orange)
    Rgb([255, 23, 68]),   // 7: Building (Neon Red)
    Rgb([255, 82, 82]),   // 8: Other-Structure (Coral)
    Rgb([213, 0, 249]),   // 9: Storage-Tank (Neon Purple)
    Rgb([255, 214, 0]),   // 10: Container-Lot (Amber/Gold)
    Rgb([255, 145, 0]),   // 11: Construction-Site (Warm Ochre)
    Rgb([0, 191, 165]),   // 12: Railway (Mint/Teal)
    Rgb([245, 0, 87]),    // 13: Engineering-Machinery (Hot Pink)
    Rgb([224, 64, 251]),  // 14: Tower-Pylon (Lavender)
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
    #[allow(dead_code)]
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

    let font: Option<FontRef> = None;

    fs::create_dir_all(&output_dir)?;

    let mut entries: Vec<_> = fs::read_dir(&json_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "json"))
        .collect();

    entries.sort();

    println!(" Visualizer Running...");
    let start = Instant::now();

    entries.par_iter().for_each(|entry| {
        if let Err(e) = process_map(entry, &font, &output_dir) {
            eprintln!("Error processing {:?}: {}", entry, e);
        }
    });

    let duration = start.elapsed();
    println!(" Done. Check your output directory.");
    println!("Total annotation time: {:.2?}", duration);
    Ok(())
}

fn process_map(json_path: &Path, _font: &Option<FontRef>, output_dir: &str) -> Result<()> {
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
