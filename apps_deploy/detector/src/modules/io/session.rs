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
    #[allow(dead_code)]
    Cpu,
    Cuda { device_id: i32 },
}

pub fn initialize_session(model_path: &Path, provider: ExecutionProvider) -> Result<Session> {
    let builder_res = Session::builder().and_then(|builder| {
        let builder = builder
            .with_intra_threads(2)?
            .with_inter_threads(1)?
            .with_intra_op_spinning(false)?
            .with_inter_op_spinning(false)?;

        match provider {
            ExecutionProvider::Cpu => Ok(builder),
            ExecutionProvider::Cuda { device_id } => Ok(builder.with_execution_providers([CUDA::default()
                .with_device_id(device_id)
                .with_tf32(true)
                .with_conv_algorithm_search(ConvAlgorithmSearch::Exhaustive)
                .with_arena_extend_strategy(ArenaExtendStrategy::SameAsRequested)
                .with_cuda_graph(true)
                .build()])?),
        }
    });

    let session = builder_res
        .map_err(|e| anyhow::anyhow!("ONNX session builder error: {}", e))?
        .commit_from_file(model_path);

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
