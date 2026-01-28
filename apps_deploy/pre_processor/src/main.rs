pub mod inference_manifest {
    include!(concat!(env!("OUT_DIR"), "/inference_manifest.rs"));
}
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::fs;
use std::sync::{ Arc, Mutex };
use std::time::Instant;
use std::path::PathBuf;
use std::collections::HashSet;
use indicatif::{ProgressBar, ProgressStyle};
use gdal::Dataset;
use log::error;
use crossbeam::channel::unbounded;
use threadpool::ThreadPool;
mod modules;
use modules::watcher::start_watch_loop;
use modules::tiler::process_inference_image;

fn main() {
    env_logger::init();
    ThreadPoolBuilder::new().num_threads(32).build_global().unwrap();

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

    // Initialize processed set for continuous mode
    let mut processed: HashSet<PathBuf> = HashSet::new();

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

    image_paths.par_iter().for_each(|img_path| {
        let pb_clone = Arc::clone(&pb);
        if let Err(e) = process_inference_image(
            img_path,
            tiles_output_dir,
            manifest_output_dir,
            tile_size,
            stride,
            jpeg_quality,
            pb_clone
        ) {
            error!("Failed to process image {}: {}", img_path.display(), e);
        }
    });

    // Finish the progress bar after all processing
    pb.lock().unwrap().finish_with_message("All images processed");
    let elapsed = start.elapsed();
    println!("Total processing time: {:.2}s", elapsed.as_secs_f64());

    // Populate processed set with batch files to avoid re-processing
    for path in &image_paths {
        if let Ok(canon) = fs::canonicalize(path) {
            processed.insert(canon);
        }
    }

    // Now start continuous watching for new files
    println!(" Switching to continuous mode. Monitoring for new TIFFs...");

    let pool = ThreadPool::new(16);
    let (tx, rx) = crossbeam::channel::unbounded::<PathBuf>();

    // Spawn the watcher
    std::thread::spawn(move || {
        if let Err(e) = modules::watcher::start_watch_loop(input_dir, tx) {
            eprintln!("❌ Watcher critical failure: {}", e);
        }
    });

    // Continuous loop for new files
    for path in rx {
        let canon = match fs::canonicalize(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if processed.contains(&canon) {
            continue;
        }
        processed.insert(canon);
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        println!("New file detected: {:?}", file_name);

        let tiles_out = tiles_output_dir.to_string();
        let manifest_out = manifest_output_dir.to_string();
        let tile_sz = tile_size;
        let strd = stride;
        let qual = jpeg_quality;

        pool.execute(move || {
            let pb = Arc::new(Mutex::new(ProgressBar::new(0)));
            // Hide progress bar for continuous mode to avoid overlap
            pb.lock().unwrap().set_style(ProgressStyle::default_bar().template("").unwrap());

            if let Err(e) = process_inference_image(
                &path,
                &tiles_out,
                &manifest_out,
                tile_sz,
                strd,
                qual,
                pb
            ) {
                error!("❌ Processing Failed for {:?}: {}", path, e);
            } else {
                println!("✅ Tiling Complete: {:?}", path.file_name().unwrap());
            }
        });
    }
}
