use crate::modules::data::task::PipelineMessage;
use crate::modules::io::virtual_tiler;
use anyhow::Result;
use crossbeam::channel;
use std::fs;
use std::path::PathBuf;
use std::thread; // Use OS threads instead of Rayon

const PRODUCER_PARALLELISM: usize = 4; // Read 4 TIFFs at once

pub fn run_producer(input_dir: PathBuf, task_tx: channel::Sender<PipelineMessage>) -> Result<()> {
    // 1. Collect and Sort Paths
    let mut entries: Vec<_> = fs::read_dir(input_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "tif" || e == "tiff"))
        .collect();

    entries.sort();

    // 2. Process in Chunks using OS Threads
    // This prevents "Rayon Starvation"
    // Process in chunks of 4 using OS threads
    for chunk in entries.chunks(PRODUCER_PARALLELISM) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let tx = task_tx.clone();

                thread::spawn(move || {
                    let canonical_path = match path.canonicalize() {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(e) => {
                            eprintln!("Path error: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = virtual_tiler::process_geotiff(&canonical_path, tx) {
                        eprintln!("Error processing {:?}: {}", path, e);
                    }
                })
            })
            .collect();

        // Wait for these 4 files to finish before starting the next 4
        for h in handles {
            let _ = h.join();
        }
    }

    let _ = task_tx.send(PipelineMessage::Terminate);
    Ok(())
}
