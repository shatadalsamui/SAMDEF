pub mod inference_manifest {
    include!(concat!(env!("OUT_DIR"), "/inference_manifest.rs"));
}
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::fs;
use std::sync::{ Arc, Mutex };
use std::time::Instant;
use indicatif::ProgressBar;
use gdal::Dataset;
mod modules;
use modules::tiler::process_inference_image;

fn main() {
    ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    let start = Instant::now();
    let input_dir = "/home/shatadal/SAMDEF_DATA/val_images";
    let tiles_output_dir = "/home/shatadal/SAMDEF/raw_data/inference/inference_tiles";
    let manifest_output_dir = "/home/shatadal/SAMDEF/raw_data/inference/inference_manifests";
    let tile_size = 896;
    let stride = 716;
    let jpeg_quality = 95;

    let image_paths: Vec<_> = fs
        ::read_dir(input_dir)
        .expect("Failed to read input directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().map_or(false, |ext| ext == "tif") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Calculate total tiles across all images
    let total_tiles: u64 = image_paths
        .iter()
        .map(|img_path| {
            let dataset = Dataset::open(img_path).expect("Failed to open dataset for tile count");
            let (width_usize, height_usize) = dataset.raster_size();
            let width = width_usize as u32;
            let height = height_usize as u32;
            let n_tiles_x = ((width - 1) / stride + 1) as u64;
            let n_tiles_y = ((height - 1) / stride + 1) as u64;
            n_tiles_x * n_tiles_y
        })
        .sum();

    // Create a single shared progress bar
    let pb = Arc::new(Mutex::new(ProgressBar::new(total_tiles)));

    // Commented out: Scope/channel approach (causing hangs)
    /*
    // Create channel for the buffer
    let (tx, rx): (crossbeam::channel::Sender<std::path::PathBuf>, crossbeam::channel::Receiver<std::path::PathBuf>) = channel::unbounded();

    // Use rayon::scope for stable worker pool
    rayon::scope(|s| {
        // Spawn 16 worker tasks
        for _ in 0..16 {
            let rx = rx.clone();
            let pb = Arc::clone(&pb);
            let tiles_output_dir = tiles_output_dir.to_string();
            let manifest_output_dir = manifest_output_dir.to_string();
            s.spawn(move |_| {
                while let Ok(img_path) = rx.recv() {
                    let _ = process_inference_image(
                        &img_path,
                        &tiles_output_dir,
                        &manifest_output_dir,
                        tile_size,
                        stride,
                        jpeg_quality,
                        Arc::clone(&pb),
                    );
                }
            });
        }

        // Send all image paths to the channel
        for img_path in image_paths {
            tx.send(img_path).unwrap();
        }
    });

    // Close the channel and finish
    drop(tx);
    */

    image_paths.par_iter().for_each(|img_path| {
        let pb_clone = Arc::clone(&pb);
        let _ = process_inference_image(
            img_path,
            tiles_output_dir,
            manifest_output_dir,
            tile_size,
            stride,
            jpeg_quality,
            pb_clone
        );
    });

    // Finish the progress bar after all processing
    pb.lock().unwrap().finish_with_message("All images processed");
    let elapsed = start.elapsed();
    println!("Total processing time: {:.2}s", elapsed.as_secs_f64());
}
