use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use crossbeam::channel;
use crate::modules::data::task::InferenceTask;
use crate::modules::io::virtual_tiler;

pub fn run_producer(input_dir: PathBuf, task_tx: channel::Sender<InferenceTask>) -> Result<()> {
    let entries = fs::read_dir(input_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "tif" || e == "tiff") {
            if let Err(e) = virtual_tiler::process_geotiff(&path, task_tx.clone()) {
                eprintln!("Error processing GeoTIFF {:?}: {}", path, e);
            }
        }
    }
    Ok(())
}