use crate::modules::io::aggregator::NmsWorkItem;
use crate::modules::processing::post_processing::{
    create_nms_pool, resolve_tiled_detections_in_pool, Detection,
};
use anyhow::Result;
use crossbeam::channel;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

pub struct ResolvedScene {
    pub source_path: String,
    pub geo_transform: [f64; 6],
    pub width: usize,
    pub height: usize,
    pub detections: Vec<Detection>,
}

/// Stage 4: Dedicated OS worker running Boundary-Aware Seam Merge & Hybrid NMS
/// inside an isolated Rayon pool so Stage 1 preprocessing and Stage 2 GPU never block.
pub fn spawn_nms_worker(
    nms_rx: channel::Receiver<NmsWorkItem>,
    save_tx: mpsc::Sender<ResolvedScene>,
) -> JoinHandle<Result<()>> {
    thread::spawn(move || -> Result<()> {
        let nms_pool = create_nms_pool();

        while let Ok(work) = nms_rx.recv() {
            let detections = resolve_tiled_detections_in_pool(
                &nms_pool,
                work.raw_detections,
                work.width,
                work.height,
                0.45,
            );
            let resolved = ResolvedScene {
                source_path: work.source_path,
                geo_transform: work.geo_transform,
                width: work.width,
                height: work.height,
                detections,
            };
            if save_tx.blocking_send(resolved).is_err() {
                break;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::data::task::InferenceTask;
    use crate::modules::io::aggregator::{spawn_aggregator_worker, PostMessage};
    use ndarray::Array3;

    #[test]
    fn test_decoupled_aggregator_and_nms_worker_pipeline() {
        let (post_tx, post_rx) = channel::bounded::<PostMessage>(16);
        let (nms_tx, nms_rx) = channel::bounded::<NmsWorkItem>(16);
        let (save_tx, mut save_rx) = mpsc::channel::<ResolvedScene>(16);

        let agg_handle = spawn_aggregator_worker(post_rx, nms_tx);
        let nms_handle = spawn_nms_worker(nms_rx, save_tx);

        // Tile 0 (offset 0,0): ship left half touching local x_max=1151 -> global [800..1151]
        let out_t0 = Array3::from_shape_vec(
            (1, 1, 6),
            vec![800.0, 500.0, 1151.0, 580.0, 0.78, 4.0],
        )
        .unwrap()
        .into_dyn();
        let task_t0 = InferenceTask {
            image_data: Vec::new(),
            source_path: "/tmp/scene.tif".to_string(),
            global_offset_x: 0,
            global_offset_y: 0,
        };

        // Tile 1 (offset 922,0): ship right half touching local x_min=0 -> global [922..1260]
        let out_t1 = Array3::from_shape_vec(
            (1, 1, 6),
            vec![0.0, 501.0, 338.0, 579.0, 0.72, 4.0],
        )
        .unwrap()
        .into_dyn();
        let task_t1 = InferenceTask {
            image_data: Vec::new(),
            source_path: "/tmp/scene.tif".to_string(),
            global_offset_x: 922,
            global_offset_y: 0,
        };

        post_tx
            .send(PostMessage::InferenceResult {
                outputs: out_t0,
                tasks: vec![task_t0],
            })
            .unwrap();
        post_tx
            .send(PostMessage::InferenceResult {
                outputs: out_t1,
                tasks: vec![task_t1],
            })
            .unwrap();
        post_tx
            .send(PostMessage::EndOfFile {
                source_path: "/tmp/scene.tif".to_string(),
                geo_transform: [0.0; 6],
                width: 2074,
                height: 1152,
                expected_tiles: 2,
            })
            .unwrap();
        post_tx.send(PostMessage::Terminate).unwrap();

        let resolved = save_rx.blocking_recv().expect("Expected resolved scene");
        assert_eq!(resolved.detections.len(), 1);
        assert!((resolved.detections[0].bbox.x_min - 800.0).abs() < 1e-3);
        assert!((resolved.detections[0].bbox.x_max - 1260.0).abs() < 1e-3);
        assert!(save_rx.blocking_recv().is_none());

        agg_handle.join().unwrap().unwrap();
        nms_handle.join().unwrap().unwrap();
    }
}
