use crate::modules::writer::payload::DetectionPayload;
use anyhow::Result;
use sqlx::PgPool;

pub async fn insert_detections(pool: &PgPool, payload: &DetectionPayload) -> Result<()> {
    // Prepare data for bulk insert
    let source_files: Vec<&str> = vec![&payload.source_image; payload.detections.len()];
    let class_ids: Vec<i32> = payload
        .detections
        .iter()
        .map(|d| d.class_id as i32)
        .collect();
    let confidences: Vec<f32> = payload.detections.iter().map(|d| d.confidence).collect();

    // Convert pixel coordinates to WKT strings for PostGIS geometries
    let geom_points: Vec<String> = payload
        .detections
        .iter()
        .map(|d| {
            let center_x = d.bbox.x_min + (d.bbox.x_max - d.bbox.x_min) / 2.0;
            let center_y = d.bbox.y_min + (d.bbox.y_max - d.bbox.y_min) / 2.0;
            let lon = pixel_to_lon(center_x, payload);
            let lat = pixel_to_lat(center_y, payload);
            format!("POINT({} {})", lon, lat)
        })
        .collect();

    let geom_bboxes: Vec<String> = payload
        .detections
        .iter()
        .map(|d| {
            let lon_min = pixel_to_lon(d.bbox.x_min, payload);
            let lat_min = pixel_to_lat(d.bbox.y_min, payload);
            let lon_max = pixel_to_lon(d.bbox.x_max, payload);
            let lat_max = pixel_to_lat(d.bbox.y_max, payload);
            format!(
                "POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))",
                lon_min,
                lat_min,
                lon_max,
                lat_min,
                lon_max,
                lat_max,
                lon_min,
                lat_max,
                lon_min,
                lat_min
            )
        })
        .collect();

    // Bulk insert using UNNEST
    sqlx::query(
        "INSERT INTO detections (source_file, class_id, confidence, geom_point, geom_bbox)
         SELECT unnest($1::text[]), unnest($2::int[]), unnest($3::float4[]),
                ST_SetSRID(ST_GeomFromText(unnest($4::text[])), 4326),
                ST_SetSRID(ST_GeomFromText(unnest($5::text[])), 4326)",
    )
    .bind(&source_files)
    .bind(&class_ids)
    .bind(&confidences)
    .bind(&geom_points)
    .bind(&geom_bboxes)
    .execute(pool)
    .await?;

    println!(
        "Inserted {} detections from {}",
        payload.detections.len(),
        payload.source_image
    );
    Ok(())
}

// Helper functions to convert pixel coordinates to lat/lon using GDAL affine transform
fn pixel_to_lon(x: f32, payload: &DetectionPayload) -> f64 {
    payload.geo_transform[0] + x as f64 * payload.geo_transform[1] + 0.0 * payload.geo_transform[2]
}

fn pixel_to_lat(y: f32, payload: &DetectionPayload) -> f64 {
    payload.geo_transform[3] + 0.0 * payload.geo_transform[4] + y as f64 * payload.geo_transform[5]
}

// Insert into detections_pixels table
pub async fn insert_detections_pixels(pool: &PgPool, payload: &DetectionPayload) -> Result<()> {
    let source_files: Vec<&str> = vec![&payload.source_image; payload.detections.len()];
    let class_ids: Vec<i32> = payload
        .detections
        .iter()
        .map(|d| d.class_id as i32)
        .collect();
    let confidences: Vec<f32> = payload.detections.iter().map(|d| d.confidence).collect();
    let x_mins: Vec<f32> = payload.detections.iter().map(|d| d.bbox.x_min).collect();
    let y_mins: Vec<f32> = payload.detections.iter().map(|d| d.bbox.y_min).collect();
    let x_maxs: Vec<f32> = payload.detections.iter().map(|d| d.bbox.x_max).collect();
    let y_maxs: Vec<f32> = payload.detections.iter().map(|d| d.bbox.y_max).collect();

    sqlx::query(
        "INSERT INTO detections_pixels (source_file, class_id, confidence, x_min, y_min, x_max, y_max)
         SELECT unnest($1::text[]), unnest($2::int[]), unnest($3::float4[]),
                unnest($4::float4[]), unnest($5::float4[]), unnest($6::float4[]), unnest($7::float4[])"
    )
    .bind(&source_files)
    .bind(&class_ids)
    .bind(&confidences)
    .bind(&x_mins)
    .bind(&y_mins)
    .bind(&x_maxs)
    .bind(&y_maxs)
    .execute(pool)
    .await?;

    println!(
        "Inserted {} pixel detections from {}",
        payload.detections.len(),
        payload.source_image
    );
    Ok(())
}
