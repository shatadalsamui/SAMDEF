use crate::modules::data::buffer_pool::{create_pinned_tensor_ring, PinAcquirer, TileBytePool};
use crate::modules::data::results::save_resolved_scene;
use crate::modules::data::task::{InferenceTask, PipelineMessage};
use crate::modules::io::aggregator::{spawn_aggregator_worker, NmsWorkItem, PostMessage};
use crate::modules::io::nms_worker::{spawn_nms_worker, ResolvedScene};
use crate::modules::io::publisher::ZenohPublisher;
use crate::modules::io::session::{initialize_session, ExecutionProvider};
use crate::modules::io::virtual_tiler::TILE_SIZE;
use crate::modules::processing::inference::GpuInferenceEngine;
use crate::modules::processing::pre_processing::{create_preproc_pool, fill_batch_tensor};
use anyhow::Result;
use crossbeam::channel;
use ort::value::Tensor;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

struct GpuBatch {
    tensor: Tensor<f32>,
    tasks: Vec<InferenceTask>,
    valid_len: usize,
}

fn dispatch_batch_to_gpu(
    batch: &mut Vec<InferenceTask>,
    preproc_pool: &rayon::ThreadPool,
    byte_pool: &TileBytePool,
    pin_acquirer: &PinAcquirer,
    gpu_tx: &channel::Sender<Option<GpuBatch>>,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let valid_len = batch.len();
    let mut tensor = pin_acquirer.acquire()?;
    {
        let (_, dst) = tensor.try_extract_tensor_mut::<f32>()?;
        fill_batch_tensor(preproc_pool, byte_pool, batch, dst)?;
    }
    gpu_tx
        .send(Some(GpuBatch {
            tensor,
            tasks: std::mem::take(batch),
            valid_len,
        }))
        .map_err(|_| anyhow::anyhow!("Stage 2 GPU queue closed"))?;
    Ok(())
}

#[tokio::main]
pub async fn run_consumer(
    task_rx: channel::Receiver<PipelineMessage>,
    model_path: PathBuf,
    output_dir: PathBuf,
    batch_size: usize,
    tile_byte_pool: TileBytePool,
) -> Result<()> {
    const GPU_QUEUE_CAP: usize = 6;

    // Channels connecting the 5 isolated pipeline stages:
    // Stage 1 (CPU Preprocessor) -> Stage 2 (Pure GPU CUDA Runner)
    let (gpu_tx, gpu_rx) = channel::bounded::<Option<GpuBatch>>(GPU_QUEUE_CAP);
    // One-shot handoff so GPU thread creates Session + CUDA_PINNED ring cleanly on its own thread
    let (pin_acq_tx, pin_acq_rx) = channel::bounded::<PinAcquirer>(1);
    // Stage 2 & Stage 1 -> Stage 3 (Dedicated Fast Tile Aggregator)
    let (post_tx, post_rx) = channel::bounded::<PostMessage>(128);
    // Stage 3 -> Stage 4 (Dedicated Boundary-Aware NMS OS Thread)
    let (nms_tx, nms_rx) = channel::bounded::<NmsWorkItem>(64);
    // Stage 4 -> Stage 5 (Tokio Async Disk Write & Zenoh Publisher)
    let (save_tx, mut save_rx) = mpsc::channel::<ResolvedScene>(64);

    let post_tx_eof = post_tx.clone();

    // Stage 2: Pure GPU inference on dedicated OS thread (CUDA Graph + IoBinding)
    let gpu_handle = thread::spawn(move || -> Result<()> {
        const DEVICE_ID: i32 = 0;
        let session = initialize_session(&model_path, ExecutionProvider::Cuda { device_id: DEVICE_ID })?;
        let (pin_acquirer, pin_recycler) = create_pinned_tensor_ring(
            &session,
            Some(DEVICE_ID),
            batch_size,
            GPU_QUEUE_CAP + 2,
            TILE_SIZE,
        )?;
        let _ = pin_acq_tx.send(pin_acquirer);

        let mut engine = GpuInferenceEngine::new(session, Some(DEVICE_ID), batch_size)?;

        while let Ok(Some(gpu_batch)) = gpu_rx.recv() {
            let outputs = match engine.run_batch(&gpu_batch.tensor, gpu_batch.valid_len) {
                Ok(out) => out,
                Err(e) => {
                    eprintln!("Stage 2 GPU inference error: {:?}", e);
                    return Err(e);
                }
            };
            // Immediately recycle pinned tensor so Stage 2 never drops/unmaps large host buffers
            pin_recycler.recycle(gpu_batch.tensor);
            let _ = post_tx.send(PostMessage::InferenceResult {
                outputs,
                tasks: gpu_batch.tasks,
            });
        }
        let _ = post_tx.send(PostMessage::Terminate);
        Ok(())
    });

    // Stage 1: Linear LUT preprocessing into warm CUDA_PINNED buffers on isolated pool
    let preprocessor_handle = thread::spawn(move || -> Result<()> {
        let res = (|| -> Result<()> {
            let pin_acquirer = pin_acq_rx
                .recv()
                .map_err(|_| anyhow::anyhow!("GPU thread exited before sending PinAcquirer"))?;
            let preproc_pool = create_preproc_pool();
            let mut batch: Vec<InferenceTask> = Vec::with_capacity(batch_size);

            while let Ok(msg) = task_rx.recv() {
                match msg {
                    PipelineMessage::Process(task) => {
                        batch.push(task);
                        if batch.len() == batch_size {
                            dispatch_batch_to_gpu(
                                &mut batch,
                                &preproc_pool,
                                &tile_byte_pool,
                                &pin_acquirer,
                                &gpu_tx,
                            )?;
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
                        dispatch_batch_to_gpu(
                            &mut batch,
                            &preproc_pool,
                            &tile_byte_pool,
                            &pin_acquirer,
                            &gpu_tx,
                        )?;
                        let _ = gpu_tx.send(None);
                        break;
                    }
                }
            }
            Ok(())
        })();
        if let Err(ref e) = res {
            eprintln!("Stage 1 preprocessor error: {:?}", e);
        }
        res
    });

    // Stage 3 & Stage 4: Isolated Aggregator and Boundary-Aware NMS OS threads
    let aggregator_handle = spawn_aggregator_worker(post_rx, nms_tx);
    let nms_handle = spawn_nms_worker(nms_rx, save_tx);

    // Stage 5: Non-blocking Tokio sink for JSON serialization + Zenoh publishing
    let publisher = Arc::new(ZenohPublisher::new().await);
    let mut save_tasks = JoinSet::new();

    while let Some(scene) = save_rx.recv().await {
        let pub_clone = publisher.clone();
        let out_clone = output_dir.clone();

        save_tasks.spawn(async move {
            if let Err(e) = save_resolved_scene(
                &scene.source_path,
                scene.geo_transform,
                scene.detections,
                &out_clone,
                &pub_clone,
                scene.width,
                scene.height,
            )
            .await
            {
                eprintln!("Save error: {}", e);
            }
        });
    }

    while let Some(_) = save_tasks.join_next().await {}

    if let Ok(Err(e)) = preprocessor_handle.join() {
        return Err(e);
    }
    if let Ok(Err(e)) = gpu_handle.join() {
        return Err(e);
    }
    let _ = aggregator_handle.join();
    let _ = nms_handle.join();

    Ok(())
}
