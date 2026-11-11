## Typical Data Flow for One Image

1. **Image Input (`main.rs`, `producer.rs`, `virtual_tiler.rs`)**:
   - Producer worker threads borrow reusable 3.98 MB planar buffers from `TileBytePool`, read 3-band GeoTIFF windows via GDAL into planar `[R | G | B]` CHW layout, and push `PipelineMessage::Process` + `EndOfFile` into the bounded Crossbeam queue.
2. **Stage 1 — Linear LUT Preprocessing (`consumer.rs`, `pre_processing.rs`)**:
   - Acquires pre-allocated page-locked `CUDA_PINNED` batch tensors from `PinAcquirer`, converts `u8 -> f32` via an L1-resident LUT on an isolated Rayon pool, and sends `GpuBatch` to Stage 2.
3. **Stage 2 — Static CUDA Graph Inference (`inference.rs`)**:
   - Owns the CUDA `Session` and static `cuda_in` device buffer bound to `IoBinding` so CUDA Graph replay never sees rotating host addresses; recycles pinned host tensors back via `PinRecycler`.
4. **Stage 3 — Parsing & Scene Aggregation (`aggregator.rs`)**:
   - Parses `[B, 5000, 6]` detections, maps local boxes to full-image pixels, accumulates per-scene tiles until `expected_tiles` arrive, and dispatches complete scenes to Stage 4.
5. **Stage 4 — Boundary-Aware Grid NMS (`nms_worker.rs`, `post_processing.rs`, `seam_merger.rs`, `boundary_nms.rs`)**:
   - Merges targets split across overlapping tile seams and runs multi-cell spatial grid NMS in an isolated Rayon pool.
6. **Stage 5 — Async Persistence & Telemetry (`consumer.rs`, `results.rs`, `publisher.rs`, `benchmark.rs`)**:
   - Writes `*_results.json`, publishes payloads via Zenoh, and optionally writes `SamdefBenchmarkSummary` JSON when `BENCHMARK_SUMMARY_PATH` is set.
