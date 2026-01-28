// Import serialization traits for saving/loading structs as JSON
use serde::{Serialize, Deserialize};

/// Metadata for a single generated tile (JPG chip)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMetadata {
    pub filename: String,      // Name of the tile file, e.g., "border_scan_0_0.jpg"
    pub row_idx: usize,        // Row index in the grid (starts at 0)
    pub col_idx: usize,        // Column index in the grid (starts at 0)
    pub x_offset: u32,         // Global pixel X of the top-left corner of the tile
    pub y_offset: u32,         // Global pixel Y of the top-left corner of the tile
    pub width: u32,            // Actual width of the tile (usually 896, less at edges)
    pub height: u32,           // Actual height of the tile (usually 896, less at edges)
}

/// Manifest describing the tiling of a source image
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceManifest {
    pub source_image: String,      // Stem of the source image file (no extension)
    pub source_width: u32,         // Width of the source image in pixels
    pub source_height: u32,        // Height of the source image in pixels
    pub geo_transform: [f64; 6],   // GDAL GeoTransform array for pixel-to-coord mapping
    pub tiles: Vec<TileMetadata>,  // List of all generated tiles and their metadata
}