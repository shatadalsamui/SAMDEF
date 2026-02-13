-- Create the detections table with both point and polygon geometries
CREATE TABLE IF NOT EXISTS detections (
    id           BIGSERIAL PRIMARY KEY,         -- Unique auto-incrementing identifier
    source_file  TEXT,                          -- Name of the source image file
    class_id     INTEGER,                       -- Detected object class/type
    confidence   REAL,                          -- Detection confidence score
    geom_point   geometry(Point, 4326),         -- Center of the bounding box (WGS84 lon/lat)
    geom_bbox    geometry(Polygon, 4326),       -- Full bounding box as polygon (WGS84)
    created_at   TIMESTAMP DEFAULT NOW()        -- Timestamp of insertion (defaults to now)
);

-- Create spatial indexes for efficient geospatial queries
CREATE INDEX IF NOT EXISTS detections_geom_point_gist ON detections USING GIST (geom_point);
CREATE INDEX IF NOT EXISTS detections_geom_bbox_gist ON detections USING GIST (geom_bbox);
