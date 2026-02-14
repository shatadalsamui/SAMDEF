use crate::modules::data::payload::DetectionPayload;
use crate::modules::io::publisher::ZenohPublisher;
use crate::modules::processing::post_processing::Detection;
use anyhow::Result;
use gdal::Dataset;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct FinalOutput {
    pub source_image: String,
    pub geo_transform: [f64; 6],
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

pub async fn process_and_save_results(
    results_by_path: HashMap<String, (Vec<Detection>, [f64; 6])>,
    output_dir: &PathBuf,
) -> Result<()> {
    // Initialize ZenohPublisher once (async)
    let publisher = ZenohPublisher::new().await;

    for (path_str, (mut detections, geo_transform)) in results_by_path {
        let path = PathBuf::from(&path_str);
        println!(
            "Processing results for {:?} ({} detections)",
            path,
            detections.len()
        );

        let dataset = Dataset::open(&path)?;
        let (width, height) = dataset.raster_size();

        // Global NMS Pass
        crate::modules::processing::post_processing::non_maximum_suppression(&mut detections, 0.45);

        let output_data = FinalOutput {
            source_image: path_str.clone(),
            geo_transform,
            source_width: width,
            source_height: height,
            detections: detections.clone(),
        };

        // Map post_processing::Detection to payload::Detection
        let payload_detections: Vec<crate::modules::data::payload::Detection> = detections
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
        let file = File::create(&output_path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), &output_data)?;
        println!("Saved: {:?}", output_path);

        // Build and publish DetectionPayload via Zenoh (async)
        let payload = DetectionPayload {
            source_image: path_str,
            geo_transform,
            source_width: width,
            source_height: height,
            detections: payload_detections,
        };
        publisher.publish_detection(&payload).await;
        println!("Published detection payload via Zenoh.");
    }
    Ok(())
}
