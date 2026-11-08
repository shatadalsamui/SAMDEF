pub mod events;
pub mod input_keys;
pub mod view;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveInput {
    ResultsDir,
    FileSearch,
    TableSearch,
}

use crate::geo::GeoTransform;
use crate::model::CLASS_COUNT;
use crate::io::{
    default_results_dir, load_tiff_or_image, resolve_tiff_path, scan_json_directory, LoadedImage,
};
use crate::model::{DisplayDetection, FinalOutput};
use crate::viewer::ViewerState;
use gpui::{Bounds, Context, Pixels};
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub struct SamdefApp {
    pub current_image: Option<LoadedImage>,
    pub current_json_path: Option<PathBuf>,
    pub detections: Vec<DisplayDetection>,
    pub geo_transform: GeoTransform,
    pub source_width: usize,
    pub source_height: usize,

    pub results_dir: String,
    pub available_files: Vec<PathBuf>,
    pub current_file_idx: Option<usize>,
    pub file_filter_query: String,
    pub table_search_query: String,
    pub active_input: Option<ActiveInput>,
    pub file_dropdown_open: bool,

    pub class_visible: [bool; CLASS_COUNT],
    pub confidence_threshold: f32,

    pub status_message: String,
    pub is_loading: bool,
    pub show_table_drawer: bool,

    pub viewer_state: ViewerState,
    pub viewport_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub minimap_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub conf_slider_bounds: Rc<Cell<Bounds<Pixels>>>,
}

impl SamdefApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let results_dir = default_results_dir();
        let initial_files = scan_json_directory(&results_dir).unwrap_or_default();
        let initial_idx = (!initial_files.is_empty()).then_some(0);

        let mut app = Self {
            current_image: None,
            current_json_path: None,
            detections: Vec::new(),
            geo_transform: [0.0; 6],
            source_width: 0,
            source_height: 0,
            results_dir,
            available_files: initial_files,
            current_file_idx: initial_idx,
            file_filter_query: String::new(),
            table_search_query: String::new(),
            active_input: None,
            file_dropdown_open: false,
            class_visible: [true; CLASS_COUNT],
            confidence_threshold: 0.20,
            status_message: "Scanning inference directory...".to_string(),
            is_loading: false,
            show_table_drawer: false,
            viewer_state: ViewerState::default(),
            viewport_bounds: Rc::new(Cell::new(Bounds::default())),
            minimap_bounds: Rc::new(Cell::new(Bounds::default())),
            conf_slider_bounds: Rc::new(Cell::new(Bounds::default())),
        };

        if let Some(idx) = initial_idx {
            app.load_file_at_index(idx, cx);
        }

        app
    }

    #[inline]
    pub fn image_dims(&self) -> (f32, f32) {
        self.current_image
            .as_ref()
            .map(|i| (i.width as f32, i.height as f32))
            .unwrap_or((1.0, 1.0))
    }

    #[inline]
    pub fn vp_bounds(&self) -> Bounds<Pixels> {
        self.viewport_bounds.get()
    }

    /// Load JSON results and decode corresponding GeoTIFF asynchronously in background
    pub fn load_file_at_index(&mut self, idx: usize, cx: &mut Context<Self>) {
        let Some(json_path) = self.available_files.get(idx).cloned() else {
            return;
        };

        self.current_file_idx = Some(idx);
        self.is_loading = true;
        let file_name = json_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        self.status_message = format!("Loading {}...", file_name);
        cx.notify();

        cx.spawn(async move |this, cx| {
            let load_result: Result<(PathBuf, FinalOutput, LoadedImage), String> = cx
                .background_executor()
                .spawn(async move {
                    let json_content = fs::read_to_string(&json_path)
                        .map_err(|e| format!("Failed to read {}: {}", json_path.display(), e))?;
                    let output: FinalOutput = serde_json::from_str(&json_content)
                        .map_err(|e| format!("Failed to parse {}: {}", json_path.display(), e))?;
                    let img_path = resolve_tiff_path(&json_path, &output.source_image);
                    let loaded_img = load_tiff_or_image(&img_path).map_err(|e| {
                        format!("Failed to load TIFF at {}: {}", img_path.display(), e)
                    })?;
                    Ok((json_path, output, loaded_img))
                })
                .await;

            this.update(cx, |this, cx| {
                this.is_loading = false;
                match load_result {
                    Ok((json_path, output, loaded_img)) => {
                        this.current_json_path = Some(json_path);
                        this.geo_transform = output.geo_transform;
                        this.source_width = output.source_width;
                        this.source_height = output.source_height;
                        this.detections = output
                            .detections
                            .iter()
                            .enumerate()
                            .map(|(id, d)| DisplayDetection {
                                id,
                                x_min: d.bbox.x_min,
                                y_min: d.bbox.y_min,
                                x_max: d.bbox.x_max,
                                y_max: d.bbox.y_max,
                                class_id: d.class_id,
                                confidence: d.confidence,
                            })
                            .collect();

                        let (w, h) = (loaded_img.width, loaded_img.height);
                        this.current_image = Some(loaded_img);
                        this.viewer_state.reset_view();
                        this.viewer_state.selected_detection_id = None;
                        this.status_message = format!(
                            "Ready: {}x{} px | {} detections",
                            w,
                            h,
                            this.detections.len()
                        );
                    }
                    Err(err) => {
                        this.status_message = format!("Error: {}", err);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn scan_directory(&mut self, cx: &mut Context<Self>) {
        match scan_json_directory(&self.results_dir) {
            Ok(files) => {
                self.status_message = format!("Found {} JSON results", files.len());
                self.available_files = files;
                if !self.available_files.is_empty() {
                    self.load_file_at_index(0, cx);
                }
            }
            Err(e) => self.status_message = e,
        }
        cx.notify();
    }

    pub fn next_file(&mut self, cx: &mut Context<Self>) {
        if !self.available_files.is_empty() {
            let next = self
                .current_file_idx
                .map_or(0, |i| (i + 1) % self.available_files.len());
            self.load_file_at_index(next, cx);
        }
    }

    pub fn prev_file(&mut self, cx: &mut Context<Self>) {
        if !self.available_files.is_empty() {
            let len = self.available_files.len();
            let prev = self
                .current_file_idx
                .map_or(0, |i| if i > 0 { i - 1 } else { len - 1 });
            self.load_file_at_index(prev, cx);
        }
    }

    pub fn class_counts(&self) -> [usize; CLASS_COUNT] {
        let mut counts = [0; CLASS_COUNT];
        for d in &self.detections {
            if d.class_id < CLASS_COUNT && d.confidence >= self.confidence_threshold {
                counts[d.class_id] += 1;
            }
        }
        counts
    }

    pub fn count_visible(&self) -> usize {
        self.class_counts()
            .iter()
            .enumerate()
            .filter(|(i, _)| self.class_visible[*i])
            .map(|(_, &c)| c)
            .sum()
    }
}

