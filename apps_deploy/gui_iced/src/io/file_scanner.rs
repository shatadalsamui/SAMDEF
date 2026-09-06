use std::fs;
use std::path::{Path, PathBuf};

pub fn default_results_dir() -> String {
    std::env::var("RESULTS_DIR")
        .or_else(|_| std::env::var("OUTPUT_DIR"))
        .unwrap_or_else(|_| "/home/shatadal/SAMDEF_DATA/raw_data/inference/results".to_string())
}

pub fn default_val_images_dir() -> String {
    std::env::var("INPUT_DIR")
        .or_else(|_| std::env::var("VAL_IMAGES_DIR"))
        .unwrap_or_else(|_| "/home/shatadal/SAMDEF_DATA/val_images".to_string())
}

/// Scan a directory and return sorted JSON file paths
pub fn scan_json_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>, String> {
    let dir_path = dir.as_ref();
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", dir_path.display()));
    }

    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Resolves the corresponding TIFF image path for a JSON file
pub fn resolve_tiff_path(json_path: &Path, source_image: &str) -> PathBuf {
    // 1. Direct check of the source_image path in the JSON
    let path = PathBuf::from(source_image);
    if path.exists() {
        return path;
    }

    // 2. Try looking in val_images dir with the same stem (e.g. 1836.tif)
    let val_dir = default_val_images_dir();
    if let Some(stem) = json_path.file_stem().and_then(|s| s.to_str()) {
        let clean_stem = stem.trim_end_matches("_results");
        let candidate = PathBuf::from(&val_dir).join(format!("{}.tif", clean_stem));
        if candidate.exists() {
            return candidate;
        }

        let candidate_tiff = PathBuf::from(&val_dir).join(format!("{}.tiff", clean_stem));
        if candidate_tiff.exists() {
            return candidate_tiff;
        }
    }

    // Fallback to source_image path
    path
}
