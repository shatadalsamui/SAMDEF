# SAMDEF GIS Viewer & Exploitation GUI (gui_iced) Architecture

## 1. Overview

The `gui_iced` application is a high-performance, native desktop graphical interface built in Rust using the `iced` framework. It serves as the primary visual exploitation tool within the SAMDEF (Satellite Object Detection & Exploitation Framework) platform.

The tool provides analyst-grade interactive inspection of ultra-high-resolution satellite imagery (GeoTIFF, typically 3195x3215 pixels or larger) combined with real-time vector bounding box overlays produced by deep learning object detection pipelines.

Key capabilities include:
- Smooth pan and zoom navigation centered on the cursor position with sub-pixel precision.
- Zero-allocation dynamic vector annotation rendering with zero raster overhead.
- Instantaneous class-level filtering across 8 target classes and continuous confidence thresholding.
- Direct file search and compact dropdown menu navigation across hundreds of inference datasets.
- Telemetry HUD displaying sensor dimensions, visible detection counts, magnification level, and image-space pixel coordinates.
- Dark theme styling adhering to VS Code and Zed dark editor color systems.

---

## 2. Core Architectural Principles

### 2.1. Strict Separation of Concerns (SRP) & Modularity
The codebase is structured into isolated, cohesive modules where each file maintains a focused responsibility. File length is strictly capped (no file exceeds 220 lines), preventing monolithic code bloat:
- `app`: State definitions, message routing, and update logic.
- `io`: Disk access, Planar TIFF decoding, and directory scanning.
- `model`: Detection domain entities, coordinate types, and color palettes.
- `ui`: Modular component views and custom styling sheets.
- `viewer`: Custom low-level Iced widget, event processing, coordinate transformation, and multi-layer rendering.

### 2.2. Zero-Allocation Vector Layer Architecture
Earlier architectures that generated transparent RGBA raster masks on the CPU and uploaded them to the GPU caused memory exhaustion and GLES driver crashes. The viewer implements a direct vector layer:
- The base satellite image is the sole raster texture uploaded to the GPU atlas.
- Bounding boxes are generated on-the-fly as solid vector quads inside `renderer.with_layer(bounds, ...)`.
- Box interiors have no geometry drawn, remaining completely transparent so the raw satellite pixels beneath are rendered with full fidelity.
- Changing class filters or confidence thresholds triggers zero memory allocations and executes in under 0.1 milliseconds.

### 2.3. Resilient Font & Glyph Rendering
To prevent missing-glyph placeholder boxes (empty rectangles) caused by missing Unicode font glyphs on Linux systems, the interface avoids non-standard Unicode symbols. Navigation controls use standard ASCII labels (`< Prev`, `Next >`, `Scan`, `Reset View`), and class indicators use vector-drawn container swatches rather than symbol characters.

---

## 3. High-Level System Architecture

```
+-----------------------------------------------------------------------------------+
|                                  SAMDEF VIEWER                                    |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +---------------------------+       +-----------------------------------------+  |
|  |       SIDEBAR (340px)     |       |          VIEWPORT CONTAINER             |  |
|  |---------------------------|       |-----------------------------------------|  |
|  | Title & Header            |       | Top Toolbar                             |  |
|  | Directory Path & Scan     |       |   - View Reset                          |  |
|  | Search Input (Jump)       |       |   - Status Indicator                    |  |
|  | Dropdown File Picker      |       |   - System Version                      |  |
|  | Prev / Next Stepper       |       |-----------------------------------------|  |
|  | Annotation Batch Controls |       | AnnotatedViewer (Custom Iced Widget)   |  |
|  | 8 Target Class Toggles    |       |   Layer 1: Base GeoTIFF Raster Image    |  |
|  | Confidence Slider         |       |   Layer 2: Dynamic Vector BBoxes        |  |
|  | Telemetry HUD             |       |   Layer 3: Target Class & Conf Badges   |  |
|  +---------------------------+       +-----------------------------------------+  |
|                                                                                   |
+-----------------------------------------------------------------------------------+
```

### Component Data Flow

```
[User Action: Search / Select / Step]
                 |
                 v
        [Message::SelectJsonFile]
                 |
                 v
       [Command::perform] (Async Tokio)
                 |
        +--------+--------+
        |                 |
        v                 v
[Read JSON File]    [Resolve TIFF Path]
        |                 |
        v                 v
[Parse FinalOutput] [Planar TIFF Loader]
        |                 |
        +--------+--------+
                 |
                 v
      [State Updated in app]
                 |
                 v
     [AnnotatedViewer::draw]
                 |
     +-----------+-----------+
     |                       |
     v                       v
[Draw Base Texture]    [Draw Vector Quads]
```

---

## 4. Directory & Module Structure

```
apps_deploy/gui_iced/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Application entry point and runner
│   ├── app/
│   │   ├── mod.rs              # Iced Application trait implementation
│   │   ├── message.rs          # System message taxonomy
│   │   ├── state.rs            # SamdefViewer state struct & class histogram logic
│   │   └── update.rs           # Pure update handler and async task commands
│   ├── io/
│   │   ├── mod.rs              # IO module exports
│   │   ├── file_scanner.rs     # Directory traversal and path resolution
│   │   └── tiff_loader.rs      # High-performance Planar Config 2 TIFF decoder
│   ├── model/
│   │   ├── mod.rs              # Model module exports
│   │   ├── detection.rs        # Bounding box and detection entity definitions
│   │   └── palette.rs          # Neon color palette and class taxonomy
│   ├── ui/
│   │   ├── mod.rs              # UI layout assembly (sidebar + canvas)
│   │   ├── class_filters.rs    # Class toggle list and confidence slider
│   │   ├── file_nav.rs         # Directory scanner, search, and pick_list dropdown
│   │   ├── quick_actions.rs    # Batch toggles (All On / All Off) and badge switch
│   │   ├── sidebar.rs          # Sidebar composite layout
│   │   ├── style.rs            # VS Code / Zed Dark style sheets and color tokens
│   │   ├── telemetry.rs        # Resolution, coordinates, and detection stats HUD
│   │   └── toolbar.rs          # Top status bar and reset controls
│   └── viewer/
│       ├── mod.rs              # Viewer module exports
│       ├── events.rs           # Mouse pan, cursor tracking, and affine zoom math
│       ├── render.rs           # Multi-layer rendering (raster + hollow quads)
│       ├── state.rs            # ViewerState, aspect ratio, and coordinate projection
│       └── widget.rs           # AnnotatedViewer custom Iced Widget implementation
```

---

## 5. Detailed Module Breakdown

### 5.1. Application Core (`src/app/`)

#### `state.rs`
Defines `SamdefViewer`, the central state structure:
- Image state: `current_image: Option<LoadedImage>`, `current_json_path: Option<PathBuf>`, and `detections: Vec<DisplayDetection>`.
- File browsing state: `results_dir_input: String`, `available_json_files: Vec<PathBuf>`, `current_file_idx: Option<usize>`, and `file_filter_query: String`.
- Filter settings: `class_visible: [bool; 8]`, `confidence_threshold: f32`, and `show_labels: bool`.
- Telemetry: `current_zoom: f32`, `cursor_coord: Option<(f32, f32)>`, and `reset_counter: usize`.
- Helper methods:
  - `class_counts(&self) -> [usize; 8]`: Calculates per-class detection counts matching the current confidence threshold in a single iteration.
  - `count_visible_detections(&self) -> usize`: Returns the total number of detections that satisfy both class visibility and confidence filters.

#### `message.rs`
Enumerates all events and actions within the application:
- Directory & navigation: `ResultsDirChanged`, `ScanResultsDir`, `FileFilterChanged`, `FileFilterSubmitted`, `SelectJsonFile`, `NextFile`, `PrevFile`.
- Asynchronous load completion: `JsonLoaded(Result<(PathBuf, FinalOutput), String>)`, `ImageLoaded(Result<LoadedImage, String>)`.
- Annotation filters: `ToggleClass(usize, bool)`, `SelectAllClasses`, `TurnOffAllClasses`, `ConfidenceChanged(f32)`, `ToggleLabels(bool)`.
- Canvas interactions: `ViewerStatus { zoom, cursor }`, `ResetView`.

#### `update.rs`
Contains `handle_update(&mut SamdefViewer, Message) -> Command<Message>`:
- Dispatches messages to mutate state.
- Spawns asynchronous tasks via `Command::perform` for non-blocking file loading, keeping the UI responsive.
- Implements wraparound indexing for sequential navigation (`NextFile`, `PrevFile`).
- Filters files on query submission and immediately loads the best match.

---

### 5.2. I/O & Image Processing (`src/io/`)

#### `tiff_loader.rs`
Satellite imagery often uses Planar Configuration 2 (separate color planes: RRR... GGG... BBB...) rather than interleaved RGBRGB... standard formats. Standard image decoders often produce distorted or grayscale outputs when loading these files.
- `load_tiff_or_image`: Opens the file, verifies existence, and routes to `load_tiff_file`.
- `load_tiff_file`:
  1. Inspects the TIFF header tags (`Tag::PlanarConfiguration`, `Tag::StripOffsets`, `Tag::TileOffsets`).
  2. For Planar Configuration 2: reads separate strips for red, green, and blue planes, interweaves them into standard packed RGBA8, and assigns 100% alpha (255).
  3. For Interleaved RGB / Grayscale: chunks data into standard 4-byte RGBA.
  4. Wraps buffer into `iced::widget::image::Handle::from_pixels(width, height, rgba)`.
  5. Includes automatic fallback to `image::open` if specialized decoding fails.

#### `file_scanner.rs`
- `scan_json_directory`: Reads directory entries, filters for `.json` extensions, and returns sorted `Vec<PathBuf>`.
- `resolve_tiff_path`: Resolves the absolute path of the satellite image corresponding to an inference JSON file:
  1. Primary: Reads `source_image` directly from the JSON. If the path exists, it is used immediately.
  2. Override: Checks environment variables `INPUT_DIR` or `VAL_IMAGES_DIR`.
  3. Fallback: Checks `/home/shatadal/SAMDEF_DATA/val_images/{stem}.tif`.

---

### 5.3. Domain Model (`src/model/`)

#### `detection.rs`
Defines serialization structures mapping to the detector output schema:
- `BBoxCoords`: `x_min`, `y_min`, `x_max`, `y_max` (float coordinates in pixel space).
- `Detection`: `bbox: BBoxCoords`, `class_id: usize`, `confidence: f32`.
- `FinalOutput`: Top-level JSON schema with `source_image`, `geo_transform`, `source_width`, `source_height`, `detections`.
- `DisplayDetection`: Internal lightweight representation optimized for rendering loops.

#### `palette.rs`
Defines the visual taxonomy for the 8 target classes:
- Index 0: Long Truck (Neon Pink `[255, 0, 128]`)
- Index 1: Boxy Truck (Neon Green `[57, 255, 20]`)
- Index 2: Small Vehicle / Car (Neon Cyan `[0, 255, 255]`)
- Index 3: Building (Neon Yellow `[255, 255, 0]`)
- Index 4: Container (Neon Magenta `[255, 0, 255]`)
- Index 5: Construction (Neon Aqua `[0, 255, 128]`)
- Index 6: Tank (Neon Orange `[255, 110, 0]`)
- Index 7: Container Lot (Neon Blue-Green `[0, 255, 200]`)

---

### 5.4. User Interface Components (`src/ui/`)

#### `style.rs`
Implements the VS Code / Zed Dark aesthetic:
- Color constants:
  - Text Primary: `#d4d4d4`
  - Text Secondary / Muted: `#8c8c94`
  - Text Header: `#c7c7d1`
  - Accent Blue: `#4094e6`
  - Accent Green: `#4ec9b0`
  - Accent Amber: `#d9ad26`
  - Subtle Border: `#333338`
- Surface styling:
  - Sidebar: `#252527`
  - Top Toolbar: `#1f1f21`
  - Canvas Background: `#1a1a1c`
  - Card Containers: `#2e2e33`
  - Inputs & Dropdown: `#38383d`
- Custom widget style sheets: `GhostButton`, `GhostButtonAccent`, `VscodeInput`, `VscodeCheckbox`, `VscodeSlider`, `ColorSwatch`.

#### `file_nav.rs`
Implements the hybrid file selection interface:
- Text input for instant search queries (`Jump to file (e.g. 1062)...`).
- Compact, closed-by-default `pick_list` dropdown menu. Takes only one row when closed; expands to display all or filtered results when activated.
- Sequential `< Prev` and `Next >` stepping buttons.
- Status text showing current file index and matching file count.

#### `class_filters.rs`
- 8 class rows containing:
  - Dark-styled checkbox.
  - Vector container color swatch (10x10 px) matching class neon color.
  - Class name label.
  - Current detection count matching the confidence threshold.
- Continuous confidence threshold slider (0.0 to 1.0) with real-time percentage readout.

#### `quick_actions.rs`
- Batch action buttons: `Select All` and `Clear All` for fast visibility toggling.
- Checkbox toggle for class name and confidence percentage badges.

#### `telemetry.rs`
Dark HUD card displaying real-time operational parameters:
- Sensor Resolution: Image width and height in pixels.
- Detections: Visible count versus total count.
- Zoom Level: Current optical magnification percentage.
- Cursor Coordinates: Real-time image-space pixel coordinates (X and Y), or "Outside Image" when cursor leaves the image bounds.

#### `toolbar.rs`
Top toolbar providing:
- `Reset View` button: Centers the image and resets zoom scale to 1.0.
- Real-time status indicator: Shows loading spinner text or dataset details.
- System version indicator.

---

### 5.5. Custom Viewer & Rendering Engine (`src/viewer/`)

#### `widget.rs`
Implements the `iced::advanced::Widget` trait in `AnnotatedViewer`:
- Maintains internal tree state (`ViewerState`) preserved across render frames.
- Coordinates reset signals between external state updates and internal view offsets.
- Dispatches layout calculation, mouse interaction states (`Grab`, `Grabbing`), and render passes.

#### `events.rs`
Handles mouse wheel and mouse drag operations:
- Continuous Zoom: Calculates scale factor adjustment centered on the global mouse cursor position:
  $$\text{adjustment} = (\text{cursor} - \text{bounds.center}) \times \text{factor} + \text{offset} \times \text{factor}$$
- Clamped Pan: Tracks drag deltas and clamps offsets to ensure the image cannot be lost outside the viewport boundaries.
- Cursor Projection: Maps screen coordinates back to raw image pixel coordinates:
  $$P_x = \frac{\text{cursor}_x - \text{top\_left}_x}{\text{displayed\_width}} \times \text{raw\_width}$$
  $$P_y = \frac{\text{cursor}_y - \text{top\_left}_y}{\text{displayed\_height}} \times \text{raw\_height}$$

#### `render.rs`
Executes multi-layer rendering:
- **Layer 1 (Satellite Raster)**:
  - Calls `renderer.with_layer(bounds, ...)` and invokes `image::Renderer::draw` for the base satellite texture.
  - Rendered at the current scaled dimensions and pan offset.
- **Layer 2 (Dynamic Vector Bounding Boxes)**:
  - Bounding boxes are filtered by class visibility, confidence threshold, and viewport frustum clipping.
  - Drawn using `draw_hollow_box`: 4 thin solid edge rectangles (top, bottom, left, right).
  - Box interiors have no geometry drawn, remaining 100% transparent.
  - Line thickness adapts dynamically with ultra-fine hairline strokes (0.40px for small vehicles, 0.50px in standard non-zoomed view, up to 1.25px when magnified).
- **Layer 3 (Confidence & Class Badges)**:
  - Rendered when zoomed in (`scale >= 1.5 || scale_x >= 0.35`) and `show_labels` is enabled.
  - Draws a solid background tag with class name and confidence percentage.

---

## 6. Rendering & Memory Optimization Analysis

### 6.1. The WGPU Texture Atlas & Driver Panic Prevention

In `iced_wgpu`, textures larger than 2048x2048 pixels are automatically sliced across multiple layers in a GPU texture array. For a $3195 \times 3215$ satellite TIFF:
- Sliced into a $2 \times 2$ grid across **4 texture layers**.
- Linux OpenGL/GLES drivers (`wgpu-hal-0.16.2/src/gles/queue.rs`) contain an assertion where texture arrays with $\ge 6$ layers trigger cubemap blitting logic:
  ```rust
  let dst_is_cubemap = dst.texture_type == glow::TEXTURE_CUBE_MAP || layers >= 6;
  ```
  Since cubemaps only have 6 faces (indices 0..5), any allocation pushing total layers to 6 or more caused:
  `index out of bounds: the len is 6 but the index is 6`.

By utilizing dynamic vector quads instead of a second full-size transparent raster image, the total GPU texture layers are fixed at exactly 4, permanently avoiding this driver panic.

### 6.2. Performance Comparison

| Metric | Previous Raster Overlay Approach | Current Dynamic Vector Layer |
|---|---|---|
| Filter Toggle Latency | 150 ms - 450 ms (CPU raster + GPU upload) | < 0.1 ms (instantaneous) |
| Heap Allocation on Filter Click | ~41 MB per toggle | 0 bytes |
| GPU Texture Re-uploads | Required on every click/slider move | 0 re-uploads |
| Max Texture Layers | 8 layers (Triggered GLES crash) | 4 layers (Safe and stable) |
| Visual Clarity | Raster pixelation at high zoom | Infinite vector sharpness |

---

## 7. Data Contracts

### 7.1. Detector Output Schema (`*_results.json`)

```json
{
  "source_image": "/home/shatadal/SAMDEF_DATA/val_images/1062.tif",
  "geo_transform": [0.0, 1.0, 0.0, 0.0, 0.0, -1.0],
  "source_width": 3195,
  "source_height": 3215,
  "detections": [
    {
      "bbox": {
        "x_min": 1420.5,
        "y_min": 830.2,
        "x_max": 1480.0,
        "y_max": 895.4
      },
      "class_id": 0,
      "confidence": 0.892
    }
  ]
}
```

---

## 8. Build, Test, and Execution

### Compilation

```bash
# Debug build (fast compile for development)
cargo build

# Optimized release build
cargo build --release
```

### Execution

```bash
# Standard execution
cargo run --release

# Execution with custom results directory
RESULTS_DIR=/path/to/results cargo run --release

# Execution with custom images directory fallback
INPUT_DIR=/path/to/images cargo run --release
```

### Testing

```bash
# Run unit tests
cargo test

# Run linter checks
cargo clippy --all-targets
```
