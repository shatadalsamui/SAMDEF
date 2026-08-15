use crate::modules::data::results::process_single_image;
use crate::modules::data::task::{InferenceTask, PipelineMessage};
use crate::modules::io::publisher::ZenohPublisher;
use crate::modules::io::session::initialize_session;
use crate::modules::io::session::ExecutionProvider;
use crate::modules::processing::batch::process_batch;
use crate::modules::processing::post_processing::Detection;
use anyhow::Result;
use crossbeam::channel;
use ort::session::Session;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;

// Track the state of each file individually
struct FileState {
    detections: Vec<Detection>,
    geo_transform: [f64; 6],
    width: usize,
    height: usize,
    expected_tiles: usize,
    processed_tiles: usize,
    is_eof_received: bool,
}

#[tokio::main]
pub async fn run_consumer(
    task_rx: channel::Receiver<PipelineMessage>,
    model_path: PathBuf,
    output_dir: PathBuf,
    batch_size: usize,
) -> Result<()> {
    //config for cpu and gpu , use as per specs

    //let mut session = initialize_session(&model_path, ExecutionProvider::Cpu)?;
    let mut session = initialize_session(&model_path, ExecutionProvider::Cuda { device_id: 0 })?;

    let mut batch = Vec::with_capacity(batch_size);

    // State Tracker: Key = Canonicalized Filename
    let mut pending_files: HashMap<String, FileState> = HashMap::new();
    let publisher = Arc::new(ZenohPublisher::new().await);
    let mut save_tasks = JoinSet::new();

    // GPU idle timer: total time spent waiting for data
    let mut total_idle_duration = std::time::Duration::default();

    // Main consumer loop: Greedily accumulate tiles into batches of 64.
    loop {
        let start_wait = std::time::Instant::now();
        let msg = match task_rx.recv() {
            Ok(m) => {
                total_idle_duration += start_wait.elapsed();
                m
            }
            Err(_) => break,
        };

        match msg {
            PipelineMessage::Process(task) => {
                batch.push(task);

                // Only process when we hit the FULL batch size.
                // This ensures 100% GPU utilization.
                if batch.len() == batch_size {
                    process_batch_and_update(
                        &mut session,
                        &batch,
                        &mut pending_files,
                        &output_dir,
                        &publisher,
                        &mut save_tasks,
                    )
                    .await?;
                    batch.clear();
                }
            }
            PipelineMessage::EndOfFile {
                source_path,
                geo_transform,
                width,
                height,
                expected_tiles,
            } => {
                // Register that this file expects N tiles.
                // Do NOT flush the batch here; wait for batch size or termination.

                let entry = pending_files
                    .entry(source_path.clone())
                    .or_insert(FileState {
                        detections: Vec::new(),
                        geo_transform,
                        width: width as usize,
                        height: height as usize,
                        expected_tiles: 0,
                        processed_tiles: 0,
                        is_eof_received: false,
                    });

                entry.geo_transform = geo_transform;
                entry.width = width as usize;
                entry.height = height as usize;
                entry.expected_tiles = expected_tiles;
                entry.is_eof_received = true;
            }
            PipelineMessage::Terminate => {
                // Final flush of whatever is left in the batch.
                if !batch.is_empty() {
                    process_batch_and_update(
                        &mut session,
                        &batch,
                        &mut pending_files,
                        &output_dir,
                        &publisher,
                        &mut save_tasks,
                    )
                    .await?;
                }
                // Wait for all background async saves to finish before exiting.
                while let Some(_) = save_tasks.join_next().await {}
                // Print total idle time before exiting
                println!(
                    "Total GPU idle time (waiting for data): {:.2?}",
                    total_idle_duration
                );
                break;
            }
        }

        // After handling each message, check for completed files and spawn async save tasks immediately.
        let mut completed_paths = Vec::new();
        for (path, state) in pending_files.iter() {
            if state.is_eof_received && state.processed_tiles >= state.expected_tiles {
                completed_paths.push(path.clone());
            }
        }
        for path in completed_paths {
            if let Some(mut state) = pending_files.remove(&path) {
                let pub_clone = publisher.clone();
                let out_clone = output_dir.clone();
                let path_clone = path.clone();
                let dets_owned = std::mem::take(&mut state.detections);

                save_tasks.spawn(async move {
                    if let Err(e) = process_single_image(
                        &path_clone,
                        state.geo_transform,
                        dets_owned,
                        &out_clone,
                        &pub_clone,
                        state.width as usize,
                        state.height as usize,
                    )
                    .await
                    {
                        eprintln!("Save error: {}", e);
                    }
                });
            }
        }
    }
    Ok(())
}

/// Process a batch of tiles, update file states, and spawn async saves for completed files.
/// This function is called only when a full batch is ready or at termination.
async fn process_batch_and_update(
    session: &mut Session,
    batch: &[InferenceTask],
    pending_files: &mut HashMap<String, FileState>,
    output_dir: &PathBuf,
    publisher: &Arc<ZenohPublisher>,
    save_tasks: &mut JoinSet<()>,
) -> Result<()> {
    let results = process_batch(session, batch)?;

    // 1. Distribute results to their file states by canonicalized path.
    for (i, dets) in results.into_iter().enumerate() {
        let path = batch[i].source_path.clone();

        let entry = pending_files.entry(path.clone()).or_insert(FileState {
            detections: Vec::new(),
            geo_transform: [0.0; 6],
            width: 0,
            height: 0,
            expected_tiles: usize::MAX, // Unknown until EOF arrives
            processed_tiles: 0,
            is_eof_received: false,
        });

        entry.detections.extend(dets);
        entry.processed_tiles += 1;
    }

    // 2. Check for completed files (all tiles processed and EOF received).
    let mut completed_paths = Vec::new();
    for (path, state) in pending_files.iter() {
        if state.is_eof_received && state.processed_tiles >= state.expected_tiles {
            completed_paths.push(path.clone());
        }
    }

    // 3. Spawn async save tasks for completed files and remove from map.
    for path in completed_paths {
        if let Some(mut state) = pending_files.remove(&path) {
            let pub_clone = publisher.clone();
            let out_clone = output_dir.clone();
            let path_clone = path.clone();
            let dets_owned = std::mem::take(&mut state.detections);

            // Spawn Async Save using JoinSet
            save_tasks.spawn(async move {
                if let Err(e) = process_single_image(
                    &path_clone,
                    state.geo_transform,
                    dets_owned,
                    &out_clone,
                    &pub_clone,
                    state.width as usize,
                    state.height as usize,
                )
                .await
                {
                    eprintln!("Save error: {}", e);
                }
            });
        }
    }

    Ok(())
}
