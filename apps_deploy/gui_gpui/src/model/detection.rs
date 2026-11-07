#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use crate::geo::{estimate_gsd_meters, pixel_to_geo, GeoTransform};

/// Bounding box coordinate rectangle in pixel coordinates
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct BBoxCoords {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl BBoxCoords {
    pub fn width(&self) -> f32 {
        (self.x_max - self.x_min).max(0.0)
    }

    pub fn height(&self) -> f32 {
        (self.y_max - self.y_min).max(0.0)
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.x_min + self.x_max) / 2.0, (self.y_min + self.y_max) / 2.0)
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x_min && px <= self.x_max && py >= self.y_min && py <= self.y_max
    }
}

/// Raw detection format deserialized from detector's `*_results.json`
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
    pub geo_transform: GeoTransform,
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

/// Unified display detection entity used across viewer, table, and inspector
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayDetection {
    pub id: usize,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub class_id: usize,
    pub confidence: f32,
}

impl DisplayDetection {
    pub fn width(&self) -> f32 {
        (self.x_max - self.x_min).max(0.0)
    }

    pub fn height(&self) -> f32 {
        (self.y_max - self.y_min).max(0.0)
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.x_min + self.x_max) / 2.0, (self.y_min + self.y_max) / 2.0)
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x_min && px <= self.x_max && py >= self.y_min && py <= self.y_max
    }

    /// Calculate geographic center (Lon, Lat) from GeoTransform
    pub fn geo_center(&self, gt: &GeoTransform) -> (f64, f64) {
        let (cx, cy) = self.center();
        pixel_to_geo(gt, cx as f64, cy as f64)
    }

    /// Estimate approximate ground footprint in square meters
    pub fn ground_area_sqm(&self, gt: &GeoTransform) -> f64 {
        let gsd = estimate_gsd_meters(gt);
        (self.area() as f64) * (gsd * gsd)
    }

    /// Estimate approximate ground dimensions (width_m, height_m)
    pub fn ground_dimensions_m(&self, gt: &GeoTransform) -> (f64, f64) {
        let gsd = estimate_gsd_meters(gt);
        (self.width() as f64 * gsd, self.height() as f64 * gsd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_math_and_deserialization() {
        let json_str = r#"{
            "source_image": "test.tif",
            "geo_transform": [100.0, 0.00001, 0.0, 10.0, 0.0, -0.00001],
            "source_width": 2048,
            "source_height": 2048,
            "detections": [
                {
                    "bbox": { "x_min": 100.0, "y_min": 200.0, "x_max": 150.0, "y_max": 250.0 },
                    "class_id": 2,
                    "confidence": 0.88
                }
            ]
        }"#;

        let output: FinalOutput = serde_json::from_str(json_str).expect("Valid JSON");
        assert_eq!(output.detections.len(), 1);
        let det = &output.detections[0];
        assert_eq!(det.class_id, 2);
        assert!((det.confidence - 0.88).abs() < 1e-4);
        assert_eq!(det.bbox.width(), 50.0);
        assert_eq!(det.bbox.height(), 50.0);
        assert_eq!(det.bbox.area(), 2500.0);
        assert_eq!(det.bbox.center(), (125.0, 225.0));

        let display = DisplayDetection {
            id: 1,
            x_min: det.bbox.x_min,
            y_min: det.bbox.y_min,
            x_max: det.bbox.x_max,
            y_max: det.bbox.y_max,
            class_id: det.class_id,
            confidence: det.confidence,
        };

        assert!(display.contains_point(120.0, 220.0));
        assert!(!display.contains_point(50.0, 50.0));

        let (lon, lat) = display.geo_center(&output.geo_transform);
        assert!(lon > 100.0);
        assert!(lat < 10.0);

        let area_m2 = display.ground_area_sqm(&output.geo_transform);
        assert!(area_m2 > 0.0);
    }
}
