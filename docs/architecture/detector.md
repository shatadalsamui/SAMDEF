# SAMDEF Detector Microservice Architecture & Data Flow

## 1. Overview

`samdef_detector` is a streaming geospatial object-detection service built in Rust using ONNX Runtime CUDA (`ort`), GDAL (`virtual_tiler.rs`), Crossbeam bounded queues, isolated Rayon pools, Tokio async I/O, and Zenoh messaging.

## 2. Module Map

- **`src/main.rs`**: Parses `.env` config (`INPUT_DIR`, `OUTPUT_DIR`, `MODEL_PATH`, `BATCH_SIZE`, `PRODUCER_PARALLELISM`, optional `BENCHMARK_SUMMARY_PATH`), allocates `TileBytePool`, spawns producer and consumer threads, and writes `SamdefBenchmarkSummary` if configured.
- **`modules/data/`**:
  - `task.rs`: `InferenceTask` and `PipelineMessage` (`Process`, `EndOfFile`, `Terminate`).
  - `buffer_pool.rs`: `TileBytePool` (reusable planar `Vec<u8>`) and `create_pinned_tensor_ring` (`PinAcquirer`, `PinRecycler` over `CUDA_PINNED` memory).
  - `results.rs` / `payload.rs`: JSON serialization (`FinalOutput`, `geo_transform`) and Zenoh publishing (`DetectionPayload`).
  - `benchmark.rs`: `RunStats`, `SamdefBenchmarkSummary`, and JSON writer (`write_summary`).
- **`modules/io/`**:
  - `producer.rs`: Work-stealing GDAL pool across sorted `.tif`/`.tiff` paths (`PRODUCER_PARALLELISM` workers).
  - `virtual_tiler.rs`: Slices each GeoTIFF into `1152x1152` overlapping windows (stride `922`, ~20% overlap) directly in planar `[R | G | B]` CHW layout using `TileBytePool`.
  - `session.rs`: Configures ONNX Runtime `CUDAExecutionProvider` (with CUDA Graph / arena tuning).
  - `consumer.rs`: Coordinates the 5 decoupled consumer stages and returns `RunStats`.
  - `aggregator.rs`: Parses tile tensors, maps to full-image pixel coordinates, and joins tiles per scene.
  - `nms_worker.rs`: Runs scene-level boundary resolution inside an isolated Rayon pool.
  - `publisher.rs`: Publishes detection payloads over Zenoh.
- **`modules/processing/`**:
  - `pre_processing.rs`: L1-resident LUT normalizes planar `u8` tiles into `CUDA_PINNED` FP32 batch tensors on an isolated Rayon pool.
  - `inference.rs`: `GpuInferenceEngine` keeps a persistent `cuda_in` device buffer and `IoBinding` for static CUDA Graph replay.
  - `geometry.rs`, `seam_merger.rs`, `boundary_nms.rs`, `post_processing.rs`: Edge truncation tagging, iterative cross-seam merging, truncated-fragment absorption, and multi-cell spatial grid NMS across 15 classes.

## 3. Producer + 5-Stage Consumer Pipeline

1. **Producer Pool (`producer.rs` -> `msg_tx`)**: `PRODUCER_PARALLELISM` (default `4`) GDAL threads read GeoTIFF bands straight into planar `TileBytePool` buffers and push `PipelineMessage` items into a bounded ring (`max(batch_size * 8, 32)`).
2. **Stage 1 — Preprocessing (`consumer.rs` -> `gpu_tx`, cap `6`)**: Acquires pre-allocated `CUDA_PINNED` host tensors from `PinAcquirer`, normalizes batches via `pre_processing.rs`, and forwards `GpuBatch` + returns raw tile bytes to `TileBytePool`.
3. **Stage 2 — Dedicated GPU Execution (`consumer.rs` -> `post_tx`, cap `128`)**: Copies pinned host memory into static `cuda_in` (`sync_pinned_to_cuda_in`), executes `IoBinding` on CUDA, immediately recycles the pinned host slot via `PinRecycler`, and sends raw `[B, 5000, 6]` outputs to Stage 3.
4. **Stage 3 — Parsing & Aggregation (`aggregator.rs` -> `nms_tx`, cap `64`)**: Tracks tile counts per scene and forwards complete tile sets once `EndOfFile { expected_tiles, .. }` arrives.
5. **Stage 4 — Seam Merge & Spatial Grid NMS (`nms_worker.rs` -> `save_tx`, cap `64`)**: Resolves cross-tile split detections and suppresses duplicates on an isolated Rayon pool.
6. **Stage 5 — Async Output & Telemetry (`consumer.rs` + `results.rs` + `benchmark.rs`)**: Tokio tasks write `<stem>_results.json`, publish to Zenoh, and accumulate `RunStats` (`total_images`, `total_detections`) for `SamdefBenchmarkSummary`.

## 4. Measured Baseline Performance

On 281 validation GeoTIFFs (`run15_FP16.onnx`, `BATCH_SIZE=1`, `PRODUCER_PARALLELISM=4`, NVIDIA GeForce RTX 4060 Laptop GPU):
- **Wall-clock time:** `46.54 s` (`0.1656 s/image`, `6.037 img/s`, `362.24 scenes/min`, `520,192` detections).
- **GPU utilization (`nvidia-smi` @ `250 ms`):** `95.4%` mean, `99.0%` max, `519 MiB` peak VRAM.
- **Batch sweep (`apps_deploy/detector/logs/runs_history.csv`):** `BATCH_SIZE=2` ran in `49.11 s` (`790 MiB` VRAM); `BATCH_SIZE=4` ran in `50.52 s` (`1,349 MiB` VRAM).
- **SAHI parity comparison:** `2.0x` faster than SAHI Python + ONNX CUDA (`93.15 s`, `41.6%` mean GPU util, `1,229 MiB` VRAM). See `baseline_comparison_SAHI/results/comparison_report.md` and `baseline_comparison_SAHI/logs/runs_history.csv`.
