use anyhow::Result;
use ort::{ep::CUDAExecutionProvider, session::Session};
use std::path::PathBuf;

pub enum ExecutionProvider {
    Cpu,
    Cuda { device_id: i32 },
}

pub fn initialize_session(model_path: &PathBuf, provider: ExecutionProvider) -> Result<Session> {
    let builder = Session::builder();
    let session = builder.and_then(|builder| {
        match provider {
            ExecutionProvider::Cpu => Ok(builder), // Default is CPU
            ExecutionProvider::Cuda { device_id } => builder.with_execution_providers([
                CUDAExecutionProvider::default().with_device_id(device_id as i32).build()
            ]),
        }
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

