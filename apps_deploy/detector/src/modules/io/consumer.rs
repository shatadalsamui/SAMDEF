use anyhow::Result;
use std::collections::HashMap;
use crate::modules::processing::batch::process_batch;
use crate::modules::processing::post_processing::Detection;
use crate::modules::data::task::InferenceTask;
use crate::modules::io::session::initialize_session;
use ort::session::Session;
use crossbeam::channel;

const BATCH_SIZE: usize = 32;

pub fn run_consumer(
    task_rx: channel::Receiver<InferenceTask>,
    model_path: std::path::PathBuf,
) -> Result<HashMap<String, (Vec<Detection>, [f64; 6])>> {
    let mut session = initialize_session(&model_path)?;
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut all_results: HashMap<String, (Vec<Detection>, [f64; 6])> = HashMap::new();

    while let Ok(task) = task_rx.recv() {
        batch.push(task);
        if batch.len() >= BATCH_SIZE {
            process_batch_and_store(&mut session, &batch, &mut all_results)?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        process_batch_and_store(&mut session, &batch, &mut all_results)?;
    }

    Ok(all_results)
}

pub fn process_batch_and_store(
    session: &mut Session,
    batch: &[InferenceTask],
    all_results: &mut HashMap<String, (Vec<Detection>, [f64; 6])>,
) -> Result<()> {
    let batch_results = process_batch(session, batch)?;
    for (i, detections) in batch_results.into_iter().enumerate() {
        let task = &batch[i];
        all_results
            .entry(task.source_path.clone())
            .or_insert_with(|| (Vec::new(), task.geo_transform))
            .0
            .extend(detections);
    }
    Ok(())
}