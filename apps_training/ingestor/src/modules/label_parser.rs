use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct GeoJson {
	pub features: Vec<Label>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
	pub properties: Properties,
}

impl Label {
	/// Parse bounds_imcoords "xmin,ymin,xmax,ymax" into integer coords.
	pub fn parse_bounds(&self) -> Option<(u32, u32, u32, u32)> {
		let parts: Vec<&str> = self.properties.bounds_imcoords.split(',').collect();
		if parts.len() != 4 {
			return None;
		}
		let xmin = parts.get(0)?.trim().parse::<f64>().ok()? as u32;
		let ymin = parts.get(1)?.trim().parse::<f64>().ok()? as u32;
		let xmax = parts.get(2)?.trim().parse::<f64>().ok()? as u32;
		let ymax = parts.get(3)?.trim().parse::<f64>().ok()? as u32;
		Some((xmin, ymin, xmax, ymax))
	}
}

#[derive(Debug, Deserialize, Clone)]
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
