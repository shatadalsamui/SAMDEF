use anyhow::Result;
use ort::{ep::CUDAExecutionProvider, session::Session};
use std::path::PathBuf;

pub fn initialize_session(model_path: &PathBuf) -> Result<Session> {
    let session = Session::builder()
        .and_then(|builder| {
            builder.with_execution_providers([CUDAExecutionProvider::default()
                .with_device_id(0)
                .build()])
        })
        .and_then(|builder| builder.commit_from_file(model_path));

    match session {
        Ok(session) => {
            println!("Model Session Created (Check nvidia-smi for GPU usage)");
            Ok(session)
        }
        Err(e) => {
            eprintln!("Failed to create ONNX session: {:?}", e);
            Err(anyhow::Error::from(e))
        }
    }
}
