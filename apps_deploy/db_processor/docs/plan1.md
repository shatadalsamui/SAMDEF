# Detector-to-DB Processor Data Flow Plan (Zenoh Peer Mode)

## 1. Overview

This plan describes the process of transmitting object detection results (bounding box coordinates and metadata) from the Detector to the DB Processor using the Zenoh crate in peer mode. The goal is to ensure efficient, reliable, and schema-compliant data transfer for geospatial storage and later visualization.

---

## 2. Components

- **Detector**: Runs inference, generates detection results, writes them to JSON (existing workflow), and also publishes them via Zenoh (new workflow).
- **DB Processor**: Listens for detection messages, processes them, and writes to the PostGIS database.
- **Transport**: Zenoh (peer mode, no broker/server).

---

## 3. Data Protocol

- **Serialization**: Use `bincode` for compact, fast binary serialization.
- **Topic Naming**: `satellite/detections/{source_file}` (e.g., `satellite/detections/Map_Region_04.tif`)
- **Payload Structure**:
  - **Header**:
    - `source_image` (String): TIFF/image filename
    - `geo_transform` ([f64; 6]): GDAL affine transform for pixel-to-geo conversion
    - `source_width` (usize)
    - `source_height` (usize)
  - **Body** (Vec of detections):
    - `bbox`: `{ x_min, y_min, x_max, y_max }` (pixel coordinates, f32)
    - `class_id` (usize)
    - `confidence` (f32)

---

## 4. Detector (Producer) Flow

1. **Run inference** on the image and collect detections.
2. **For each detection**, record bounding box, class_id, and confidence.
3. **After processing a TIFF**:
    - **Write detections to JSON** (existing logic, retained for debugging and compatibility).
    - **Build a payload struct** with header and detection list (new logic).
    - **Serialize** the payload using `bincode` (new logic).
    - **Publish** the payload to Zenoh on the topic `satellite/detections/{source_file}` (new logic).
4. **Do not** write to DB or disk in any other format (no CSV, no direct DB connection).

---

## 5. DB Processor (Consumer) Flow

1. **Initialize Zenoh** in peer mode and subscribe to `satellite/detections/**`.
2. **On message arrival**:
    - **Deserialize** the payload using `bincode`.
    - **For each detection**:
        - Convert bbox pixel coordinates to:
            - **Center point** (for `geom_point`)
            - **Polygon** (for `geom_bbox`)
          using the provided `geo_transform`.
    - **Insert** all detections into the `detections` table (bulk insert, transactional).
3. **Log** the operation (e.g., number of detections, source file, time taken).
4. **No visualization or image modification** at this stage.

---

## 6. Error Handling & Robustness

- **Zenoh**: Handle reconnections and message loss gracefully.
- **Deserialization**: Validate payload structure and log errors.
- **Database**: Use transactions for atomicity; log and handle DB errors.

---

## 7. Extensibility

- Future summary/metadata tables can be updated in the same flow.
- The protocol can be extended to support new detection types or metadata.

---

## 8. Message Batching and Performance Note

- Each Zenoh message should contain all detections for a single image (TIFF).
- Batching 500–5000 detections per message is efficient and well within the capabilities of Zenoh, bincode, and PostgreSQL/PostGIS on a single laptop.
- This approach is recommended for both performance and simplicity.
- Only consider chunking if you encounter images with tens of thousands of detections and observe performance issues.


---

## 9. File/Module Naming

- This plan is saved as `plan1.md`.
- Recommended Rust modules:
  - Detector: `publisher.rs`, `payload.rs`
  - DB Processor: `subscriber.rs`, `db_writer.rs`, `geometry.rs`

---