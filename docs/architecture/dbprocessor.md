# SAMDEF DB Processor Microservice Architecture & Data Flow

---

## 1. Overview: What the DB Processor Does

The SAMDEF DB Processor microservice ingests detection results from the detector microservice (via Zenoh), deserializes the detection payloads, and persists them into a PostgreSQL database with PostGIS extensions. It ensures that all detected objects, their metadata, and geospatial information are efficiently stored and indexed for later querying, analysis, and visualization.

This service acts as the bridge between the real-time detection pipeline and the persistent, queryable storage layer.

---

## 2. Technologies, Libraries, and Key Concepts

- **Rust**: The primary implementation language for safety and performance.
- **Tokio**: For asynchronous operations and task management.
- **Zenoh**: For subscribing to detection payloads published by the detector microservice.
- **SQLx**: For asynchronous, compile-time checked PostgreSQL queries.
- **dotenv**: For environment variable management.
- **PostgreSQL + PostGIS**: For relational and geospatial data storage.
- **Serde/Bincode**: For (de)serialization of detection payloads.
- **GDAL Affine Transform**: For converting pixel coordinates to geospatial coordinates.

---

## 3. High-Level Architecture Diagram (Textual)

```
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|                   |      |                   |      |                   |      |                   |
|   Zenoh Network   | ---> |   Zenoh Subscriber| ---> |   DB Processor    | ---> |   PostgreSQL DB   |
| (Detection Pub)   |      |                   |      |                   |      |   (with PostGIS)  |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
        |                        |                           |                          |
        v                        v                           v                          v
[DetectionPayloads]      [Deserialization]         [Insert/Update Logic]         [Persistent Storage]
```

**Key Data Flow:**
- Detector publishes detection payloads to Zenoh.
- DB Processor subscribes, deserializes, and inserts data into the database.

---

## 4. Architecture Components & Module Definitions

### 4.1. Main Entry (`main.rs`)
- Loads environment variables (e.g., database URL).
- Initializes the PostgreSQL connection pool.
- Reads and executes the schema SQL file to ensure tables and indexes exist.
- Starts the Zenoh subscriber to listen for detection payloads.

### 4.2. Modules

#### `modules/writer`
- **zenoh_subscriber.rs**: Subscribes to Zenoh topics, deserializes detection payloads, and triggers database insertion.
- **queries.rs**: Contains logic for inserting detection data into the database, including geospatial conversions and bulk inserts.
- **payload.rs**: Defines the Rust structs for detection payloads, detections, and bounding boxes.

#### `modules/schema`
- **init.sql**: SQL schema for creating tables (`detections`, `detections_pixels`) and spatial indexes.

#### `modules/reader`
- (Currently empty; reserved for future extensions such as database querying or analytics.)

---

## 5. Detailed Data Flow

### 5.1. Initialization

- The service loads environment variables (e.g., `DATABASE_URL`) using dotenv.
- Connects to the PostgreSQL database using SQLx and creates a connection pool.
- Reads the schema SQL file and executes each statement to ensure all tables and indexes are present.

### 5.2. Zenoh Subscription

- The Zenoh subscriber is initialized and subscribes to the topic pattern `satellite/detections/**`.
- It listens for incoming detection payloads published by the detector microservice.

### 5.3. Payload Deserialization

- Upon receiving a message, the payload is deserialized from binary (bincode) into a `DetectionPayload` struct.
- Each payload contains:
  - `source_image`: The name of the source image file.
  - `geo_transform`: Affine transform for pixel-to-geospatial conversion.
  - `source_width`, `source_height`: Dimensions of the source image.
  - `detections`: A list of detected objects, each with bounding box, class ID, and confidence.

### 5.4. Database Insertion

- For each detection payload:
  - **Geospatial Conversion**: Pixel coordinates are converted to longitude/latitude using the affine transform.
  - **Bulk Insert**: All detections are inserted into the `detections` table (with geospatial columns) and the `detections_pixels` table (with raw pixel coordinates).
  - **Spatial Indexing**: PostGIS spatial indexes are used for efficient geospatial queries.

### 5.5. Error Handling and Logging

- All SQL statements are executed with error handling and logging.
- Any deserialization or insertion errors are logged for debugging.

---

## 6. Database Schema

- **detections**: Stores each detection with geospatial point (center) and polygon (bounding box) in WGS84 coordinates, along with class, confidence, and source file.
- **detections_pixels**: Stores each detection with raw pixel coordinates, class, confidence, and source file.
- **Indexes**: Spatial indexes on geometry columns and standard indexes on class/source for fast queries.

---

## 7. Component Roles and Responsibilities

- **Zenoh Subscriber**: Listens for detection payloads and triggers downstream processing.
- **Deserializer**: Converts binary payloads into Rust structs for processing.
- **Database Writer**: Handles all logic for converting, batching, and inserting detection data into the database.
- **Schema Manager**: Ensures the database schema is up-to-date and ready for new data.

---

## 8. Data Flow Summary (Step-by-Step)

1. **Startup**: Load environment, connect to database, initialize schema.
2. **Subscribe**: Listen for detection payloads on Zenoh.
3. **Receive**: On message, deserialize payload into structured data.
4. **Convert**: For each detection, convert pixel coordinates to geospatial coordinates.
5. **Insert**: Bulk insert detections into both geospatial and pixel tables.
6. **Index**: Ensure all data is indexed for fast querying.
7. **Repeat**: Continue processing as new detection payloads arrive.

---

## 9. Extending or Debugging the System

- **Adding New Fields**: Update the schema and Rust structs, and adjust insertion logic.
- **Supporting New Payload Types**: Extend the Zenoh subscriber and deserialization logic.
- **Querying Data**: Implement new modules under `reader` for analytics or reporting.
- **Debugging**: Use logs for tracing errors in deserialization or database operations.

---

This documentation provides a complete, detailed, and accessible overview of the SAMDEF DB Processor microservice, its architecture, and its data flow. Any engineer should be able to understand, extend, or debug the system using this guide.