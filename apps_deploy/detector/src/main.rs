use anyhow::Result;
use crossbeam::channel;
use std::fs;
use std::path::PathBuf;
use std::thread;

mod modules;
use modules::data::task::InferenceTask;
use modules::io::consumer::run_consumer;
use modules::io::producer::run_producer;
use modules::data::results::process_and_save_results;

const BATCH_SIZE: usize = 32;

fn main() -> Result<()> {
    env_logger::init();
    let _ = ort::init().with_name("SAMDEF_Detector").commit();

    let input_dir = PathBuf::from("/home/shatadal/SAMDEF_DATA/val_images");
    let output_dir = PathBuf::from("/home/shatadal/SAMDEF/raw_data/inference/results");
    let model_path = PathBuf::from("/home/shatadal/SAMDEF/apps_deploy/detector/model/best.onnx");

    fs::create_dir_all(&output_dir)?;

    println!("SAMDEF Detector Starting...");
    println!("Input: {:?}", input_dir);
    println!("Model: {:?}", model_path);

    let (task_tx, task_rx) = channel::bounded::<InferenceTask>(BATCH_SIZE * 2);

    let consumer_handle = thread::spawn(move || run_consumer(task_rx, model_path));
    let producer_handle = thread::spawn(move || run_producer(input_dir, task_tx));

    let _ = producer_handle.join().map_err(|e| anyhow::anyhow!("Producer thread panicked: {:?}", e))?;
    let results_by_path = consumer_handle.join().map_err(|e| anyhow::anyhow!("Consumer thread panicked: {:?}", e))??;

    process_and_save_results(results_by_path, &output_dir)?;

    println!("Pipeline Complete.");
    Ok(())
}
