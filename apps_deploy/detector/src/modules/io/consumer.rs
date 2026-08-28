use crate::modules::data::results::process_single_image;
use crate::modules::data::task::{InferenceTask, PipelineMessage};
use crate::modules::io::publisher::ZenohPublisher;
use crate::modules::io::session::{initialize_session, ExecutionProvider};
use crate::modules::processing::inference::run_inference;
use crate::modules::processing::post_processing::{parse_output, Detection};
use crate::modules::processing::pre_processing::preprocess_batch;
use anyhow::Result;
use crossbeam::channel;
use half::f16;
use ndarray::Array4;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
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

enum PostMessage {
    InferenceResult {
        outputs: ndarray::ArrayD<f16>,
        tasks: Vec<InferenceTask>,
    },
    EndOfFile {
        source_path: String,
        geo_transform: [f64; 6],
        width: usize,
        height: usize,
        expected_tiles: usize,
    },
    Terminate,
}

#[tokio::main]
pub async fn run_consumer(
    task_rx: channel::Receiver<PipelineMessage>,
    model_path: PathBuf,
    output_dir: PathBuf,
    batch_size: usize,
) -> Result<()> {
    // Channels connecting the 3 stages:
    // Stage 1 -> Stage 2 (GPU)
    let (gpu_tx, gpu_rx) = channel::bounded::<Option<(Array4<f16>, Vec<InferenceTask>)>>(4);
    // Stage 2 & Stage 1 -> Stage 3 (Postprocess & Save)
    let (post_tx, post_rx) = channel::bounded::<PostMessage>(64);

    let post_tx_eof = post_tx.clone();

    // 1. Preprocess and batch tiles on CPU (dedicated OS thread)
    let preprocessor_handle = thread::spawn(move || -> Result<()> {
        let mut batch: Vec<InferenceTask> = Vec::with_capacity(batch_size);

        while let Ok(msg) = task_rx.recv() {
            match msg {
                PipelineMessage::Process(task) => {
                    batch.push(task);

                    if batch.len() == batch_size {
                        let input_data = preprocess_batch(&batch)?;
                        let input_tensor =
                            Array4::<f16>::from_shape_vec((batch.len(), 3, 896, 896), input_data)?;
                        let _ = gpu_tx.send(Some((input_tensor, std::mem::take(&mut batch))));
                    }
                }
                PipelineMessage::EndOfFile {
                    source_path,
                    geo_transform,
                    width,
                    height,
                    expected_tiles,
                } => {
                    let _ = post_tx_eof.send(PostMessage::EndOfFile {
                        source_path,
                        geo_transform,
                        width: width as usize,
                        height: height as usize,
                        expected_tiles,
                    });
                }
                PipelineMessage::Terminate => {
                    // Flush any remaining tiles
                    if !batch.is_empty() {
                        let input_data = preprocess_batch(&batch)?;
                        let input_tensor =
                            Array4::<f16>::from_shape_vec((batch.len(), 3, 896, 896), input_data)?;
                        let _ = gpu_tx.send(Some((input_tensor, std::mem::take(&mut batch))));
                    }
                    let _ = gpu_tx.send(None);
                    break;
                }
            }
        }
        Ok(())
    });

    // 2. Run inference on GPU (dedicated OS thread)
    let gpu_handle = thread::spawn(move || -> Result<()> {
        let mut session =
            initialize_session(&model_path, ExecutionProvider::Cuda { device_id: 0 })?;

        while let Ok(Some((input_tensor, tasks))) = gpu_rx.recv() {
            let outputs = run_inference(&mut session, input_tensor)?;
            let _ = post_tx.send(PostMessage::InferenceResult { outputs, tasks });
        }
        let _ = post_tx.send(PostMessage::Terminate);
        Ok(())
    });

    // 3. Postprocess detections and save files (Tokio async loop)
    let mut pending_files: HashMap<String, FileState> = HashMap::new();
    let publisher = Arc::new(ZenohPublisher::new().await);
    let mut save_tasks = JoinSet::new();

    while let Ok(msg) = post_rx.recv() {
        match msg {
            PostMessage::InferenceResult { outputs, tasks } => {
                for (i, single_output_dyn) in outputs.outer_iter().enumerate() {
                    let task = &tasks[i];
                    if let Ok(single_output_2d) =
                        single_output_dyn.into_dimensionality::<ndarray::Ix2>()
                    {
                        let mut detections = parse_output(single_output_2d);

                        for det in &mut detections {
                            det.bbox.x_min =
                                (det.bbox.x_min as f64 + task.global_offset_x as f64) as f32;
                            det.bbox.y_min =
                                (det.bbox.y_min as f64 + task.global_offset_y as f64) as f32;
                            det.bbox.x_max =
                                (det.bbox.x_max as f64 + task.global_offset_x as f64) as f32;
                            det.bbox.y_max =
                                (det.bbox.y_max as f64 + task.global_offset_y as f64) as f32;
                        }

                        let entry = pending_files
                            .entry(task.source_path.clone())
                            .or_insert(FileState {
                                detections: Vec::new(),
                                geo_transform: [0.0; 6],
                                width: 0,
                                height: 0,
                                expected_tiles: usize::MAX,
                                processed_tiles: 0,
                                is_eof_received: false,
                            });

                        entry.detections.extend(detections);
                        entry.processed_tiles += 1;
                    }
                }
            }
            PostMessage::EndOfFile {
                source_path,
                geo_transform,
                width,
                height,
                expected_tiles,
            } => {
                let entry = pending_files.entry(source_path).or_insert(FileState {
                    detections: Vec::new(),
                    geo_transform,
                    width,
                    height,
                    expected_tiles: 0,
                    processed_tiles: 0,
                    is_eof_received: false,
                });

                entry.geo_transform = geo_transform;
                entry.width = width;
                entry.height = height;
                entry.expected_tiles = expected_tiles;
                entry.is_eof_received = true;
            }
            PostMessage::Terminate => {
                break;
            }
        }

        // Check for completed files and spawn async save tasks immediately
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
                        state.width,
                        state.height,
                    )
                    .await
                    {
                        eprintln!("Save error: {}", e);
                    }
                });
            }
        }
    }

    // Wait for all background async saves to finish before exiting
    while let Some(_) = save_tasks.join_next().await {}

    let _ = preprocessor_handle.join();
    let _ = gpu_handle.join();

    Ok(())
}
