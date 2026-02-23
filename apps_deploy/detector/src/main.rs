use anyhow::Result;
use crossbeam::channel;
use std::fs;
use std::path::PathBuf;
use std::thread;

mod modules;
use modules::data::task::PipelineMessage;
use modules::io::consumer::run_consumer;
use modules::io::producer::run_producer;

const BATCH_SIZE: usize = 32;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let _ = ort::init().with_name("SAMDEF_Detector").commit();

    let input_dir = PathBuf::from("/home/shatadal/SAMDEF_DATA/val_images");
    let output_dir = PathBuf::from("/home/shatadal/SAMDEF/raw_data/inference/results");
    let model_path = PathBuf::from("/home/shatadal/SAMDEF/apps_deploy/detector/model/best_fp16_patched.onnx");

    fs::create_dir_all(&output_dir)?;

    println!("SAMDEF Detector Starting...");
    println!("Input: {:?}", input_dir);
    println!("Model: {:?}", model_path);

    // Total pipeline timer
    let start_time = std::time::Instant::now();

    // Use PipelineMessage channel instead of InferenceTask
    let (msg_tx, msg_rx) = channel::bounded::<PipelineMessage>(BATCH_SIZE * 2);

    // Pass output_dir to consumer for per-file saving/publishing
    let consumer_handle = thread::spawn(move || run_consumer(msg_rx, model_path, output_dir));
    let producer_handle = thread::spawn(move || run_producer(input_dir, msg_tx));

    let _ = producer_handle
        .join()
        .map_err(|e| anyhow::anyhow!("Producer thread panicked: {:?}", e))?;
    let _ = consumer_handle
        .join()
        .map_err(|e| anyhow::anyhow!("Consumer thread panicked: {:?}", e))?;

    let total_duration = start_time.elapsed();
    println!("Pipeline Complete.");
    println!("Total pipeline execution time: {:.2?}", total_duration);
    Ok(())
}
