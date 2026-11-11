use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct BBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Detection {
    pub bbox: BBox,
    pub class_id: usize,
    pub confidence: f32,
}

/// Supports both SAMDEF (`geo_transform`) and SAHI baseline (`elapsed_seconds`, `detections_count`) JSONs.
#[derive(Debug, Deserialize)]
pub struct FinalOutput {
    pub source_image: String,
    #[serde(default)]
    pub detections: Vec<Detection>,
}

pub struct PostConfig {
    pub json_dir: String,
    pub output_dir: String,
    pub conf_threshold: f32,
}

pub fn parse_config() -> PostConfig {
    let mut json_dir = std::env::var("JSON_DIR").ok();
    let mut output_dir = std::env::var("OUTPUT_DIR").ok();
    let mut conf_threshold = std::env::var("CONF_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0);

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json-dir" => json_dir = args.next(),
            "--output-dir" => output_dir = args.next(),
            "--conf" => {
                if let Some(v) = args.next().and_then(|s| s.parse::<f32>().ok()) {
                    conf_threshold = v;
                }
            }
            _ => {}
        }
    }

    PostConfig {
        json_dir: json_dir.expect("JSON_DIR must be set in .env or via --json-dir"),
        output_dir: output_dir.expect("OUTPUT_DIR must be set in .env or via --output-dir"),
        conf_threshold,
    }
}

pub fn is_result_json(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.ends_with("_results.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_sahi_and_samdef_result_schemas() {
        let sahi = r#"{
            "source_image": "/tmp/1038.tif",
            "source_width": 3195,
            "source_height": 3215,
            "elapsed_seconds": 2.02,
            "detections_count": 1,
            "detections": [{"bbox": {"x_min": 1.0, "y_min": 2.0, "x_max": 3.0, "y_max": 4.0}, "class_id": 9, "confidence": 0.9}]
        }"#;
        let samdef = r#"{
            "source_image": "/tmp/1038.tif",
            "geo_transform": [103.9, 1e-5, 0.0, 1.3, 0.0, -1e-5],
            "source_width": 3195,
            "source_height": 3215,
            "detections": []
        }"#;
        let parsed_sahi: FinalOutput = serde_json::from_str(sahi).unwrap();
        let parsed_samdef: FinalOutput = serde_json::from_str(samdef).unwrap();
        assert_eq!(parsed_sahi.detections.len(), 1);
        assert_eq!(parsed_samdef.detections.len(), 0);
    }

    #[test]
    fn filters_summary_json_and_accepts_results_json() {
        let tmp = std::env::temp_dir().join("samdef_pp_filter_test");
        std::fs::create_dir_all(&tmp).unwrap();
        let res = tmp.join("1038_results.json");
        let summary = tmp.join("benchmark_summary.json");
        std::fs::write(&res, "{}").unwrap();
        std::fs::write(&summary, "{}").unwrap();

        assert!(is_result_json(&res));
        assert!(!is_result_json(&summary));
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}

