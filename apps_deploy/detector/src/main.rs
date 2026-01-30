use anyhow::Result;
use crossbeam::channel;
use ndarray::Array;
use ort::{ep::CUDAExecutionProvider, session::Session}; 
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use std::thread;

mod modules;
use modules::post_processing::Detection;

// CONFIG: tile stride and filename ordering.
// STRIDE: how far tiles are spaced in global coords. If tiles abut, use 896. If they overlap (e.g., 180px), keep 716.
// SWAP_RC: set true if filenames are Map_COL_ROW and not Map_ROW_COL.
const TILE_STRIDE: f32 = 716.0;
const SWAP_RC: bool = false;

#[derive(Debug)]
pub struct InferenceTask {
    pub image_data: Vec<u8>,
    pub global_offset_x: f32,
    pub global_offset_y: f32,
    pub tile_filename: String,
}

// Helper: Extract "MapA" from "MapA_05_12.jpg"
fn extract_tiff_id(filename: &str) -> String {
    let parts: Vec<&str> = filename.split('_').collect();
    if !parts.is_empty() {
        parts[0].to_string()
    } else {
        "unknown_map".to_string()
    }
}

// Helper: Calculate Global Offsets from filename
// Expected naming (from pre-processor): <stem>_<row>_<col>_x<offx>_y<offy>.jpg
// Falls back to stride-based offsets if x/y tokens are missing.
fn calculate_offsets(filename: &str) -> (f32, f32) {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parts: Vec<&str> = stem.split('_').collect();

    if parts.len() >= 5 {
        let row_part = parts[parts.len() - 4];
        let col_part = parts[parts.len() - 3];
        let x_part = parts[parts.len() - 2];
        let y_part = parts[parts.len() - 1];

        let row = row_part.parse::<f32>().ok();
        let col = col_part.parse::<f32>().ok();
        let off_x = x_part.strip_prefix('x').and_then(|v| v.parse::<f32>().ok());
        let off_y = y_part.strip_prefix('y').and_then(|v| v.parse::<f32>().ok());

        if let (Some(row), Some(col), Some(off_x), Some(off_y)) = (row, col, off_x, off_y) {
            // Respect SWAP_RC only if row/col are swapped; x/y offsets are authoritative.
            let (row_val, col_val) = if SWAP_RC { (col, row) } else { (row, col) };
            let (mut gx, mut gy) = (off_x, off_y);
            // If someone encoded row/col but not x/y, fall back to stride below.
            // With x/y present, trust them.
            if gx >= 0.0 && gy >= 0.0 {
                return (gx, gy);
            }
            let gx_stride = col_val * TILE_STRIDE;
            let gy_stride = row_val * TILE_STRIDE;
            return (gx_stride, gy_stride);
        }
    }

    // Fallback: old scheme <stem>_<row>_<col>.jpg
    if parts.len() >= 3 {
        if let (Ok(a), Ok(b)) = (
            parts[parts.len() - 2].parse::<f32>(),
            parts[parts.len() - 1].parse::<f32>(),
        ) {
            let (row, col) = if SWAP_RC { (b, a) } else { (a, b) };
            return (col * TILE_STRIDE, row * TILE_STRIDE);
        }
    }

    (0.0, 0.0)
}

fn main() -> Result<()> {
    // 1. Initialize ORT Global State
    // FIX 1: Removed `?` because in this version commit() returns bool
    let _ = ort::init()
        .with_name("SAMDEF_Detector")
        .commit();

    // 2. CONFIGURATION (ABSOLUTE PATHS)
    let input_dir = "/home/shatadal/SAMDEF/raw_data/inference/inference_tiles";
    let output_dir = "/home/shatadal/SAMDEF/raw_data/inference/results";
    
    // IMPORTANT: Make sure this file exists!
    let model_path = "/home/shatadal/SAMDEF/apps_deploy/detector/model/best.onnx";

    fs::create_dir_all(output_dir)?;

    println!(" SAMDEF Detector Starting...");
    println!(" Input: {}", input_dir);
    println!(" Model: {}", model_path);

    // 3. Create Pipeline
    let (tx, rx) = channel::bounded::<InferenceTask>(50);

    // 4. Spawn Engine Thread
    let model_path_clone = model_path.to_string(); // Clone string for the thread

    let handle = thread::spawn(move || -> Result<()> {
        // Create Session
        let mut session = Session::builder()?
            .with_execution_providers([CUDAExecutionProvider::default().with_device_id(0).build()])?
            .commit_from_file(&model_path_clone)?;
        
        // FIX 2: Removed `execution_providers()` print as it caused compilation error.
        // Verify GPU usage with `watch -n 1 nvidia-smi` in another terminal.
        println!("Model Session Created (Check nvidia-smi for GPU usage)");

        let mut batch = Vec::with_capacity(18);
        let mut global_results: HashMap<String, Vec<Detection>> = HashMap::new();

        for task in rx {
            batch.push(task);
            if batch.len() >= 18 {
                process_batch(&mut session, &mut batch, &mut global_results)?;
            }
        }
        if !batch.is_empty() {
            process_batch(&mut session, &mut batch, &mut global_results)?;
        }

        // Finalize
        println!("Inference Done. Exporting Results...");
        for (tiff_id, mut detections) in global_results {
            // Global NMS Pass to remove duplicates at tile borders
            modules::post_processing::non_maximum_suppression(&mut detections, 0.45);

            let file_path = format!("{}/{}_manifest.json", output_dir, tiff_id);
            let file = File::create(&file_path)?;
            serde_json::to_writer_pretty(BufWriter::new(file), &detections)?;
            println!("Saved: {}", file_path);
        }
        Ok(())
    });

    // 5. Producer Loop
    let entries = fs::read_dir(input_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "jpg" || e == "jpeg") {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let (off_x, off_y) = calculate_offsets(&filename);

            let task = InferenceTask {
                image_data: fs::read(&path)?,
                global_offset_x: off_x,
                global_offset_y: off_y,
                tile_filename: filename,
            };
            
            if tx.send(task).is_err() {
                break; // Stop if thread died
            }
        }
    }

    drop(tx);
    handle.join().unwrap()?;
    println!("Pipeline Complete.");
    Ok(())
}

fn process_batch(
    session: &mut Session,
    batch: &mut Vec<InferenceTask>,
    aggregator: &mut HashMap<String, Vec<Detection>>,
) -> Result<()> {
    let batch_len = batch.len();
    
    // Preprocess
    let input_data = modules::pre_processing::preprocess_batch(batch)?;
    let input_tensor = Array::from_shape_vec(ndarray::IxDyn(&[batch_len, 3, 896, 896]), input_data)?;

    // Inference
    let outputs = modules::inference::run_inference(session, input_tensor)?;

    // Postprocess
    for (i, single_output_dyn) in outputs.outer_iter().enumerate() {
        let task = &batch[i];
        if let Ok(single_output_2d) = single_output_dyn.into_dimensionality::<ndarray::Ix2>() {
            // No Transpose: The model output is already [300, 6]
            let output_view = single_output_2d;

            let mut detections =
                modules::post_processing::parse_output(output_view, &task.tile_filename);

            for det in &mut detections {
                det.bbox.x_min += task.global_offset_x;
                det.bbox.y_min += task.global_offset_y;
                det.bbox.x_max += task.global_offset_x;
                det.bbox.y_max += task.global_offset_y;
            }

            let tiff_id = extract_tiff_id(&task.tile_filename);
            aggregator.entry(tiff_id).or_default().extend(detections);
        }
    }

    batch.clear();
    Ok(())
}