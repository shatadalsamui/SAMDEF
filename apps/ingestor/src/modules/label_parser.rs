use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GeoJson {
	pub features: Vec<Label>,
}

#[derive(Debug, Deserialize)]
pub struct Label {
	pub properties: Properties,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
	pub image_id: String,
	pub type_id: u32,
	pub bounds_imcoords: String,
}

/// Loads and parses the GeoJSON label file into a vector of labels
pub fn load_labels(path: &str) -> Result<Vec<Label>, Box<dyn std::error::Error>> {
	let geojson_str = fs::read_to_string(path)?;
	let geojson: GeoJson = serde_json::from_str(&geojson_str)?;
	Ok(geojson.features)
}
