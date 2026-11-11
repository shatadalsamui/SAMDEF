use crate::modules::data::task::InferenceTask;
use crate::modules::processing::post_processing::{
    compute_local_trunc_mask, parse_output, RawDetection,
};
use anyhow::Result;
use crossbeam::channel;
use std::collections::HashMap;
use std::thread::{self, JoinHandle};

pub enum PostMessage {
    InferenceResult {
        outputs: ndarray::ArrayD<f32>,
        tasks: Vec<InferenceTask>,
    },
    EndOfFile {
        source_path: String,
        geo_transform: [f64; 6],
        width: usize,
        height: usize,
        expected_tiles: usize,
    },
    Terminate,
}

pub struct NmsWorkItem {
    pub source_path: String,
    pub geo_transform: [f64; 6],
    pub width: usize,
    pub height: usize,
    pub raw_detections: Vec<RawDetection>,
}

struct FileState {
    detections: Vec<RawDetection>,
    geo_transform: [f64; 6],
    width: usize,
    height: usize,
    expected_tiles: usize,
    processed_tiles: usize,
    is_eof_received: bool,
}

fn dispatch_completed_scenes(
    pending_files: &mut HashMap<String, FileState>,
    nms_tx: &channel::Sender<NmsWorkItem>,
) {
    let completed_paths: Vec<String> = pending_files
        .iter()
        .filter(|(_, state)| state.is_eof_received && state.processed_tiles >= state.expected_tiles)
        .map(|(path, _)| path.clone())
        .collect();

    for path in completed_paths {
        if let Some(mut state) = pending_files.remove(&path) {
            let work = NmsWorkItem {
                source_path: path,
                geo_transform: state.geo_transform,
                width: state.width,
                height: state.height,
                raw_detections: std::mem::take(&mut state.detections),
            };
            let _ = nms_tx.send(work);
        }
    }
}

/// Stage 3: Dedicated OS thread draining GPU outputs, tagging local edge truncation,
/// and forwarding completed scenes to the isolated NMS worker without blocking GPU.
pub fn spawn_aggregator_worker(
    post_rx: channel::Receiver<PostMessage>,
    nms_tx: channel::Sender<NmsWorkItem>,
) -> JoinHandle<Result<()>> {
    thread::spawn(move || -> Result<()> {
        let mut pending_files: HashMap<String, FileState> = HashMap::new();

        while let Ok(msg) = post_rx.recv() {
            match msg {
                PostMessage::InferenceResult { outputs, tasks } => {
                    for (i, single_output_dyn) in outputs.outer_iter().enumerate() {
                        let task = &tasks[i];
                        if let Ok(single_output_2d) =
                            single_output_dyn.into_dimensionality::<ndarray::Ix2>()
                        {
                            let parsed = parse_output(single_output_2d);
                            let tile_origin = (task.global_offset_x, task.global_offset_y);
                            let ox = task.global_offset_x as f64;
                            let oy = task.global_offset_y as f64;

                            let raw_dets: Vec<RawDetection> = parsed
                                .into_iter()
                                .map(|mut det| {
                                    // Tag local edge truncation before adding global offsets.
                                    let trunc_mask =
                                        compute_local_trunc_mask(&det.bbox, tile_origin);
                                    det.bbox.x_min = (det.bbox.x_min as f64 + ox) as f32;
                                    det.bbox.y_min = (det.bbox.y_min as f64 + oy) as f32;
                                    det.bbox.x_max = (det.bbox.x_max as f64 + ox) as f32;
                                    det.bbox.y_max = (det.bbox.y_max as f64 + oy) as f32;
                                    RawDetection::from_detection(det, tile_origin, trunc_mask)
                                })
                                .collect();

                            let entry = pending_files
                                .entry(task.source_path.clone())
                                .or_insert(FileState {
                                    detections: Vec::new(),
                                    geo_transform: [0.0; 6],
                                    width: 0,
                                    height: 0,
                                    expected_tiles: usize::MAX,
                                    processed_tiles: 0,
                                    is_eof_received: false,
                                });

                            entry.detections.extend(raw_dets);
                            entry.processed_tiles += 1;
                        }
                    }
                }
                PostMessage::EndOfFile {
                    source_path,
                    geo_transform,
                    width,
                    height,
                    expected_tiles,
                } => {
                    let entry = pending_files.entry(source_path).or_insert(FileState {
                        detections: Vec::new(),
                        geo_transform,
                        width,
                        height,
                        expected_tiles: 0,
                        processed_tiles: 0,
                        is_eof_received: false,
                    });

                    entry.geo_transform = geo_transform;
                    entry.width = width;
                    entry.height = height;
                    entry.expected_tiles = expected_tiles;
                    entry.is_eof_received = true;
                }
                PostMessage::Terminate => {
                    break;
                }
            }

            dispatch_completed_scenes(&mut pending_files, &nms_tx);
        }

        Ok(())
    })
}

