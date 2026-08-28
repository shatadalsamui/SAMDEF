use crate::modules::data::task::PipelineMessage;
use crate::modules::io::virtual_tiler;
use anyhow::Result;
use crossbeam::channel;
use std::fs;
use std::path::PathBuf;
use std::thread; // Use OS threads instead of Rayon

pub fn run_producer(
    input_dir: PathBuf,
    task_tx: channel::Sender<PipelineMessage>,
    producer_parallelism: usize,
) -> Result<()> {
    // 1. Collect and Sort Paths
    let mut entries: Vec<_> = fs::read_dir(input_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "tif" || e == "tiff"))
        .collect();

    entries.sort();

    // 2. Create a work queue for paths and fill it
    let (path_tx, path_rx) = channel::unbounded::<PathBuf>();
    for path in entries {
        let _ = path_tx.send(path);
    }
    drop(path_tx); // Close the sending side so workers exit when empty

    // 3. Spawn a fixed pool of persistent worker threads using OS Threads
    // This prevents "Rayon Starvation"
    let mut worker_handles = Vec::with_capacity(producer_parallelism);
    for _ in 0..producer_parallelism {
        let rx = path_rx.clone();
        let tx = task_tx.clone();

        let handle = thread::spawn(move || {
            // Workers continuously pull next available image with zero barrier wait
            while let Ok(path) = rx.recv() {
                let canonical_path = match path.canonicalize() {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(e) => {
                        eprintln!("Path error: {}", e);
                        continue;
                    }
                };

                if let Err(e) = virtual_tiler::process_geotiff(&canonical_path, tx.clone()) {
                    eprintln!("Error processing {:?}: {}", path, e);
                }
            }
        });
        worker_handles.push(handle);
    }

    // 4. Wait for all workers to finish the whole dataset
    for h in worker_handles {
        let _ = h.join();
    }

    // 5. Send final termination signal to consumer
    let _ = task_tx.send(PipelineMessage::Terminate);
    Ok(())
}
