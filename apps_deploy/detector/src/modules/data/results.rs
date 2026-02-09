use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufWriter;
use serde::Serialize;
use gdal::Dataset;
use crate::modules::processing::post_processing::Detection;

#[derive(Serialize)]
pub struct FinalOutput {
    pub source_image: String,
    pub geo_transform: [f64; 6],
    pub source_width: usize,
    pub source_height: usize,
    pub detections: Vec<Detection>,
}

pub fn process_and_save_results(
    results_by_path: HashMap<String, (Vec<Detection>, [f64; 6])>,
    output_dir: &PathBuf,
) -> Result<()> {
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
            source_image: path_str,
            geo_transform,
            source_width: width,
            source_height: height,
            detections,
        };

        let file_name = path.file_stem().ok_or_else(|| anyhow::anyhow!("Invalid file stem"))?.to_str().ok_or_else(|| anyhow::anyhow!("Non-UTF8 file name"))?;
        let output_path = output_dir.join(format!("{}_results.json", file_name));
        let file = File::create(&output_path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), &output_data)?;
        println!("Saved: {:?}", output_path);
    }
    Ok(())
}