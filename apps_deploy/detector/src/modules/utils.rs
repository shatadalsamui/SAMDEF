use std::path::Path;

// CONFIG: tile stride and filename ordering.
// STRIDE: how far tiles are spaced in global coords. If tiles abut, use 896. If they overlap (e.g., 180px), keep 716.
// SWAP_RC: set true if filenames are Map_COL_ROW and not Map_ROW_COL.
const TILE_STRIDE: f32 = 716.0;
const SWAP_RC: bool = false;

// Helper: Extract "MapA" from "MapA_05_12.jpg"
pub fn extract_tiff_id(filename: &str) -> String {
    let parts: Vec<&str> = filename.split('_').collect();
    if !parts.is_empty() {
        parts[0].to_string()
    } else {
        "unknown_map".to_string()
    }
}

// Helper: Calculate Global Offsets from filename
// Expected naming (from pre-processor): <stem>_<row>_<col>_x<offx>_y<offy>.jpg
// Falls back to stride-based offsets if x/y tokens are missing.
pub fn calculate_offsets(filename: &str) -> (f32, f32) {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parts: Vec<&str> = stem.split('_').collect();

    if parts.len() >= 5 {
        let row_part = parts[parts.len() - 4];
        let col_part = parts[parts.len() - 3];
        let x_part = parts[parts.len() - 2];
        let y_part = parts[parts.len() - 1];

        let row = row_part.parse::<f32>().ok();
        let col = col_part.parse::<f32>().ok();
        let off_x = x_part.strip_prefix('x').and_then(|v| v.parse::<f32>().ok());
        let off_y = y_part.strip_prefix('y').and_then(|v| v.parse::<f32>().ok());

        if let (Some(row), Some(col), Some(off_x), Some(off_y)) = (row, col, off_x, off_y) {
            // Respect SWAP_RC only if row/col are swapped; x/y offsets are authoritative.
            let (row_val, col_val) = if SWAP_RC { (col, row) } else { (row, col) };
            let (mut gx, mut gy) = (off_x, off_y);
            // If someone encoded row/col but not x/y, fall back to stride below.
            // With x/y present, trust them.
            if gx >= 0.0 && gy >= 0.0 {
                return (gx, gy);
            }
            let gx_stride = col_val * TILE_STRIDE;
            let gy_stride = row_val * TILE_STRIDE;
            return (gx_stride, gy_stride);
        }
    }

    // Fallback: old scheme <stem>_<row>_<col>.jpg
    if parts.len() >= 3 {
        if let (Ok(a), Ok(b)) = (
            parts[parts.len() - 2].parse::<f32>(),
            parts[parts.len() - 1].parse::<f32>(),
        ) {
            let (row, col) = if SWAP_RC { (b, a) } else { (a, b) };
            return (col * TILE_STRIDE, row * TILE_STRIDE);
        }
    }

    (0.0, 0.0)
}