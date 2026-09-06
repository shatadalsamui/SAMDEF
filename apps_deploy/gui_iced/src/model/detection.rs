use serde::{Deserialize, Serialize};

/// Bounding box coordinate rectangle
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct BBoxCoords {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

/// Detection format from detector's *_results.json
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Detection {
    pub bbox: BBoxCoords,
    pub class_id: usize,
    pub confidence: f32,
}

/// Full detector JSON output structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinalOutput {
    pub source_image: String,
    #[serde(default)]
    pub geo_transform: [f64; 6],
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

/// Unified detection struct used internally by the viewer
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayDetection {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub class_id: usize,
    pub confidence: f32,
}

impl From<&Detection> for DisplayDetection {
    fn from(d: &Detection) -> Self {
        Self {
            x_min: d.bbox.x_min,
            y_min: d.bbox.y_min,
            x_max: d.bbox.x_max,
            y_max: d.bbox.y_max,
            class_id: d.class_id,
            confidence: d.confidence,
        }
    }
}
