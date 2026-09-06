use crate::io::LoadedImage;
use crate::model::FinalOutput;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    // File & Directory Navigation
    ResultsDirChanged(String),
    ScanResultsDir,
    FileFilterChanged(String),
    FileFilterSubmitted,
    SelectJsonFile(usize),
    NextFile,
    PrevFile,
    JsonLoaded(Result<(PathBuf, FinalOutput), String>),
    ImageLoaded(Result<LoadedImage, String>),

    // Annotation Filters
    ToggleClass(usize, bool),
    SelectAllClasses,
    TurnOffAllClasses,
    ConfidenceChanged(f32),
    ToggleLabels(bool),

    // Canvas / Viewer Interaction
    ViewerStatus {
        zoom: f32,
        cursor: Option<(f32, f32)>,
    },
    ResetView,
}
