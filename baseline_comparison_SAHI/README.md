# SAHI Baseline Comparison Harness (`baseline_comparison_SAHI`)

This directory provides a reproducible benchmark harness comparing **SAHI** (*Slicing Aided Hyper Inference*) against **SAMDEF**'s streaming Rust pipeline.

---

## Benchmark Results: SAMDEF vs. SAHI (281 Multi-Band GeoTIFF Images)

Both systems evaluated on the **identical 281 multi-band GeoTIFF images** using the **identical ONNX model weights (`run15_FP16.onnx`)** executed on an **NVIDIA GeForce RTX 4060 Laptop GPU (8GB GDDR6)** via ONNX Runtime CUDA:

| Metric | SAHI Baseline (Python + CUDA) | SAMDEF (Rust Engine) | Advantage / Speedup |
| :--- | :--- | :--- | :--- |
| **Model & Weights** | `run15_FP16.onnx` (FP16 Tensor Cores) | `run15_FP16.onnx` (FP16 Tensor Cores) | **Identical weights** |
| **Execution Provider** | ONNX Runtime `CUDAExecutionProvider` | ONNX Runtime `CUDAExecutionProvider` | **Identical hardware & CUDA runtime** |
| **Tile Resolution** | $1152 \times 1152$ | $1152 \times 1152$ | **Identical tile size** |
| **Tile Overlap** | 20% overlap (`stride = 922`) | 20% overlap (`stride = 922`) | **Exact same spatial stride** |
| **Classes** | 15 Classes | 15 Classes | **Identical class mapping** |
| **Confidence Cutoff**| `0.01` | `0.01` | **Identical detection threshold** |
| **Dataset Evaluated** | 281 GeoTIFF images | 281 GeoTIFF images | **Exact same imagery** |
| **Total Wall-Clock Time** | **99.07 s** | **49.83 s** | **SAMDEF is 2.0× FASTER** |


---

## 1. Benchmark Methodology


Controlled test configuration across both systems:
- **Identical Model**: Both systems run the exact same ONNX model ([`model/run15_FP16.onnx`](model/run15_FP16.onnx)) with FP16 Tensor Core weights.
- **Identical Execution Engine**: Both run on ONNX Runtime with `CUDAExecutionProvider` on the same NVIDIA RTX 4060 GPU.
- **Identical Slicing Dimensions**: Both slice GeoTIFF imagery into $1152 \times 1152$ tiles with 20% overlap (`overlap_ratio=0.20`, matching SAMDEF's 922 stride).
- **Identical Validation Dataset**: Both process the same 281 multi-band GeoTIFF images from `/home/shatadal/SAMDEF_DATA/val_images`.
- **Identical Thresholds**: Confidence threshold: `0.01`, IoU threshold: `0.45`.

---

## 2. Directory Layout

```text
baseline_comparison_SAHI/
├── configs/
│   └── benchmark_config.yaml    # Declarative YAML: model path, dataset path, 1152px slice, 0.20 overlap
├── model/
│   └── run15_FP16.onnx          # Standalone copy of the Run 15 YOLO ONNX model
├── src/
│   ├── __init__.py
│   ├── sahi_model.py            # Custom SAHI DetectionModel running run15_FP16.onnx on CUDA
│   ├── benchmark_runner.py      # End-to-end benchmark runner across 281 GeoTIFFs
│   └── report_generator.py      # Generates markdown comparison tables & throughput metrics
├── results/                     # Per-image detection JSONs and benchmark_summary.json
├── requirements.txt             # Python dependencies
└── run_benchmark.sh             # Execution script
```

---

## 3. Running the Benchmark

### One-Command Execution:
```bash
cd baseline_comparison_SAHI
./run_benchmark.sh
```

### Or using Python directly:
```bash
/home/shatadal/PROJECTS/SAMDEF/apps_training/yolo_model_trainer/venv/bin/python3 src/benchmark_runner.py --config configs/benchmark_config.yaml
```

The script will:
1. Initialize the ONNX Runtime CUDA session for `model/run15_FP16.onnx`.
2. Process all 281 GeoTIFF images through SAHI's `get_sliced_prediction` ($1152 \times 1152$, 20% overlap).
3. Record high-precision execution timings.
4. Save detection JSONs to `results/`.
5. Output `results/benchmark_summary.json` and `results/comparison_report.md` comparing SAHI directly with SAMDEF's verified **49.83s** baseline.



