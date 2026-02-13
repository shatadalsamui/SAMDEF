
# SAMDEF DB Processor Plan

## 1. System Architecture Overview

**Deployment:** Single Laptop (Localhost)

**Topology:** Peer-to-Peer (Local Inter-Process Communication)

**Transport Layer:** Eclipse Zenoh (Peer Mode)

> No routers, brokers, or central servers are required. The two applications automatically discover each other on the local loopback interface.

**Components:**
- Producer (Detector): The inference engine
- Consumer (Post Processor): The database writer
- Storage: PostGIS (Docker Container)

```mermaid
graph LR
     A[Detector (Producer)] -- Zenoh --> B[DB Processor (Consumer)]
     B --> C[PostGIS Database]
```


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

3. **Create the detections table and index after connecting to the database:**
    ```sql
    CREATE TABLE IF NOT EXISTS detections (
         id BIGSERIAL PRIMARY KEY,
         source_file TEXT,
         class_id INTEGER,
         confidence REAL,
         geom geometry(Point, 4326),
         created_at TIMESTAMP DEFAULT NOW()
    );
    CREATE INDEX IF NOT EXISTS detections_geom_gist ON detections USING GIST (geom);
    ```

Table: detections

| Column       | Type                | Notes                        |
|--------------|---------------------|------------------------------|
| id           | BigInt/Serial       | Primary Key                  |
| source_file  | Text                | Indexed                      |
| class_id     | Integer             |                              |
| confidence   | Float (Real)        |                              |
| geom         | Geometry(Point,4326)| Must use SRID 4326 (WGS84)   |
| created_at   | Timestamp           | Default Now                  |

Index: GIST index on the geom column for geospatial performance.


## 3. Data Protocol Specification (The "Wire" Contract)

> Since there is no shared workspace library, both the Detector and Post Processor must strictly adhere to this binary protocol.

**Serialization Format:** Bincode

**Why:** Smaller and faster than JSON, reducing serialization overhead on the CPU.

**Topic Naming Strategy:** `satellite/detections/{filename}`

Example: `satellite/detections/Map_Region_04.tif`


**Payload Structure (DetectionPayload):**

Header:
- `source_image` (String): The original filename
- `geo_transform` ([f64; 6]): The 6-coefficient GDAL matrix used to convert pixels to coordinates
- `source_width` (usize): Width of the source image
- `source_height` (usize): Height of the source image

Body (Vector of Detections):
- `bbox` (BoundingBox):
    - `x_min` (f32): Minimum X pixel
    - `y_min` (f32): Minimum Y pixel
    - `x_max` (f32): Maximum X pixel
    - `y_max` (f32): Maximum Y pixel
- `class_id` (usize): Object type
- `confidence` (f32): Detection score


## 4. Service Spec: The Detector (Producer)

**Role:** Run inference and broadcast results

**Runtime:** Rust Binary (`apps_deploy/detector`)

**Key Libraries:** zenoh, bincode, serde

**Operational Flow:**

Initialization:
- Open Zenoh session in Peer mode on startup

Processing:
- Perform Virtual Tiling → Inference → Global NMS (Existing logic)

Publishing Trigger:
- Occurs immediately after the Global NMS pass for a full TIFF file is finished

Transformation:
- Convert internal Detection structs into the CompactDetection format defined in the protocol

Broadcast:
- Serialize the payload to bytes
- Perform a Zenoh put operation to the specific topic `satellite/detections/{source_filename}`

Cleanup:
- Strict Rule: Do not write JSON or CSV files to disk. Do not connect to the database.



## 5. Service Spec: The DB Processor (Consumer)

**Role:** Headless Database Ingestor

**Runtime:** Rust Binary (`apps_deploy/post_processor`)

**Key Libraries:** tokio, sqlx (Postgres + PostGIS), zenoh, bincode

**Operational Flow:**

Database Connection:
- Establish an async connection pool (PgPool) to the local PostGIS container (localhost:5432)
- Startup Check: Verify the detections table exists and has the correct SRID (4326)

Network Listener (Zenoh Peer):
- Initialize Zenoh in Peer mode
- Declare a Subscriber on the wildcard topic: `satellite/detections/**`
- Status: The service sits idle until a message arrives

Atomic Transaction Logic (Per TIFF):
- Trigger: Receives a DetectionPayload (containing all detections for one complete TIFF)
- Step A (Deserialize): Decode the Bincode bytes into the Rust struct
- Step B (Geospatial Projection):
    - Iterate through the list of detections
    - Convert Pixel Coordinates (x_min, y_min, x_max, y_max) → Geographic Coordinates (Lon, Lat) using the geo_transform matrix provided in the payload
    - Note: No NMS or filtering happens here; raw inference results are trusted
- Step C (Bulk Persistence):
    - Open a PostgreSQL Transaction
    - Execute a Single Query using UNNEST to insert all records for that TIFF at once
    - SQL Pattern:
      ```sql
      INSERT INTO detections (source_file, class_id, confidence, geom)
      SELECT $1, unnest($2::int[]), unnest($3::float4[]),
             ST_SetSRID(ST_MakePoint(unnest($4::float8[]), unnest($5::float8[])), 4326)
      ```
      Commit Transaction

Output:
- Console Logs Only: (e.g., `[INFO] Inserted 452 objects from 'Map_Tile_04.tif' in 12ms`)
- Zero Visualization: No images are opened, read, or modified

Integration Note for Future Iced UI:
By treating the database as the "Source of Truth," you set up the future UI perfectly:
- Now: The DB Processor blindly dumps data into PostGIS
- Later: The Iced UI will run: `SELECT * FROM detections WHERE source_file = 'Map_Tile_04.tif'` to render overlays on demand, removing the need to ever draw permanent boxes on the raw images


## 6. Execution Plan

1. **Docker Start:**
    - Run the PostGIS container command (with volume)
2. **Consumer Launch:**
    - Start the post_processor binary first. It will sit idle, listening for Zenoh messages
3. **Producer Launch:**
    - Start the detector binary. It will begin processing images and silently "beaming" data to the post_processor
4. **Verification:**
    - Use DBeaver or pgAdmin to view the detections table and confirm data is arriving and mapping correctly
