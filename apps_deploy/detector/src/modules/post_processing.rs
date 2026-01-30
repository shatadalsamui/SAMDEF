use ndarray::ArrayView2;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Detection {
    pub bbox: BoundingBox,
    pub class_id: usize,
    pub confidence: f32,
    pub source_tile: String,
}

/// Parses the Raw Output from the ONNX Model
/// Format: [Batch, 300, 6] -> [x_min, y_min, x_max, y_max, confidence, class_id]
pub fn parse_output(output: ArrayView2<f32>, tile_filename: &str) -> Vec<Detection> {
    let mut detections = Vec::new();
    
    // CONFIDENCE THRESHOLDS (0.0 to 1.0)
    const CLASS_THRESHOLDS: [f32; 6] = [0.1, 0.1, 0.1, 0.1, 0.25, 0.25];

    for proposal in output.rows() {
        // Direct Mapping: [x_min, y_min, x_max, y_max, confidence, class_id]
        let x_min = proposal[0];
        let y_min = proposal[1];
        let x_max = proposal[2];
        let y_max = proposal[3];
        let confidence = proposal[4]; // Already a probability
        let class_id = proposal[5] as usize;

        if class_id < CLASS_THRESHOLDS.len() && confidence > CLASS_THRESHOLDS[class_id] {
            detections.push(Detection {
                bbox: BoundingBox { x_min, y_min, x_max, y_max },
                class_id,
                confidence,
                source_tile: tile_filename.to_string(),
            });
        }
    }
    detections
}
