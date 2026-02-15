use crate::modules::data::payload::DetectionPayload;
use crate::modules::io::publisher::ZenohPublisher;
use crate::modules::processing::post_processing::Detection;
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

/// Output struct for JSON serialization
#[derive(Serialize)]
pub struct FinalOutput {
    pub source_image: String,
    pub geo_transform: [f64; 6],
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

/// Process and save results for a single TIFF (JSON + Zenoh)
pub async fn process_single_image(
    path_str: &str,
    geo_transform: [f64; 6],
    detections: Vec<Detection>,
    output_dir: &PathBuf,
    publisher: &ZenohPublisher,
    width: usize,
    height: usize,
) -> Result<()> {
    let path = PathBuf::from(path_str);
    println!(
        "Processing results for {:?} ({} detections)",
        path,
        detections.len()
    );

    // Global NMS Pass
    let mut nms_detections = detections;
    crate::modules::processing::post_processing::non_maximum_suppression(&mut nms_detections, 0.45);

    let output_data = FinalOutput {
        source_image: path_str.to_string(),
        geo_transform,
        source_width: width,
        source_height: height,
        detections: nms_detections.clone(),
    };

    // Map post_processing::Detection to payload::Detection
    let payload_detections: Vec<crate::modules::data::payload::Detection> = nms_detections
        .iter()
        .map(|d| crate::modules::data::payload::Detection {
            bbox: crate::modules::data::payload::BoundingBox {
                x_min: d.bbox.x_min,
                y_min: d.bbox.y_min,
                x_max: d.bbox.x_max,
                y_max: d.bbox.y_max,
            },
            class_id: d.class_id,
            confidence: d.confidence,
        })
        .collect();

    // Save JSON output as before
    let file_name = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid file stem"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Non-UTF8 file name"))?;
    let output_path = output_dir.join(format!("{}_results.json", file_name));
    let file = std::fs::File::create(&output_path)?;
    serde_json::to_writer_pretty(std::io::BufWriter::new(file), &output_data)?;
    println!("Saved: {:?}", output_path);

    // Build and publish DetectionPayload via Zenoh (async)
    let payload = DetectionPayload {
        source_image: path_str.to_string(),
        geo_transform,
        source_width: width,
        source_height: height,
        detections: payload_detections,
    };
    publisher.publish_detection(&payload).await;
    println!("Published detection payload via Zenoh.");

    Ok(())
}
