use anyhow::Result;
use ort::{
    ep::{
        cuda::{ConvAlgorithmSearch, CUDA},
        ArenaExtendStrategy,
    },
    session::Session,
};
use std::path::Path;

pub enum ExecutionProvider {
    Cpu,
    Cuda { device_id: i32 },
}

pub fn initialize_session(model_path: &Path, provider: ExecutionProvider) -> Result<Session> {
    let intra_threads = match provider {
        ExecutionProvider::Cpu => std::thread::available_parallelism()
            .map(|n| (n.get() / 2).clamp(2, 8))
            .unwrap_or(4),
        ExecutionProvider::Cuda { .. } => 2,
    };

    let builder_res = Session::builder().and_then(|builder| {
        let builder = builder
            .with_intra_threads(intra_threads)?
            .with_inter_threads(1)?
            .with_intra_op_spinning(false)?
            .with_inter_op_spinning(false)?;

        match provider {
            ExecutionProvider::Cpu => Ok(builder),
            ExecutionProvider::Cuda { device_id } => Ok(builder.with_execution_providers([
                CUDA::default()
                    .with_device_id(device_id)
                    .with_tf32(true)
                    .with_conv_algorithm_search(ConvAlgorithmSearch::Exhaustive)
                    .with_arena_extend_strategy(ArenaExtendStrategy::SameAsRequested)
                    .with_cuda_graph(true)
                    .build()
                    .error_on_failure(),
            ])?),
        }
    });

    let session = builder_res
        .map_err(|e| anyhow::anyhow!("ONNX session builder error: {}", e))?
        .commit_from_file(model_path);

    match session {
        Ok(session) => {
            match provider {
                ExecutionProvider::Cuda { device_id } => {
                    println!("Model Session Created on CUDA:{} (Check nvidia-smi for GPU usage)", device_id);
                }
                ExecutionProvider::Cpu => {
                    println!(
                        "Model Session Created on CPU fallback ({} intra-op threads)",
                        intra_threads
                    );
                }
            }
            Ok(session)
        }
        Err(e) => {
            let provider_name = match provider {
                ExecutionProvider::Cuda { device_id } => format!("CUDA:{}", device_id),
                ExecutionProvider::Cpu => "CPU".to_string(),
            };
            eprintln!("Failed to create ONNX session ({}): {:?}", provider_name, e);
            Err(anyhow::Error::from(e))
        }
    }
}
