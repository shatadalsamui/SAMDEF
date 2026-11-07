#![allow(dead_code)]
//! Geographic transformation and geodesy utilities for SAMDEF GIS Viewer.
//! Supports GDAL 6-parameter affine geotransforms and coordinate projections.

/// GDAL GeoTransform affine coefficients:
/// gt[0]: Top-left X (e.g. Longitude or Easting)
/// gt[1]: W-E pixel resolution (pixel width)
/// gt[2]: Row rotation (typically 0.0)
/// gt[3]: Top-left Y (e.g. Latitude or Northing)
/// gt[4]: Column rotation (typically 0.0)
/// gt[5]: N-S pixel resolution (pixel height, typically negative)
pub type GeoTransform = [f64; 6];

/// Convert image pixel coordinates (X, Y) to Geographic coordinates (Lon/Easting, Lat/Northing)
#[inline]
pub fn pixel_to_geo(gt: &GeoTransform, px: f64, py: f64) -> (f64, f64) {
    let geo_x = gt[0] + px * gt[1] + py * gt[2];
    let geo_y = gt[3] + px * gt[4] + py * gt[5];
    (geo_x, geo_y)
}

/// Convert Geographic coordinates (Lon/Easting, Lat/Northing) back to image pixel coordinates
#[inline]
pub fn geo_to_pixel(gt: &GeoTransform, geo_x: f64, geo_y: f64) -> Option<(f64, f64)> {
    let det = gt[1] * gt[5] - gt[2] * gt[4];
    if det.abs() < 1e-12 {
        return None;
    }
    let dx = geo_x - gt[0];
    let dy = geo_y - gt[3];
    let px = (dx * gt[5] - dy * gt[2]) / det;
    let py = (dy * gt[1] - dx * gt[4]) / det;
    Some((px, py))
}

/// Check if GeoTransform has meaningful non-zero values
pub fn is_valid_geotransform(gt: &GeoTransform) -> bool {
    gt[1].abs() > 1e-12 && gt[5].abs() > 1e-12 && !(gt[0] == 0.0 && gt[3] == 0.0)
}

/// Format decimal degrees into human-friendly string (e.g. "103.9954° E, 1.3219° N")
pub fn format_lat_lon(lon: f64, lat: f64) -> String {
    let lon_dir = if lon >= 0.0 { "E" } else { "W" };
    let lat_dir = if lat >= 0.0 { "N" } else { "S" };
    format!("{:.5}° {}, {:.5}° {}", lon.abs(), lon_dir, lat.abs(), lat_dir)
}

/// Format decimal degrees into DMS (Degrees, Minutes, Seconds) string
pub fn format_dms(lon: f64, lat: f64) -> String {
    fn to_dms(val: f64) -> (u32, u32, f64) {
        let abs = val.abs();
        let deg = abs.floor() as u32;
        let min_rem = (abs - deg as f64) * 60.0;
        let min = min_rem.floor() as u32;
        let sec = (min_rem - min as f64) * 60.0;
        (deg, min, sec)
    }

    let (d_lon, m_lon, s_lon) = to_dms(lon);
    let (d_lat, m_lat, s_lat) = to_dms(lat);

    let lon_dir = if lon >= 0.0 { "E" } else { "W" };
    let lat_dir = if lat >= 0.0 { "N" } else { "S" };

    format!(
        "{:02}°{:02}'{:04.1}\"{}  {:03}°{:02}'{:04.1}\"{}",
        d_lat, m_lat, s_lat, lat_dir, d_lon, m_lon, s_lon, lon_dir
    )
}

/// Calculate Ground Sample Distance (GSD) in meters/pixel approximately from degrees or meters
pub fn estimate_gsd_meters(gt: &GeoTransform) -> f64 {
    let pixel_w_deg = gt[1].abs();
    let center_lat_rad = gt[3].to_radians();
    // 1 deg latitude ≈ 111,320 meters
    let meters_per_deg_lat = 111_320.0;
    // 1 deg longitude ≈ 111,320 * cos(lat) meters
    let meters_per_deg_lon = 111_320.0 * center_lat_rad.cos().abs().max(0.1);

    if pixel_w_deg > 1.0 {
        // Already in projected meters (UTM)
        pixel_w_deg
    } else {
        // Degree-based geographic coordinates (WGS-84)
        pixel_w_deg * ((meters_per_deg_lat + meters_per_deg_lon) / 2.0)
    }
}

/// Calculate Great-circle distance between two points in meters using Haversine formula
pub fn haversine_distance_meters(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in meters
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_phi = (lat2 - lat1).to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

/// Calculate bearing / azimuth from point 1 to point 2 in degrees (0..360°)
pub fn calculate_bearing_degrees(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let y = delta_lambda.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * delta_lambda.cos();
    let bearing_rad = y.atan2(x);
    (bearing_rad.to_degrees() + 360.0) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_conversion() {
        let gt: GeoTransform = [103.99538, 2.8147e-6, 0.0, 1.32192, 0.0, -2.8147e-6];
        let (lon, lat) = pixel_to_geo(&gt, 1000.0, 1000.0);
        assert!((lon - 103.99819).abs() < 1e-4);
        assert!((lat - 1.31910).abs() < 1e-4);

        let (px, py) = geo_to_pixel(&gt, lon, lat).unwrap();
        assert!((px - 1000.0).abs() < 1e-2);
        assert!((py - 1000.0).abs() < 1e-2);
    }

    #[test]
    fn test_format_dms() {
        let dms = format_dms(103.99538, 1.32192);
        assert!(dms.contains("01°"));
        assert!(dms.contains("103°"));
        assert!(dms.contains('N'));
        assert!(dms.contains('E'));
    }

    #[test]
    fn test_gsd_and_haversine() {
        let gt: GeoTransform = [100.0, 0.00001, 0.0, 0.0, 0.0, -0.00001];
        let gsd = estimate_gsd_meters(&gt);
        assert!(gsd > 1.0 && gsd < 1.2); // ~1.11 m at equator

        let d = haversine_distance_meters(0.0, 0.0, 0.0, 1.0);
        assert!((d - 111195.0).abs() < 500.0);

        let bearing = calculate_bearing_degrees(0.0, 0.0, 0.0, 1.0);
        assert!((bearing - 0.0).abs() < 1e-4); // Due North
    }
}
