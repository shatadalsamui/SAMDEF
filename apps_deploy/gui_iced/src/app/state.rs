use crate::io::{default_results_dir, scan_json_directory, LoadedImage};
use crate::model::DisplayDetection;
use std::path::PathBuf;

pub struct SamdefViewer {
    // Current loaded data
    pub current_image: Option<LoadedImage>,
    pub current_json_path: Option<PathBuf>,
    pub detections: Vec<DisplayDetection>,

    // File list & directory scanning
    pub results_dir_input: String,
    pub available_json_files: Vec<PathBuf>,
    pub current_file_idx: Option<usize>,
    pub file_filter_query: String,

    // Annotation controls
    pub class_visible: [bool; 8],
    pub confidence_threshold: f32,
    pub show_labels: bool,

    // Viewer telemetry and state
    pub current_zoom: f32,
    pub cursor_coord: Option<(f32, f32)>,
    pub reset_counter: usize,
    pub status_message: String,
    pub is_loading: bool,
}

impl Default for SamdefViewer {
    fn default() -> Self {
        let results_dir = default_results_dir();

        let initial_files = scan_json_directory(&results_dir).unwrap_or_default();
        let initial_idx = if !initial_files.is_empty() { Some(0) } else { None };

        Self {
            current_image: None,
            current_json_path: None,
            detections: Vec::new(),
            results_dir_input: results_dir,
            available_json_files: initial_files,
            current_file_idx: initial_idx,
            file_filter_query: String::new(),
            class_visible: [true; 8],
            confidence_threshold: 0.20,
            show_labels: true,
            current_zoom: 1.0,
            cursor_coord: None,
            reset_counter: 0,
            status_message: "Ready. Scan results directory or select a JSON file.".to_string(),
            is_loading: false,
        }
    }
}

impl SamdefViewer {
    pub fn class_counts(&self) -> [usize; 8] {
        let mut counts = [0; 8];
        for d in &self.detections {
            if d.class_id < 8 && d.confidence >= self.confidence_threshold {
                counts[d.class_id] += 1;
            }
        }
        counts
    }

    pub fn count_visible_detections(&self) -> usize {
        let counts = self.class_counts();
        counts
            .iter()
            .enumerate()
            .filter(|(i, _)| self.class_visible[*i])
            .map(|(_, &c)| c)
            .sum()
    }
}


