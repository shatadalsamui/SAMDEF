SAMDEF/apps_deploy/gui_iced/docs/plan.md
# SAMDEF Iced UI Microservice: Step-by-Step Master Plan

This document outlines the complete, step-by-step plan for building the SAMDEF Interactive GIS Viewer microservice using the [iced](https://github.com/iced-rs/iced) GUI framework in Rust. The architecture follows a decoupled microservice model, enabling real-time visualization of detection results.

---

## Context

- **Detector**: Runs inference and saves raw bounding boxes to a PostgreSQL DB via the DB Processor.
- **DB Processor (Reader)**: Queries the DB, serializes bounding boxes and image metadata using `bincode`, and publishes to a Zenoh topic (`samdef/ui/display`).
- **Iced UI**: Listens to the Zenoh topic, deserializes the payload, loads the corresponding local TIFF image, and renders it onto an interactive canvas with zoom, pan, and vector-drawn bounding boxes.

---

## Phase 1: Shared Data Structures & Dependencies

**Goal:** Define a shared data structure for communication between the DB Processor and the Iced UI.

- **Dependencies (UI `Cargo.toml`):**
  - `iced` (with `canvas`, `tokio`, `image` features)
  - `zenoh`
  - `bincode`
  - `serde`
  - `tokio`

- **Payload Struct:**
  - Define a Rust struct `UiDisplayPayload` with:
    - `image_path: String` (absolute path to the local TIFF image)
    - `width: f32` (image width)
    - `height: f32` (image height)
    - `detections: Vec<BoundingBox>`
  - `BoundingBox` struct:
    - `x_min, y_min, x_max, y_max: f32`
    - `confidence: f32`
    - `class_id: usize`
  - Both structs must derive `Serialize`, `Deserialize`, `Clone`, and `Debug`.

---

## Phase 2: The DB Processor Exporter (Zenoh Publisher)

**Goal:** Publish detection results to Zenoh when an image finishes processing.

- Write an async function in the DB Processor to:
  1. Query the `detections_pixels` PostgreSQL table for a specific `image_id` to get all bounding boxes.
  2. Construct the `UiDisplayPayload` struct with the query results.
  3. Serialize the struct using `bincode::serialize`.
  4. Publish the byte array to the Zenoh topic `samdef/ui/display` using the existing Zenoh session.

---

## Phase 3: The Iced UI App Shell & State

**Goal:** Set up the main Iced Application structure and state management.

- **Main App Struct (`SamdefViewer`):**
  - State:
    - `show_detections: bool` (default: true)
    - `current_image_path: Option<String>`
    - `detections: Vec<BoundingBox>`
    - `canvas_state`: Custom struct for pan offsets (`x`, `y`) and zoom scale (default: 1.0)
  - **Messages (`Message` enum):**
    - `ToggleDetections(bool)`
    - `ZenohPayloadReceived(UiDisplayPayload)`
    - `CanvasEvent(CanvasMessage)` (for mouse wheel/drag events)
  - **Layout (view):**
    - Use a `Row`:
      - **Left (20% width):** `Column` with a `Checkbox` for `show_detections` and telemetry text (e.g., "Detections: X")
      - **Right (80% width):** The `Canvas` widget for rendering

---

## Phase 4: The Zenoh Background Subscription

**Goal:** Receive Zenoh messages without blocking the UI thread.

- Use Iced's `Subscription` feature:
  - Spawn an async Tokio task.
  - Initialize a Zenoh session and subscribe to `samdef/ui/display`.
  - As messages arrive:
    - Deserialize bytes using `bincode::deserialize::<UiDisplayPayload>(&bytes)`.
    - Yield the payload back to the main Iced app as `Message::ZenohPayloadReceived`.

---

## Phase 5: The Interactive Canvas (Rendering & Math)

**Goal:** Implement the core interactive GIS viewer.

- Implement `iced::widget::canvas::Program` for `canvas_state`.
  - **draw method:**
    - Create a `Frame`.
    - Apply a global matrix transform based on current pan (`x`, `y`) and zoom scale.
    - **Draw Image:** If an image path is loaded, draw the raster image at (0, 0) with the parsed width and height.
      - Use `image::Handle` for caching (avoid reloading from disk every frame).
    - **Draw Vectors:** If `show_detections` is true, iterate over `detections`:
      - For each, construct a `Path::rectangle(x_min, y_min, width, height)`.
      - Stroke with a red, semi-transparent color (vector drawing ensures sharpness at any zoom).
  - **update method (mouse handling):**
    - Track `CursorMoved` and `MousePressed/MouseReleased` to calculate drag deltas and update pan.
    - Track `MouseWheel` events to adjust zoom scale (clamp between 0.1 and 10.0).
    - Ensure zooming is centered on the mouse cursor using affine transform math.

---

## How to Use This Plan

- **Step-by-step:** Work through each phase in order.
- **For AI code generation:** Give this plan to Copilot, Cursor, Claude, or similar. Ask for code for Phases 1 & 2 first, then proceed to Phases 3 & 4, and finally Phase 5.
- **Why:** This incremental approach prevents overwhelming code generation and makes debugging manageable.

---

**End of Plan**