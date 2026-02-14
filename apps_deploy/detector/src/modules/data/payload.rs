use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DetectionPayload {
    pub source_image: String,
    pub geo_transform: [f64; 6],
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detection {
    pub bbox: BoundingBox,
    pub class_id: usize,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}
