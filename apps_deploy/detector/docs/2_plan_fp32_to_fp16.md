# Plan: Migrating Detector Pipeline from FP32 to FP16

This document outlines the step-by-step plan to adapt the object detection pipeline from using 32-bit floating-point numbers (FP32) to 16-bit half-precision floating-point numbers (FP16). This is necessary to align with the FP16-quantized ONNX model.

## Phase 1: Project Setup & Data Type Introduction

1.  **Add `half` Crate**: To introduce the `f16` data type, we will add the `half` crate to the project's dependencies.
    -   **File**: `apps_deploy/detector/Cargo.toml`
    -   **Action**: Add `half = { version = "2.2.1", features = ["serde"] }` under `[dependencies]`.

## Phase 2: Updating the Pre-processing Stage

1.  **Modify `pre_processing.rs`**: The goal here is to convert the input `u8` image data directly into `f16` tensors instead of `f32`.
    -   **File**: `apps_deploy/detector/src/modules/processing/pre_processing.rs`
    -   **Actions**:
        -   Import `half::f16`.
        -   Change the return type of `preprocess_image` to `Result<Vec<f16>>`.
        -   In `preprocess_image`, modify the normalization logic to convert `u8` values to `f16`.
        -   Update `preprocess_batch` to correctly handle the `Vec<f16>` from `preprocess_image` and return a flattened `Vec<f16>`.

## Phase 3: Updating the Inference Core

This is the central part of the change, where the model interacts with the data.

1.  **Modify `batch.rs`**: This file orchestrates the processing flow. We'll adjust it to handle the `f16` data type.
    -   **File**: `apps_deploy/detector/src/modules/processing/batch.rs`
    -   **Actions**:
        -   Import `half::f16`.
        -   Update the `input_tensor` creation to use `f16` data from `preprocess_batch`. The type will be `Array<f16, ndarray::IxDyn>`.
        -   The `outputs` variable will now be an `ArrayD<f16>`. The loop that iterates through the results will need to handle this new type.

2.  **Modify `inference.rs`**: This function directly calls the ONNX runtime. It needs to be updated to send and receive `f16` tensors.
    -   **File**: `apps_deploy/detector/src/modules/processing/inference.rs`
    -   **Actions**:
        -   Import `half::f16`.
        -   Change the function signature of `run_inference` to accept an `input_tensor` of type `Array<f16, ndarray::IxDyn>`.
        -   Update the model output extraction to `try_extract_tensor::<f16>()`.
        -   Change the return type of `run_inference` to `Result<ArrayD<f16>>`.

## Phase 4: Updating the Post-processing Stage

1.  **Modify `post_processing.rs`**: This stage interprets the model's raw output. It will now receive `f16` data and must convert it to `f32` for safe and precise calculations.
    -   **File**: `apps_deploy/detector/src/modules/processing/post_processing.rs`
    -   **Actions**:
        -   Import `half::f16`.
        -   Change the function signature of `parse_output` to accept an `output` of type `ArrayView2<f16>`.
        -   Inside `parse_output`, convert all `f16` values read from the `output` tensor to `f32` (e.g., `value.to_f32()`) before they are used in any comparisons or calculations. The `Detection` and `BoundingBox` structs will continue to store `f32` values.

## Summary

This plan ensures that the core pipeline that interacts with the GPU and the ONNX model operates on `f16` data, while the surrounding logic (like final coordinate calculations and JSON serialization) continues to use `f32` for precision and external compatibility.
