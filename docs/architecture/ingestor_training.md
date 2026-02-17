# SAMDEF Ingestor Microservice Architecture & Data Flow

---

## 1. Overview: What the Ingestor Does

The SAMDEF Ingestor microservice is responsible for preparing training and validation datasets for satellite object detection. It processes large GeoTIFF images, tiles them into fixed-size patches, parses and maps labels from geojson files, generates YOLO-format label files, and organizes the output into train/val splits. This ensures the data is ready for high-fidelity model training and evaluation.

---

## 2. Technologies, Libraries, and Key Concepts

- **Rust**: The primary implementation language for safety and performance.
- **GDAL**: For reading and processing GeoTIFF satellite images.
- **TurboJPEG**: For fast JPEG compression of image tiles.
- **Rayon**: For parallel data processing (image tiling and label generation).
- **Serde**: For deserialization of geojson label files.
- **rand**: For shuffling and splitting image IDs into train/val sets.
- **Standard Library**: For file system operations, collections, and path handling.

---

## 3. High-Level Architecture Diagram (Textual)

```
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|                   |      |                   |      |                   |      |                   |
|   GeoTIFF Images  | ---> |   Image Tiler     | ---> |   Label Mapper    | ---> |   Output Folders  |
|                   |      |                   |      |                   |      |                   |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
        |                        |                           |                          |
        v                        v                           v                          v
[Raw Images]           [Tiles + Metadata]           [YOLO Labels]             [Train/Val Images & Labels]
```

**Key Data Flow:**
- Images and labels are loaded and split into train/val sets.
- Each image is tiled, and labels are mapped to tiles.
- YOLO-format label files are generated for each tile.
- Output is organized into train/val folders.

---

## 4. Architecture Components & Module Definitions

### 4.1. Main Entry (`main.rs`)
- Loads geojson label file and parses features.
- Extracts unique image IDs and shuffles them for random train/val split.
- Defines output directories for images and labels.
- Iterates over train and val splits, filtering images and labels for each.
- Tiles images and generates YOLO label files in parallel using Rayon.
- Writes a `data.yaml` file with class names for YOLO training.

### 4.2. Modules

#### `modules/image_utils.rs`
- **find_tif_images**: Scans a directory for `.tif` images.
- **tile_image**: Tiles a GeoTIFF image into fixed-size patches, compresses them to JPEG, and writes YOLO label files for each tile.

#### `modules/label_parser.rs`
- **GeoJson/Label/Properties structs**: Deserialize geojson label files.
- **load_labels**: Loads and parses geojson label file into label structs.
- **Label::parse_bounds**: Parses bounding box coordinates from label properties.

#### `modules/labeler.rs`
- **ParsedLabel struct**: Represents a mapped label with class and bounding box.
- **prepare_labels**: Maps raw labels to internal class IDs and filters by object type.
- **labels_for_tile**: Generates YOLO-format label content for a given tile.

#### `modules/mod.rs`
- Module declarations for image_utils, label_parser, and labeler.

---

## 5. Detailed Data Flow

### 5.1. Initialization

- The service loads the geojson label file and parses all label features.
- Unique image IDs are extracted and shuffled for randomization.
- A 90/10 split is performed to create train and validation sets.

### 5.2. Train/Val Split

- Image IDs are divided into train and val sets.
- Output directories for images and labels are created for each split.

### 5.3. Image Tiling

- For each image in the split:
  - The image is opened using GDAL.
  - The image is tiled into patches of fixed size (e.g., 896x896) with overlap (stride, e.g., 716 for 20% overlap).
  - Each tile is compressed to JPEG using TurboJPEG and saved to the output directory.

### 5.4. Label Mapping and YOLO Generation

- For each tile:
  - Labels are mapped to the tile using bounding box intersection.
  - Each label is converted to YOLO format: `<class_id> <x_center> <y_center> <width> <height>` (normalized to [0,1]).
  - YOLO label files are written alongside each image tile.

### 5.5. Parallel Processing

- Image tiling and label generation are performed in parallel using Rayon for high throughput.

### 5.6. Output Organization

- Images and labels are saved in structured folders:
  - `images/train`, `labels/train`
  - `images/val`, `labels/val`
- A `data.yaml` file is generated with class names and paths for YOLO training.

---

## 6. Component Roles and Responsibilities

- **Main Entry**: Coordinates the overall workflow, manages splits, and triggers processing.
- **Image Tiler**: Handles reading, tiling, and compression of images.
- **Label Mapper**: Maps and filters labels, converts to YOLO format.
- **Parallel Executor**: Uses Rayon to maximize CPU utilization.
- **Output Manager**: Organizes output folders and writes configuration files.

---

## 7. Data Flow Summary (Step-by-Step)

1. **Input**: GeoTIFF images and geojson label file are provided.
2. **Label Parsing**: Labels are loaded and parsed into internal structs.
3. **Image ID Extraction**: Unique image IDs are extracted and shuffled.
4. **Train/Val Split**: IDs are split into train and validation sets.
5. **Directory Setup**: Output directories are created for each split.
6. **Image Tiling**: Each image is tiled into patches, compressed, and saved.
7. **Label Mapping**: Labels are mapped to tiles and converted to YOLO format.
8. **Label Writing**: YOLO label files are written for each tile.
9. **Configuration**: `data.yaml` is generated for YOLO training.
10. **Completion**: All processing is logged and timed for verification.

---

## 8. Extending or Debugging the System

- **Adding New Classes**: Update class mapping in `labeler.rs` and `data.yaml`.
- **Changing Tile Size/Stride**: Modify constants in `main.rs` and `tile_image`.
- **Supporting New Label Formats**: Extend label parsing logic in `label_parser.rs`.
- **Debugging**: Use logging output, check image and label files, and verify splits.
- **Performance Tuning**: Adjust Rayon parallelism and TurboJPEG quality settings.

---

This documentation provides a complete, detailed, and accessible overview of the SAMDEF Ingestor microservice, its architecture, and its data flow. Any engineer should be able to understand, extend, or debug the system using this guide.