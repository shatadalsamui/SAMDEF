use anyhow::Result;
use crossbeam::channel;
use ndarray::Array;
use ort::{ep::CUDAExecutionProvider, session::Session}; 
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::thread;
mod modules;
use modules::post_processing::Detection;
use modules::task::InferenceTask;
use modules::utils::calculate_offsets;
use modules::batch::process_batch;

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
    let model_path = "/home/shatadal/SAMDEF/apps_deploy/detector/model/best_1000.onnx";

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

        let mut batch = Vec::with_capacity(32);
        let mut global_results: HashMap<String, Vec<Detection>> = HashMap::new();

        for task in rx {
            batch.push(task);
            if batch.len() >= 32 {
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