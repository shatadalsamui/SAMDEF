# SAMDEF Detector Microservice Architecture & Data Flow

## 1. Overview: What the Detector Does

The SAMDEF Detector microservice is a high-performance, batch-oriented inference engine designed to process large satellite images (GeoTIFFs) and detect objects using a deep learning model (ONNX format). It divides large images into tiles, runs inference on each tile using GPU acceleration, aggregates results, and publishes structured detection outputs for downstream systems.

This microservice is optimized for throughput and scalability, leveraging multi-threading, batching, and asynchronous operations to maximize GPU utilization and minimize latency.

---

## 2. Technologies, Libraries, and Key Concepts

- **Rust**: The primary implementation language, chosen for safety and performance.
- **ONNX Runtime (ort)**: For running deep learning inference on GPU (CUDA).
- **Tokio**: For asynchronous operations and task management.
- **Crossbeam**: For fast, thread-safe channels between producer and consumer threads.
- **Rayon**: For parallel data processing (e.g., preprocessing, NMS).
- **GDAL**: For reading and processing GeoTIFF satellite images.
- **Zenoh**: For publishing detection results to a distributed data fabric.
- **Serde**: For serialization (JSON, binary).
- **Bincode**: For efficient binary serialization of detection payloads.

---

## 3. High-Level Architecture Diagram (Textual)

```
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|                   |      |                   |      |                   |      |                   |
|   Producer Thread | ---> |   Channel (Queue) | ---> |  Consumer Thread  | ---> |   Output/Publish  |
|                   |      |                   |      |                   |      |                   |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
        |                        |                           |                          |
        |                        |                           |                          |
        v                        v                           v                          v
[Read GeoTIFFs]         [Batched PipelineMessages]   [Batch Inference, Aggregation]   [JSON, Zenoh]
```

**Key Data Flow:**
- Producer reads and tiles images, sending tasks to a channel.
- Consumer receives batches, runs inference, aggregates, and saves/publishes results.

---

## 4. Architecture Components & Module Definitions

### 4.1. Main Entry (`main.rs`)
- Initializes logging, ONNX runtime, and directories.
- Sets up the producer-consumer pipeline using threads and a bounded channel.
- Coordinates the overall pipeline execution and timing.

### 4.2. Modules

#### `modules/data`
- **task.rs**: Defines `InferenceTask` (tile data, offsets, geotransform) and `PipelineMessage` (task, end-of-file, terminate).
- **results.rs**: Handles aggregation and saving of detection results, including JSON serialization and Zenoh publishing.
- **payload.rs**: Defines the structure for detection payloads sent over Zenoh.

#### `modules/io`
- **producer.rs**: Scans input directory for GeoTIFFs, spawns threads to tile and send tasks to the channel.
- **consumer.rs**: Receives tasks, batches them, runs inference, aggregates results, and triggers output.
- **session.rs**: Initializes ONNX inference session with GPU support.
- **publisher.rs**: Publishes detection payloads to Zenoh.
- **virtual_tiler.rs**: Handles tiling of large GeoTIFFs into fixed-size tiles, extracting RGB data.

#### `modules/processing`
- **pre_processing.rs**: Converts raw image data into normalized tensors suitable for model input.
- **batch.rs**: Orchestrates batch inference, including preprocessing, inference, and postprocessing.
- **inference.rs**: Runs the ONNX model on input tensors.
- **post_processing.rs**: Parses model outputs, applies non-maximum suppression (NMS), and structures detections.

---

## 5. Detailed Data Flow

### 5.1. Initialization

- The service starts, initializes logging and the ONNX runtime, and prepares input/output directories.
- A bounded channel is created for passing `PipelineMessage` objects between producer and consumer.

### 5.2. Producer Thread

- Scans the input directory for GeoTIFF files.
- For each file, spawns up to 4 threads (configurable parallelism) to process files in parallel.
- Each thread:
  - Opens a GeoTIFF using GDAL.
  - Tiles the image into fixed-size (896x896) RGB tiles with 20 % overlap (stride).
  - For each tile, creates an `InferenceTask` containing the image data, offsets, and geotransform.
  - Each thread (handling a single file) sends its tiles sequentially, one at a time, as `PipelineMessage::Process(InferenceTask)` to the channel (not in batches).
  - After all tiles for that file are sent, that same thread immediately sends a `PipelineMessage::EndOfFile` for that file.
  - Multiple threads (each for a different file) may be active at once, so tiles from different files can be interleaved in the channel, but for any given file, its tiles and EndOfFile are always sent in order by the same thread.
- After all files are processed, the main producer sends a single `PipelineMessage::Terminate` to signal the end of all work.

### 5.3. Channel (Queue)

- A bounded, thread-safe channel (from Crossbeam) buffers messages between producer and consumer.
- The batch size is set to 32 for optimal GPU utilization.
- The channel buffer is set to double the batch size (64), allowing the producer to stay ahead and keep the GPU fed with data, smoothing out timing mismatches between producer and consumer.

### 5.4. Consumer Thread

- Receives `PipelineMessage` objects from the channel.
- Accumulates `InferenceTask`s into batches of 32.
- When a batch is full (or at termination), processes the batch:
  - **Preprocessing**: Converts image data from HWC (Height, Width, Channels; interleaved RGB) format to CHW (Channels, Height, Width; planar) format, then normalizes pixel values to float32 in the [0,1] range. This is required for compatibility with the ONNX model input.
  - **Inference**: Runs the ONNX model on the batch using GPU.
  - **Postprocessing**: Parses model outputs, applies per-tile NMS, and adjusts bounding boxes for global offsets.
- Aggregates detections per source file, tracking expected and processed tiles.
- When all tiles for a file are processed and EOF is received, triggers output.

### 5.5. Output & Publishing

- For each completed file:
  - Applies a global NMS pass to all detections.
  - Serializes results to a JSON file in the output directory.
  - Constructs a `DetectionPayload` and publishes it asynchronously to Zenoh.
  - The same payload is also sent to the DB processor for storage in a PostgreSQL database, enabling persistent and queryable storage of detection results.
  - Uses async tasks (Tokio JoinSet) to save and publish results without blocking the main consumer loop.


### 5.6. Termination

- On receiving `PipelineMessage::Terminate`, flushes any remaining batches.
- Waits for all async save/publish tasks to complete.
- Prints total pipeline execution time and GPU idle time.

---

## 6. Component Roles and Responsibilities

- **Producer**: Efficiently reads and tiles large images, maximizing I/O throughput and parallelism.
- **Channel**: Decouples I/O-bound producer from compute-bound consumer, smoothing bursts and maximizing GPU usage.
- **Consumer**: Maximizes GPU utilization by batching, handles all inference and aggregation logic, and manages output.
- **Preprocessing**: Ensures model receives data in the correct format and normalization.
- **Inference**: Leverages ONNX Runtime with CUDA for fast, batched inference.
- **Postprocessing**: Cleans up model outputs, applies NMS, and prepares results for downstream use.
- **Output/Publisher**: Ensures results are saved and published in both human-readable (JSON) and machine-consumable (Zenoh) formats.

---

## 7. Data Flow Summary (Step-by-Step)

1. **Input**: GeoTIFF images placed in the input directory.
2. **Tiling**: Images are split into overlapping tiles, each tile is prepared as an inference task.
3. **Task Dispatch**: Each tile is sent as a message to the channel.
4. **Batching**: The consumer accumulates tiles into batches of 32.
5. **Preprocessing**: Each batch is converted to a tensor suitable for the ONNX model.
6. **Inference**: The batch is run through the ONNX model on the GPU.
7. **Postprocessing**: Raw outputs are parsed, NMS is applied, and detections are mapped to global coordinates.
8. **Aggregation**: Detections for each source image are collected until all tiles are processed.
9. **Output**: When a file is complete, results are saved as JSON and published to Zenoh.
10. **Completion**: The pipeline terminates gracefully after all files are processed.

---

## 8. Extending or Debugging the System

- **Adding New Models**: Update the ONNX model path and ensure input/output tensor shapes match.
- **Changing Tile Size/Stride**: Modify constants in `virtual_tiler.rs` and preprocessing accordingly.
- **Adding Output Formats**: Extend `results.rs` and/or `publisher.rs` for new serialization or publishing targets.
- **Debugging**: Use logging output, check JSON results, and monitor Zenoh topics for published payloads.

---

This documentation provides a complete, detailed, and accessible overview of the SAMDEF Detector microservice, its architecture, and its data flow. Any engineer should be able to understand, extend, or debug the system using this guide.