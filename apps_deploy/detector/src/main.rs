use anyhow::Result;
use crossbeam::channel;
use dotenv::dotenv;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;

mod modules;
use modules::data::task::PipelineMessage;
use modules::io::consumer::run_consumer;
use modules::io::producer::run_producer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let _ = ort::init().with_name("SAMDEF_Detector").commit();

    let input_dir = PathBuf::from(env::var("INPUT_DIR").expect("INPUT_DIR must be set in .env"));
    let output_dir = PathBuf::from(env::var("OUTPUT_DIR").expect("OUTPUT_DIR must be set in .env"));
    let model_path = PathBuf::from(env::var("MODEL_PATH").expect("MODEL_PATH must be set in .env"));
    let batch_size: usize = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .expect("BATCH_SIZE must be a valid number in .env");
    let producer_parallelism: usize = env::var("PRODUCER_PARALLELISM")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .expect("PRODUCER_PARALLELISM must be a valid number in .env");

    fs::create_dir_all(&output_dir)?;

    println!("SAMDEF Detector Starting...");
    println!("Input: {:?}", input_dir);
    println!("Model: {:?}", model_path);

    // Total pipeline timer
    let start_time = std::time::Instant::now();

    // Use PipelineMessage channel instead of InferenceTask
    let (msg_tx, msg_rx) = channel::bounded::<PipelineMessage>(batch_size * 2);

    // Pass output_dir to consumer for per-file saving/publishing
    let consumer_handle = thread::spawn(move || run_consumer(msg_rx, model_path, output_dir, batch_size));
    let producer_handle = thread::spawn(move || run_producer(input_dir, msg_tx, producer_parallelism));

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
