# Master Plan: Migrating Detector Pipeline from FP32 to FP16

## Executive Summary
This document outlines the systematic migration of the Rust-based object detection pipeline to Half-Precision (FP16). Because the YOLO model was exported via ONNX with `half=true`, the runtime expects 16-bit tensors. This migration will cut VRAM consumption by 50% and double inference throughput by leveraging NVIDIA Tensor Cores.

Crucially, this plan implements a "Precision Handover" strategy: GPU inference operates entirely in FP16 for speed, while Post-Processing (NMS and spatial coordinate mapping) reverts to FP32 to prevent bounding box drift and arithmetic underflow.

---

## Phase 1: Project Configuration & Dependencies

**Target File:** `apps_deploy/detector/Cargo.toml`

- **Action 1 (Half Crate):**  
  Add the `half` crate to the dependencies. It must include the `serde` feature (if serialization is needed anywhere) and the `num-traits` feature to allow easy mathematical conversions.

- **Action 2 (ORT Features):**  
  Verify and update the `ort` crate dependency. It must have the `half` feature explicitly enabled so the ONNX runtime understands how to parse the Rust `f16` memory layout. Ensure the `cuda` feature remains enabled.

---

## Phase 2: Optimizing the Pre-Processing Stage

**Target File:** `apps_deploy/detector/src/modules/processing/pre_processing.rs`

- **Action 1 (Signature Updates):**  
  Update the return types of the image processing functions to return collections/vectors of `f16` instead of `f32`.

- **Action 2 (Direct Normalization):**  
  Inside the Rayon parallel iterator (`par_iter`), modify the normalization math. Calculate `(pixel / 255.0)`, but immediately cast/convert the result directly into an `f16` type before collecting it into the final vector.

---

## Phase 3: The Inference Core & Tensor Management

**Target Files:** `batch.rs` and `inference.rs` inside `apps_deploy/detector/src/modules/processing/`

- **Action 1 (Static Tensor Dimensions):**  
  In `batch.rs`, change the tensor allocation from dynamic dimensions (`IxDyn`) to fixed 4D dimensions. Allocate a zeroed `Array4` of type `f16` with the shape `(batch_size, 3, 896, 896)`. This allows the Rust compiler to optimize memory alignment.

- **Action 2 (Data Ingestion):**  
  Populate this new `f16` tensor using the data received from the updated pre-processing stage.

- **Action 3 (ONNX Execution):**  
  In `inference.rs`, update the function signatures to accept the `Array4<f16>` input.

- **Action 4 (Output Extraction):**  
  After `session.run` completes, explicitly instruct the ORT session to extract the resulting tensor as `f16`. The function should return a dynamic array of `f16` to the batch manager.

- **Action 5 (No f32 Downcast):**  
  Do not downcast to `f32` at any point in the inference pipeline. The data should remain as `f16` from pre-processing, through batching, into inference, and out of the ONNX runtime.

---

## Phase 4: Post-Processing & The "Precision Handover"

**Target File:** `apps_deploy/detector/src/modules/processing/post_processing.rs`

- **Action 1 (Data Structs):**  
  Ensure that the `Detection` and `BoundingBox` structs remain entirely unchanged (they should continue to store `f32` fields).

- **Action 2 (Signature Update):**  
  Change the `parse_output` function to accept a 2D Array View of `f16` instead of `f32`.

- **Action 3 (On-The-Fly Conversion):**  
  Inside the loop that iterates over the model's output rows, do not convert the whole array at once (which is slow). Instead:
    - Extract only the confidence score and the class ID, casting the confidence to `f32`.
    - Check if the confidence passes the baseline threshold.
    - If and only if it passes the threshold, extract the four bounding box coordinates (`x_min`, `y_min`, `x_max`, `y_max`) and cast them to `f32`.
    - Clamp and sanitize these `f32` coordinates against the image boundaries.

- **Action 4 (NMS Stability):**  
  Because the structs and the initial parse are now safely handed over to `f32`, the complex O(n) Spatial Grid NMS algorithm will run exactly as before, with perfect mathematical precision.

---

## Phase 5: Post-Migration Tuning (VRAM Capitalization)

**Target File:** `apps_deploy/detector/src/modules/io/consumer.rs` (and potentially `main.rs`)

- **Action 1 (Baseline Verification):**  
  Run the pipeline with the current `BATCH_SIZE` (e.g., 42). Verify that the "Total pipeline execution time" decreases and that detections are visually identical to the FP32 version.

- **Action 2 (Scale Up):**  
  Increment the `BATCH_SIZE` constant from 42 to 64, and subsequently to 84. Monitor GPU utilization (via `nvidia-smi`) to find the new saturation point where GPU memory is fully utilized without throwing Out-Of-Memory (OOM) errors.

---

## Summary

This plan ensures that the core pipeline that interacts with the GPU and the ONNX model operates on `f16` data, while the surrounding logic (like final coordinate calculations and JSON serialization) continues to use `f32` for precision and external compatibility.