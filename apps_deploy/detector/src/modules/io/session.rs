use anyhow::Result;
use ort::{ep::CUDAExecutionProvider, session::Session};
use std::path::PathBuf;

pub fn initialize_session(model_path: &PathBuf) -> Result<Session> {
    let session = Session::builder()?
        .with_execution_providers([CUDAExecutionProvider::default().with_device_id(0).build()])?
        .commit_from_file(model_path)?;
    println!("Model Session Created (Check nvidia-smi for GPU usage)");
    Ok(session)
}