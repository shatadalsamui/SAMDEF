# SAMDEF Detector Architecture

## Overview

The SAMDEF Detector is a high-performance Rust application designed for real-time object detection on large-scale geospatial imagery. It processes tiled JPEG images using a YOLOv26s ONNX model deployed on CUDA-enabled GPUs, producing detection results in JSON format with global coordinates for the orginal .tif/.tiff file.

### High-Level Flow of the SAMDEF Detector

The SAMDEF Detector processes tiled JPEG images from geospatial data using a producer-consumer architecture for efficient, GPU-accelerated object detection. Here's the simplified flow:

1. **Input Reading (Producer Thread)**:
   - Scans a directory for JPEG tiles (896x896 pixels each).
   - Extracts global offsets from filenames (e.g., `x<offx>_y<offy>` for absolute positioning).
   - Creates tasks and sends them via a bounded channel (capacity: 50) to buffer and prevent memory overflow.

2. **Batch Processing (Consumer Thread)**:
   - Collects up to 18 tasks into a batch.
   - Preprocesses images in parallel: JPEG → RGB pixels → normalized CHW tensors (using Rayon for CPU parallelism).
   - Runs inference on the batch using a YOLOv26s ONNX model via CUDA (output: detections with bounding boxes, confidence, and class IDs).

3. **Postprocessing and Output**:
   - Applies confidence thresholds and NMS (IoU 0.45) to filter detections.
   - Converts local tile coordinates to global coordinates using offsets.
   - Aggregates detections by TIFF ID, applies global NMS, and serializes to JSON files (e.g., `{tiff_id}_manifest.json`).

**Key Characteristics**:
- Parallel preprocessing across CPU cores.
- GPU batching for efficiency.
- Handles large-scale imagery with backpressure via the channel.
- Outputs detections in global coordinates for mapping back to source data.

## High-Level Architecture

The system follows a producer-consumer pattern with parallel preprocessing and GPU-accelerated inference:

```
[Producer Thread] --> [Crossbeam Channel] --> [Consumer Thread]
       |                       |                       |
   Read Tiles              Buffer Tasks            Batch Process
   Extract Offsets         Capacity: 50            GPU Inference
   Send Tasks              Bounded Channel         Post-process
                                                       |
                                               [JSON Output]
```

## Core Components

### 1. Producer Thread (Main Thread)

**Location:** `main.rs` - Producer Loop

**Responsibilities:**
- Scans input directory for JPEG files
- Extracts global offsets from filenames using `calculate_offsets()`
- Creates `InferenceTask` structs
- Sends tasks to bounded channel (capacity 50)
- Handles graceful shutdown on consumer failure

**Filename Parsing:**
- Expected format: `<stem>_<row>_<col>_x<offx>_y<offy>.jpg`
- Extracts offsets directly from `x<offx>_y<offy>` tokens
- Falls back to stride-based calculation if offsets missing
- Supports configurable row/column swapping (`SWAP_RC`)

### 2. Consumer Thread (Inference Engine)

**Location:** `main.rs` - Spawned Thread

**Responsibilities:**
- Maintains ONNX session with CUDA execution provider
- Batches incoming tasks (up to 18 images)
- Orchestrates preprocessing, inference, and postprocessing
- Aggregates detections by TIFF ID
- Applies global NMS across all detections per TIFF
- Serializes results to JSON files

### 3. Preprocessing Module

**Location:** `modules/pre_processing.rs`

**Functions:**
- `preprocess_image()`: Single image processing
- `preprocess_batch()`: Parallel batch processing

**Operations:**
1. JPEG decompression using `turbojpeg` (RGB format)
2. Validation: Assert 896x896 dimensions
3. HWC (Height, Width, Channels) → CHW (Channels, Height, Width) transposition with normalization   - Channels: RGB color channels (3 total: Red, Green, Blue)4. Parallel processing across CPU cores using Rayon

**Output:** Flattened float32 tensor `[batch, 3, 896, 896]`

### 4. Inference Module

**Location:** `modules/inference.rs`

**Function:** `run_inference()`

**Operations:**
1. Convert ndarray to `ort::Value`
2. Execute ONNX session with CUDA provider
3. Extract output tensor (shape: `[batch, 300, 6]`)
4. Return as `ArrayD<f32>`

**Model Details:**
- Input: `[batch, 3, 896, 896]` float32
- Output: `[batch, 300, 6]` - [x_min, y_min, x_max, y_max, confidence, class_id]
- Dynamic batch size (1-18) supported

### 5. Postprocessing Module

**Location:** `modules/post_processing.rs`

**Functions:**
- `parse_output()`: Parse raw model output
- `non_maximum_suppression()`: Remove overlapping detections

**Operations:**
1. **Thresholding:** Apply class-specific confidence thresholds
2. **Coordinate Handling:**
   - Detect normalized vs pixel coordinates
   - Scale normalized coords to 896px
   - Clamp to tile bounds [0, 895]
   - Ensure minimum 1px extent
3. **NMS:** IoU threshold 0.45, per-class suppression

## Data Structures

### InferenceTask
- image_data: Raw JPEG bytes
- global_offset_x: X offset for global coords
- global_offset_y: Y offset for global coords
- tile_filename: Source filename

### Detection
- bbox: BoundingBox
- class_id: 0-5 (Small_Vehicle, Building, etc.)
- confidence: 0.0-1.0
- source_tile: Original tile filename

### BoundingBox
- x_min, y_min, x_max, y_max

## Configuration Constants

- **TILE_STRIDE:** 716.0 (pixels between tile origins, accounting for overlap)
- **SWAP_RC:** false (filename ordering: row_col vs col_row)
- **BATCH_SIZE:** Up to 18 images
- **CHANNEL_CAPACITY:** 50 tasks
- **NMS_IOU_THRESHOLD:** 0.45
- **CLASS_THRESHOLDS:** [0.05, 0.05, 0.05, 0.05, 0.25, 0.25] for classes 0-5

## Data Flow

### 1. Input Processing
```
JPEG File → calculate_offsets() → InferenceTask → Channel
```

### 2. Batch Formation
```
Channel → Collect up to 18 tasks → preprocess_batch()
```

### 3. Preprocessing
```
JPEG Bytes → RGB Pixels → HWC Float → CHW Tensor → Batch Tensor
```

### 4. Inference
```
Batch Tensor → ONNX Session → Raw Outputs [batch, 300, 6]
```

### 5. Postprocessing
```
Raw Outputs → Threshold → Scale/Clamp → Detections → Global Offset Addition
```

### 6. Aggregation
```
Detections → Group by TIFF ID → Global NMS → JSON Serialization
```

## Coordinate System

### Local Coordinates
- Relative to 896x896 tile
- Origin: top-left (0,0)
- Range: [0, 895] for pixel coordinates

### Global Coordinates
- Absolute position in source TIFF
- Calculated: `local_coord + global_offset`
- Enables mapping detections back to original imagery

### Offset Extraction
- Primary: Direct from filename `x<offx>_y<offy>`
- Fallback: `col * TILE_STRIDE, row * TILE_STRIDE`

## Output Format

### JSON Structure
Array of detection objects with bbox, class_id, confidence, source_tile

### File Naming
- `{tiff_id}_manifest.json`
- `tiff_id` extracted from tile filename prefix

## Performance Characteristics

- **GPU Utilization:** CUDA execution provider locks to dGPU
- **CPU Parallelism:** Rayon-based preprocessing across cores
- **Memory Efficiency:** Bounded channel prevents unbounded memory growth
- **Batch Processing:** Amortizes GPU kernel launch overhead
- **Backpressure:** Channel capacity limits producer speed to consumer capacity

## Error Handling

- **Producer:** Stops on channel send failure (consumer crashed)
- **Consumer:** Panics propagate to main thread via `handle.join()`
- **Preprocessing:** Validates image dimensions, returns `Result`
- **Inference:** ORT errors bubble up as `anyhow::Error`

## Dependencies

- **ort:** ONNX Runtime Rust bindings (CUDA support)
- **ndarray:** Tensor operations
- **turbojpeg:** Fast JPEG decompression
- **rayon:** Parallel processing
- **crossbeam:** Multi-producer single-consumer channel
- **serde:** JSON serialization

## Build and Deployment

- **Compilation:** `cargo build --release`
- **CUDA Dependency:** Requires CUDA runtime and ONNX Runtime GPU libraries
- **Model Path:** Hardcoded absolute path to `best.onnx`
- **Directory Structure:** Expects specific input/output paths

This architecture enables efficient processing of large geospatial datasets with minimal latency and high throughput through GPU acceleration and parallel preprocessing.