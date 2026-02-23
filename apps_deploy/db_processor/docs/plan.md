# SAMDEF DB Processor & Detector Data Flow Plan

## 1. Overview & System Architecture

This plan describes the process of transmitting object detection results (bounding box coordinates and metadata) from the Detector to the DB Processor using the Zenoh crate in peer mode. The goal is to ensure efficient, reliable, and schema-compliant data transfer for geospatial storage and later visualization.

**Deployment:** Single Laptop (Localhost)  
**Topology:** Peer-to-Peer (Local Inter-Process Communication)  
**Transport Layer:** Eclipse Zenoh (Peer Mode)  
> No routers, brokers, or central servers are required. The two applications automatically discover each other on the local loopback interface.

**Components:**
- **Detector (Producer):** Runs inference, generates detection results, writes them to JSON (existing workflow), and also publishes them via Zenoh (new workflow).
- **DB Processor (Consumer):** Listens for detection messages, processes them, and writes to the PostGIS database.
- **Storage:** PostGIS (Docker Container)
- **Transport:** Zenoh (peer mode, no broker/server)

```mermaid
graph LR
     A[Detector (Producer)] -- Zenoh --> B[DB Processor (Consumer)]
     B --> C[PostGIS Database]
```

---

## 2. Infrastructure Specification (Database)

**Service:** PostgreSQL 17 or 18 (latest) with PostGIS 3.3+

**Deployment:** Docker Container

**Persistence:** Docker Named Volume

> Ensure the volume is created (`docker volume create samdef_pgdata`) so data survives container restarts.

**Configuration:**
- Port: 5432 mapped to host
- Env Variables: Standard POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_DB

**Schema Specification:**

**Setup Commands:**

1. **Create the Docker volume for persistent storage:**
    ```bash
    docker volume create samdef_pgdata
    ```

2. **Start the PostGIS container (latest version):**
    ```bash
    docker run -d \
        --name samdef_postgis \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=password \
        -e POSTGRES_DB=samdef \
        -p 5432:5432 \
        -v samdef_pgdata:/var/lib/postgresql/data \
        postgis/postgis:latest
    ```

3. **Create the detections table and indexes after connecting to the database:**
    ```sql
    CREATE TABLE IF NOT EXISTS detections (
        id BIGSERIAL PRIMARY KEY,
        source_file TEXT,
        class_id INTEGER,
        confidence REAL,
        geom_point geometry(Point, 4326),
        geom_bbox geometry(Polygon, 4326),
        created_at TIMESTAMP DEFAULT NOW()
    );
    CREATE INDEX IF NOT EXISTS detections_geom_point_gist ON detections USING GIST (geom_point);
    CREATE INDEX IF NOT EXISTS detections_geom_bbox_gist ON detections USING GIST (geom_bbox);
    ```

Table: detections

| Column       | Type                      | Notes                                      |
|--------------|---------------------------|--------------------------------------------|
| id           | BigInt/Serial             | Primary Key                                |
| source_file  | Text                      | Indexed                                    |
| class_id     | Integer                   |                                            |
| confidence   | Float (Real)              |                                            |
| geom_point   | Geometry(Point,4326)      | Center of bounding box (SRID 4326)         |
| geom_bbox    | Geometry(Polygon,4326)    | Full bounding box as polygon (SRID 4326)   |
| created_at   | Timestamp                 | Default Now                                |

Indexes:  
- GIST index on `geom_point` for fast point queries  
- GIST index on `geom_bbox` for fast polygon queries

> **Planned Extensions:**  
> Additional tables (e.g., `detection_summary`, `image_metadata`) will be added to store per-image statistics and metadata.

## 3. Data Protocol Specification

Both the Detector and DB Processor must strictly adhere to this binary protocol.

- **Serialization:** Use `bincode` for compact, fast binary serialization.
- **Why:** Smaller and faster than JSON, reducing serialization overhead on the CPU.
- **Topic Naming:** `satellite/detections/{source_file}` (e.g., `satellite/detections/Map_Region_04.tif`)
- **Payload Structure:**
  - **Header:**
    - `source_image` (String): TIFF/image filename
    - `geo_transform` ([f64; 6]): GDAL affine transform for pixel-to-geo conversion
    - `source_width` (usize)
    - `source_height` (usize)
  - **Body** (Vec of detections):
    - `bbox`: `{ x_min, y_min, x_max, y_max }` (pixel coordinates, f32)
    - `class_id` (usize)
    - `confidence` (f32)

## 4. Detector (Producer) Flow

- **Role:** Run inference and broadcast results
- **Runtime:** Rust Binary (`apps_deploy/detector`)
- **Key Libraries:** zenoh, bincode, serde

**Operational Steps:**
1. Run inference on the image and collect detections.
2. For each detection, record bounding box, class_id, and confidence.
3. After processing a TIFF:
    - Write detections to JSON (existing logic, retained for debugging and compatibility).
    - Build a payload struct with header and detection list (new logic).
    - Serialize the payload using `bincode` (new logic).
    - Publish the payload to Zenoh on the topic `satellite/detections/{source_file}` (new logic).
4. Do not write to DB or disk in any other format (no CSV, no direct DB connection).

## 5. DB Processor (Consumer) Flow

- **Role:** Headless Database Ingestor
- **Runtime:** Rust Binary (`apps_deploy/post_processor`)
- **Key Libraries:** tokio, sqlx (Postgres + PostGIS), zenoh, bincode

**Operational Steps:**
1. Initialize Zenoh in peer mode and subscribe to `satellite/detections/**`.
2. On message arrival:
    - Deserialize the payload using `bincode`.
    - For each detection:
        - Convert bbox pixel coordinates to:
            - Center point (for `geom_point`)
            - Polygon (for `geom_bbox`)
          using the provided `geo_transform`.
    - Insert all detections into the `detections` table (bulk insert, transactional).
3. Log the operation (e.g., number of detections, source file, time taken).
4. No visualization or image modification at this stage.

**Database Connection:**
- Establish an async connection pool (PgPool) to the local PostGIS container (localhost:5432)
- Startup Check: Verify the detections table exists and has the correct SRID (4326)

**Bulk Insert SQL Pattern:**
```sql
INSERT INTO detections (source_file, class_id, confidence, geom_point, geom_bbox)
SELECT $1, unnest($2::int[]), unnest($3::float4[]),
       unnest($4::geometry[]), unnest($5::geometry[])
```

**Output:**
- Console Logs Only: (e.g., `[INFO] Inserted 452 objects from 'Map_Tile_04.tif' in 12ms`)
- Zero Visualization: No images are opened, read, or modified

**Integration Note for Future Iced UI:**  
By treating the database as the "Source of Truth," you set up the future UI perfectly:
- Now: The DB Processor blindly dumps data into PostGIS
- Later: The Iced UI will run: `SELECT * FROM detections WHERE source_file = 'Map_Tile_04.tif'` to render overlays on demand, removing the need to ever draw permanent boxes on the raw images

---

## 6. Error Handling & Robustness

- **Zenoh:** Handle reconnections and message loss gracefully.
- **Deserialization:** Validate payload structure and log errors.
- **Database:** Use transactions for atomicity; log and handle DB errors.

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

- This plan is saved as `plan.md`.
- Recommended Rust modules:
  - Detector: `publisher.rs`, `payload.rs`
  - DB Processor: `subscriber.rs`, `db_writer.rs`, `geometry.rs`

---

## 10. Execution Plan

1. **Docker Start:**
    - Run the PostGIS container command (with volume)
2. **Consumer Launch:**
    - Start the post_processor binary first. It will sit idle, listening for Zenoh messages
3. **Producer Launch:**
    - Start the detector binary. It will begin processing images and silently "beaming" data to the post_processor
4. **Verification:**
    - Use DBeaver or pgAdmin to view the detections table and confirm data is arriving and mapping correctly

