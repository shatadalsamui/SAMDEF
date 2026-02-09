# SAMDEF Detector Architecture

## Overview

The SAMDEF Detector is a high-performance Rust application designed for real-time object detection on large-scale geospatial imagery. It processes GeoTIFF files using virtual tiling, runs inference on a YOLOv26s ONNX model deployed on CUDA-enabled GPUs, and produces detection results in JSON format with global coordinates.

### High-Level Flow of the SAMDEF Detector

The SAMDEF Detector processes GeoTIFF images using a producer-consumer architecture for efficient, GPU-accelerated object detection:

1. **Input Reading (Producer Thread)**:
   - Scans directory for GeoTIFF files (.tif/.tiff).
   - Uses virtual tiling to generate 896x896 RGB tiles with 716px stride and shift-back strategy for edge handling.
   - Creates tasks with global offsets and geo-transform data.
   - Sends tasks via bounded channel (capacity: 64) to prevent memory overflow.

2. **Batch Processing (Consumer Thread)**:
   - Collects up to 32 tasks into a batch.
   - Preprocesses tiles: RGB pixels → normalized CHW tensors (parallel CPU processing).
   - Runs inference on batch using YOLOv26s ONNX model via CUDA.

3. **Postprocessing and Output**:
   - Applies confidence thresholds and NMS (IoU 0.45).
   - Converts local tile coordinates to global coordinates using offsets.
   - Aggregates detections by source TIFF, applies global NMS, serializes to JSON.

**Key Characteristics**:
- Virtual tiling eliminates need for pre-tiled JPEGs.
- Parallel preprocessing across CPU cores.
- GPU batching for efficiency.
- Handles large GeoTIFFs with backpressure via channel.
- Outputs detections in global coordinates with geo-transform metadata.

## High-Level Architecture

Producer-consumer pattern with virtual tiling and GPU-accelerated inference:

```
[Producer Thread] --> [Crossbeam Channel] --> [Consumer Thread]
       |                       |                       |
   Virtual Tile            Buffer Tasks            Batch Process
   GeoTIFFs                Capacity: 64            GPU Inference
   Send Tasks              Bounded Channel         Post-process
                                                       |
                                               [JSON Output]
```

## Core Components

### 1. Producer Thread (Main Thread)

**Location:** `modules/io/producer.rs`

**Responsibilities:**
- Scans input directory for GeoTIFF files.
- Calls `virtual_tiler::process_geotiff()` for each file.
- Sends `InferenceTask` structs to channel.

### 2. Virtual Tiler

**Location:** `modules/io/virtual_tiler.rs`

**Responsibilities:**
- Opens GeoTIFF with GDAL, reads RGB bands.
- Generates overlapping tiles (896x896, stride 716).
- Uses shift-back for edge tiles to maintain size.
- Creates interleaved RGB data for each tile.
- Builds tasks with offsets and geo-transform.

### 3. Consumer Thread (Inference Engine)

**Location:** `modules/io/consumer.rs`

**Responsibilities:**
- Maintains ONNX session with CUDA provider.
- Batches tasks (up to 32 images).
- Calls `process_batch()` for inference.
- Aggregates results by source path.
- Returns detections grouped by TIFF.

### 4. Preprocessing Module

**Location:** `modules/processing/pre_processing.rs`

**Operations:**
- Converts RGB bytes to normalized float tensors.
- Transposes HWC → CHW.
- Parallel processing with Rayon.

**Output:** Flattened float32 tensor `[batch, 3, 896, 896]`

### 5. Inference Module

**Location:** `modules/processing/inference.rs`

**Operations:**
- Executes ONNX session on batch tensor.
- Returns output tensor `[batch, 300, 6]`.

### 6. Postprocessing Module

**Location:** `modules/processing/post_processing.rs`

**Operations:**
- Parses model output to detections.
- Applies thresholds and NMS (IoU 0.45).
- Converts to global coordinates.

## Data Structures

### InferenceTask
- image_data: RGB bytes (interleaved)
- source_path: GeoTIFF file path
- global_offset_x/y: Pixel offsets in source
- geo_transform: GDAL geo-transform array

### Detection
- bbox: BoundingBox (global coords)
- class_id: 0-7 (vehicle/building types)
- confidence: 0.0-1.0

## Configuration Constants

- **TILE_SIZE:** 896
- **STRIDE:** 716
- **BATCH_SIZE:** 32
- **CHANNEL_CAPACITY:** 64
- **NMS_IOU_THRESHOLD:** 0.45
- **CLASS_THRESHOLDS:** Varies by class

## Data Flow

### 1. Input Processing
```
GeoTIFF → GDAL Read → Virtual Tiles → InferenceTask → Channel
```

### 2. Batch Formation
```
Channel → Collect up to 32 tasks → preprocess_batch()
```

### 3. Preprocessing
```
RGB Bytes → Float Tensor → CHW → Batch Tensor
```

### 4. Inference
```
Batch Tensor → ONNX Session → Raw Outputs [batch, 300, 6]
```

### 5. Postprocessing
```
Raw Outputs → Threshold → Global Offset → Detections
```

### 6. Aggregation
```
Detections → Group by TIFF → Global NMS → JSON Serialization
```

## Coordinate System

### Local Coordinates
- Relative to 896x896 tile
- Origin: top-left (0,0)

### Global Coordinates
- Absolute pixel position in source GeoTIFF
- Calculated: `local + offset`

## Output Format

### JSON Structure
- source_image: Path to GeoTIFF
- geo_transform: GDAL transform array
- source_width/height: Image dimensions
- detections: Array of detection objects

### File Naming
- `{tiff_stem}_results.json`

## Dependencies

- **ort:** ONNX Runtime (CUDA)
- **gdal:** GeoTIFF reading
- **ndarray:** Tensor ops
- **rayon:** Parallel processing
- **crossbeam:** Channel
- **serde:** JSON serialization

## Build and Deployment

- **Compilation:** `cargo build --release`
- **CUDA Dependency:** Requires CUDA runtime
- **Model Path:** Hardcoded to `best.onnx`
- **Input:** Directory with GeoTIFF files