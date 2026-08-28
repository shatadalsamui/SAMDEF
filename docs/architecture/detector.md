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
+-------------------+      +-------------------+      +-------------------+      +-------------------+      +-------------------+      +-------------------+
|                   |      |                   |      |                   |      |                   |      |                   |      |                   |
|   Producer Thread | ---> |   Channel (Queue) | ---> |  Consumer Thread  | ---> |   Output/Publish  | ---> |   DB Processor    | ---> |   Rust Iced GUI   |
|                   |      |                   |      |                   |      |                   |      |                   |      |                   |
+-------------------+      +-------------------+      +-------------------+      +-------------------+      +-------------------+      +-------------------+
        |                        |                           |                          |                       |                          |
        |                        |                           |                          |                       |                          |
        v                        v                           v                          v                       v                          v
[Read GeoTIFFs]         [Batched PipelineMessages]   [Batch Inference, Aggregation]   [JSON, Zenoh]      [Output to DB]           [Native GUI from DB]
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
- **task.rs**: Defines `InferenceTask` (tile data, spatial offsets) and `PipelineMessage` (task, end-of-file with geotransform, terminate).
- **results.rs**: Handles aggregation and saving of detection results, including JSON serialization and Zenoh publishing.
- **payload.rs**: Defines the structure for detection payloads sent over Zenoh.

#### `modules/io`
- **producer.rs**: Uses a continuous work-stealing pool of persistent worker threads to stream tiles from GeoTIFFs into the channel without barrier stalls.
- **consumer.rs**: Implements a 3-stage decoupled pipeline: CPU batch preprocessing, dedicated GPU inference runner, and asynchronous postprocessing / Tokio file saving.
- **session.rs**: Initializes ONNX inference session with GPU CUDA execution provider.
- **publisher.rs**: Publishes detection payloads to Zenoh.
- **virtual_tiler.rs**: Handles virtual tiling of large GeoTIFFs into fixed-size tiles (896x896) with configurable stride overlap.

#### `modules/processing`
- **pre_processing.rs**: Converts raw image data into normalized planar FP16 tensors suitable for model input.
- **inference.rs**: Runs the ONNX model on input tensors on the GPU.
- **post_processing.rs**: Parses model outputs, applies spatial non-maximum suppression (NMS), and maps detections to global coordinates.

---

## 5. Detailed Data Flow

### 5.1. Initialization

- The service starts, initializes logging and the ONNX runtime, and prepares input/output directories.
- A bounded channel is created for passing `PipelineMessage` objects between producer and consumer.

### 5.2. Producer Thread & Continuous Worker Pool

- Scans the input directory for GeoTIFF files and sorts them.
- Creates a path work queue and spawns `PRODUCER_PARALLELISM` persistent worker threads.
- Each worker thread:
  - Continuously receives image paths from the work queue with zero barrier pauses.
  - Opens a GeoTIFF using GDAL.
  - Tiles the image into fixed-size (896x896) RGB tiles with 20% overlap (stride).
  - For each tile, creates an `InferenceTask` containing the raw tile bytes and global spatial offsets.
  - Sends tiles sequentially as `PipelineMessage::Process(InferenceTask)` to the channel.
  - After all tiles for that file are sent, sends a `PipelineMessage::EndOfFile` with spatial metadata and expected tile count.
- After all files are processed, the main producer sends a single `PipelineMessage::Terminate` to signal completion.

### 5.3. Channel (Queue)

- A bounded, thread-safe channel (Crossbeam) buffers messages between producer and consumer.
- Optimal configuration: `BATCH_SIZE=2` and `PRODUCER_PARALLELISM=2` for maximum throughput (44.59s) and 90%+ flatline GPU utilization.

### 5.4. Decoupled 3-Stage Consumer Pipeline (OS Threads vs Rayon)

The consumer architecture is decoupled into **3 dedicated OS threads** connected by Crossbeam channels to ensure the GPU is 100% active without waiting for CPU preparation:

1. **Stage 1 (OS Thread A - CPU Preprocessor & Batch Accumulator)**:
   - Runs as a persistent OS thread (`thread::spawn`).
   - Receives tiles from the input channel and bundles them into batches.
   - Uses Rayon data parallelism (`par_iter`) internally across CPU cores to normalize pixels and transpose HWC to CHW planar format in sub-milliseconds.
   - Forwards ready `Array4<f16>` tensors to the dedicated GPU channel.
   - Forwards `EndOfFile` metadata directly to Stage 3.
2. **Stage 2 (OS Thread B - Dedicated GPU Inference Runner)**:
   - Runs as a persistent OS thread (`thread::spawn`) holding the CUDA `Session`.
   - **Pure GPU execution loop**: Has zero CPU math, zero file I/O, and zero HashMap lookups.
   - Pulls ready tensors from the preprocessor channel and executes `session.run()` immediately on CUDA.
   - Forwards raw output tensors to Stage 3 without blocking.
3. **Stage 3 (OS Thread C - Postprocessing, NMS & Tokio Async Output)**:
   - Runs on the main Tokio runtime thread.
   - Parses bounding box coordinates, shifts by spatial tile offsets, and tracks per-file tile completion in a `HashMap`.
   - When all tiles for a file are processed, runs spatial NMS and spawns background Tokio `JoinSet` tasks to write JSON and publish via Zenoh.

#### Concurrency Advantage:
- **Thread A** prepares Batch $N+1$ on CPU cores.
- **Thread B** runs Batch $N$ on the RTX 4060 GPU simultaneously.
- **Thread C** saves Batch $N-1$ to disk and network simultaneously.
- None of the 3 stages ever block each other.

#### 5.4.1. Bounded Channels & Automatic Backpressure Flow
The stages communicate via bounded Crossbeam channels (`gpu_tx: bounded(4)`, `post_tx: bounded(64)`):
- **Zero-Wait GPU Feed**: Thread 1 keeps up to 4 preprocessed batches buffered in memory. Whenever Thread 2 (GPU) finishes a batch, the next tensor is pulled instantly in 0 microseconds without waiting for CPU math.
- **Natural Backpressure**: If CPU preprocessing outpaces GPU execution, `gpu_tx.send()` naturally pauses Thread 1 once the 4-batch cushion is full. This prevents unbounded memory growth in RAM.
- **Immediate Resumption**: As soon as the GPU dequeues a batch, Thread 1 unblocks instantly and prepares the next batch in parallel.

### 5.5. Output & Publishing

- For each completed file:
  - Applies a global NMS pass to all detections.
  - Serializes results to a JSON file in the output directory.
  - Constructs a `DetectionPayload` and publishes it asynchronously to Zenoh.
  - The same payload is also sent to the DB processor for storage in a PostgreSQL database, enabling persistent and queryable storage of detection results.
  - Uses async tasks (Tokio JoinSet) to save and publish results without blocking the main consumer loop.

### 5.6. Termination

- On receiving `PipelineMessage::Terminate`, flushes any remaining batches through the GPU and postprocessor.
- Awaits all async save/publish tasks to complete before exiting.
- Prints total pipeline execution time.

---

## 6. Component Roles and Responsibilities

- **Producer**: Efficiently reads and tiles large images using a continuous work-stealing thread pool.
- **Channel**: Decouples I/O-bound producer from compute-bound consumer, smoothing bursts and maximizing GPU usage.
- **Consumer**: Orchestrates the 3-stage decoupled pipeline for continuous 90%+ GPU utilization.
- **Preprocessing**: Ensures model receives data in the correct FP16 CHW format.
- **Inference**: Leverages ONNX Runtime with CUDA for fast, uninterrupted batched inference.
- **Postprocessing**: Cleans up model outputs, applies NMS, and prepares results for downstream use.
- **Output/Publisher**: Ensures results are saved and published in both human-readable (JSON) and machine-consumable (Zenoh) formats.

---

## 7. Data Flow Summary (Step-by-Step)

1. **Input**: GeoTIFF images placed in the input directory.
2. **Streaming Tiling**: Producer worker threads continuously split images into overlapping tiles.
3. **Task Dispatch**: Tiles are sent as `PipelineMessage` items through a Crossbeam channel.
4. **Batch Preprocessing (Stage 1)**: CPU thread converts batches to normalized `Array4<f16>` tensors.
5. **Dedicated Inference (Stage 2)**: GPU thread executes inference on CUDA uninterrupted.
6. **Postprocessing & NMS (Stage 3)**: Bounding boxes are parsed, shifted to global coordinates, and aggregated.
7. **Async Output**: Completed files are serialized to JSON and published to Zenoh via Tokio background tasks.
8. **Completion**: The pipeline terminates gracefully after all files are processed.

---

## 8. Extending or Debugging the System

- **Adding New Models**: Update the ONNX model path and ensure input/output tensor shapes match.
- **Changing Tile Size/Stride**: Modify constants in `virtual_tiler.rs` and preprocessing accordingly.
- **Adding Output Formats**: Extend `results.rs` and/or `publisher.rs` for new serialization or publishing targets.
- **Debugging**: Use logging output, check JSON results, and monitor Zenoh topics for published payloads.

---

This documentation provides a complete, detailed, and accessible overview of the SAMDEF Detector microservice, its architecture, and its data flow. Any engineer should be able to understand, extend, or debug the system using this guide.