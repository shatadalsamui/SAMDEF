use anyhow::Result;
use serde::Serialize;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone, Copy)]
pub struct RunStats {
    pub total_images: usize,
    pub total_detections: usize,
}

#[derive(Serialize)]
pub struct SamdefBenchmarkSummary {
    pub framework: &'static str,
    pub timestamp_unix_s: u64,
    pub model_file: String,
    pub slice_size: String,
    pub overlap: &'static str,
    pub batch_size: usize,
    pub producer_parallelism: usize,
    pub total_images: usize,
    pub total_wall_clock_seconds: f64,
    pub mean_seconds_per_image: f64,
    pub throughput_images_per_second: f64,
    pub throughput_scenes_per_minute: f64,
    pub total_detections: usize,
}

fn round_to(val: f64, decimals: i32) -> f64 {
    let scale = 10f64.powi(decimals);
    (val * scale).round() / scale
}

pub fn build_summary(
    model_path: &Path,
    tile_size: usize,
    batch_size: usize,
    producer_parallelism: usize,
    stats: RunStats,
    elapsed_secs: f64,
) -> SamdefBenchmarkSummary {
    let secs = elapsed_secs.max(1e-6);
    let imgs = stats.total_images as f64;
    let model_file = model_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let timestamp_unix_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    SamdefBenchmarkSummary {
        framework: "SAMDEF (Rust + ort CUDA)",
        timestamp_unix_s,
        model_file,
        slice_size: format!("{}x{}", tile_size, tile_size),
        overlap: "20%",
        batch_size,
        producer_parallelism,
        total_images: stats.total_images,
        total_wall_clock_seconds: round_to(secs, 2),
        mean_seconds_per_image: if stats.total_images > 0 { round_to(secs / imgs, 4) } else { 0.0 },
        throughput_images_per_second: round_to(imgs / secs, 3),
        throughput_scenes_per_minute: round_to((imgs / secs) * 60.0, 2),
        total_detections: stats.total_detections,
    }
}

pub fn write_summary(path: &Path, summary: &SamdefBenchmarkSummary) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), summary)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_rounds_summary_metrics() {
        let s = build_summary(
            Path::new("/tmp/run15_FP16.onnx"),
            1152,
            1,
            4,
            RunStats { total_images: 281, total_detections: 12000 },
            46.294,
        );
        assert_eq!(s.model_file, "run15_FP16.onnx");
        assert_eq!(s.total_wall_clock_seconds, 46.29);
        assert_eq!(s.total_images, 281);
        assert_eq!(s.total_detections, 12000);
    }
}

