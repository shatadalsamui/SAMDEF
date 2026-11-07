use anyhow::Result;
use crate::geo::{format_lat_lon, is_valid_geotransform, pixel_to_geo, GeoTransform};
use crate::model::{class_name, DisplayDetection};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Export detections to standard GIS GeoJSON FeatureCollection format
pub fn export_geojson<P: AsRef<Path>>(
    path: P,
    detections: &[DisplayDetection],
    gt: &GeoTransform,
    image_name: &str,
) -> Result<()> {
    let has_geo = is_valid_geotransform(gt);
    let mut features = Vec::new();

    for d in detections {
        // Calculate 4 corners of bounding box in geographic coordinates (or pixel fallback)
        let coords = if has_geo {
            let (lon_tl, lat_tl) = pixel_to_geo(gt, d.x_min as f64, d.y_min as f64);
            let (lon_tr, lat_tr) = pixel_to_geo(gt, d.x_max as f64, d.y_min as f64);
            let (lon_br, lat_br) = pixel_to_geo(gt, d.x_max as f64, d.y_max as f64);
            let (lon_bl, lat_bl) = pixel_to_geo(gt, d.x_min as f64, d.y_max as f64);
            vec![vec![
                [lon_tl, lat_tl],
                [lon_tr, lat_tr],
                [lon_br, lat_br],
                [lon_bl, lat_bl],
                [lon_tl, lat_tl], // Close the polygon ring
            ]]
        } else {
            vec![vec![
                [d.x_min as f64, d.y_min as f64],
                [d.x_max as f64, d.y_min as f64],
                [d.x_max as f64, d.y_max as f64],
                [d.x_min as f64, d.y_max as f64],
                [d.x_min as f64, d.y_min as f64],
            ]]
        };

        let (w_m, h_m) = d.ground_dimensions_m(gt);
        let area_m2 = d.ground_area_sqm(gt);

        let feature = json!({
            "type": "Feature",
            "properties": {
                "id": d.id,
                "class_id": d.class_id,
                "class_name": class_name(d.class_id),
                "confidence": d.confidence,
                "confidence_pct": (d.confidence * 100.0).round(),
                "pixel_bbox": {
                    "x_min": d.x_min,
                    "y_min": d.y_min,
                    "x_max": d.x_max,
                    "y_max": d.y_max,
                },
                "ground_width_m": (w_m * 10.0).round() / 10.0,
                "ground_height_m": (h_m * 10.0).round() / 10.0,
                "ground_area_sqm": (area_m2 * 10.0).round() / 10.0,
                "source_image": image_name
            },
            "geometry": {
                "type": "Polygon",
                "coordinates": coords
            }
        });
        features.push(feature);
    }

    let root = json!({
        "type": "FeatureCollection",
        "name": format!("SAMDEF_Exploitation_{}", image_name),
        "crs": {
            "type": "name",
            "properties": {
                "name": if has_geo { "urn:ogc:def:crs:OGC:1.3:CRS84" } else { "PixelCoordinates" }
            }
        },
        "features": features
    });

    let content = serde_json::to_string_pretty(&root)?;
    fs::write(path, content)?;
    Ok(())
}

/// Export detections table to CSV
pub fn export_csv<P: AsRef<Path>>(
    path: P,
    detections: &[DisplayDetection],
    gt: &GeoTransform,
) -> Result<()> {
    let mut w = String::from(
        "id,class_id,class_name,confidence,x_min,y_min,x_max,y_max,center_lon,center_lat,area_sqm\n",
    );
    for d in detections {
        let (lon, lat) = d.geo_center(gt);
        let area_m2 = d.ground_area_sqm(gt);
        w.push_str(&format!(
            "{},{},\"{}\",{:.4},{:.1},{:.1},{:.1},{:.1},{:.7},{:.7},{:.1}\n",
            d.id,
            d.class_id,
            class_name(d.class_id),
            d.confidence,
            d.x_min,
            d.y_min,
            d.x_max,
            d.y_max,
            lon,
            lat,
            area_m2
        ));
    }
    fs::write(path, w)?;
    Ok(())
}

/// Export intelligence summary report in Markdown
pub fn export_summary_report<P: AsRef<Path>>(
    path: P,
    detections: &[DisplayDetection],
    gt: &GeoTransform,
    image_name: &str,
    visible_count: usize,
    threshold: f32,
) -> Result<()> {
    let mut class_counts = [0usize; 8];
    for d in detections {
        if d.class_id < 8 {
            class_counts[d.class_id] += 1;
        }
    }

    let has_geo = is_valid_geotransform(gt);
    let (top_left_lon, top_left_lat) = if has_geo {
        pixel_to_geo(gt, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let mut md = format!(
        "# SAMDEF GIS Intelligence Report\n\n\
         **Source Scene**: `{}`\n\
         **Confidence Cutoff**: `{:.0}%`\n\
         **Active Target Count**: `{}` / `{}` total\n",
        image_name,
        threshold * 100.0,
        visible_count,
        detections.len()
    );

    if has_geo {
        md.push_str(&format!(
            "**Upper-Left Georeference**: {}\n\n",
            format_lat_lon(top_left_lon, top_left_lat)
        ));
    } else {
        md.push('\n');
    }

    md.push_str("## Target Class Breakdown\n\n");
    md.push_str("| Class ID | Class Description | Detected Objects | Proportion |\n");
    md.push_str("|---|---|---|---|\n");

    let total = visible_count.max(1) as f32;
    for (i, &count) in class_counts.iter().enumerate() {
        let pct = (count as f32 / total) * 100.0;
        md.push_str(&format!(
            "| {} | {} | {} | {:.1}% |\n",
            i,
            class_name(i),
            count,
            pct
        ));
    }

    fs::write(path, md)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_suite() {
        let temp_dir = std::env::temp_dir().join("samdef_export_test");
        let _ = fs::create_dir_all(&temp_dir);

        let dets = vec![
            DisplayDetection {
                id: 1,
                x_min: 100.0,
                y_min: 100.0,
                x_max: 200.0,
                y_max: 200.0,
                class_id: 0,
                confidence: 0.95,
            },
            DisplayDetection {
                id: 2,
                x_min: 300.0,
                y_min: 300.0,
                x_max: 350.0,
                y_max: 380.0,
                class_id: 1,
                confidence: 0.85,
            },
        ];

        let gt: GeoTransform = [103.99, 0.00001, 0.0, 1.32, 0.0, -0.00001];

        // 1. GeoJSON
        let geojson_path = temp_dir.join("test.geojson");
        export_geojson(&geojson_path, &dets, &gt, "sample.tif").unwrap();
        let geojson_content = fs::read_to_string(&geojson_path).unwrap();
        assert!(geojson_content.contains("\"FeatureCollection\""));
        assert!(geojson_content.contains("\"Long Truck\""));

        // 2. CSV
        let csv_path = temp_dir.join("test.csv");
        export_csv(&csv_path, &dets, &gt).unwrap();
        let csv_content = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_content.contains("id,class_id,class_name"));
        assert!(csv_content.contains("Long Truck"));

        // 3. Markdown
        let md_path = temp_dir.join("test.md");
        export_summary_report(&md_path, &dets, &gt, "sample.tif", 2, 0.5).unwrap();
        let md_content = fs::read_to_string(&md_path).unwrap();
        assert!(md_content.contains("# SAMDEF GIS Intelligence Report"));
        assert!(md_content.contains("sample.tif"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
