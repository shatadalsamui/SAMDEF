# SAHI Baseline Comparison (`baseline_comparison_SAHI`)

Reproducible benchmark comparing **SAHI** (*Slicing Aided Hyper Inference*, Python + ONNX Runtime CUDA) against **SAMDEF** (`samdef_detector`, Rust + ONNX Runtime CUDA) using identical weights, tile geometry, thresholds, and dataset.

## Live Measured Results (281 GeoTIFF Scenes)

Measured end-to-end via `./run_full_comparison.sh` on an **NVIDIA GeForce RTX 4060 Laptop GPU** (`run15_FP16.onnx`):

| Metric | SAHI (Python + CUDA) | SAMDEF (Rust + CUDA) | Notes |
| :--- | :--- | :--- | :--- |
| **Model & Weights** | `run15_FP16.onnx` | `run15_FP16.onnx` | Identical weights |
| **Execution Provider** | `CUDAExecutionProvider` | `CUDAExecutionProvider` | Shared GPU & CUDA runtime |
| **Tile Resolution / Overlap** | `1152x1152`, 20% | `1152x1152`, 20% (`stride=922`) | Matched geometry |
| **Batch Size / Threshold** | `1` / `0.01` | `1` / `0.01` | Matched config |
| **Dataset** | 281 GeoTIFF images | 281 GeoTIFF images | Identical dataset |
| **Total Detections** | 575,876 | 520,192 | Threshold parity |
| **Total Wall-Clock Time** | **93.15 s** | **46.54 s** | **SAMDEF 2.0x faster** |
| **Mean Time / Image** | 0.3201 s | 0.1656 s | 2.0x speedup |
| **Throughput** | 3.017 img/s (181.0 /min) | 6.037 img/s (362.24 /min) | 2.0x speedup |
| **GPU Utilization (Mean / Max)** | 41.6% / 59.0% | **95.4% / 99.0%** | Continuous GPU feed |
| **Peak VRAM Used** | 1,229 MiB | **519 MiB** | Lower device memory |

Multi-run history is logged in [`logs/runs_history.csv`](logs/runs_history.csv) (including paired `batch_size=1` `2.0x` and `batch_size=2` `4.98x` comparisons across all 281 scenes).

## Directory Layout

```text
baseline_comparison_SAHI/
├── configs/benchmark_config.yaml   # Slice geometry, batch_size=1, dataset paths, SAMDEF telemetry paths
├── model/run15_FP16.onnx           # Copy of Run 15 FP16 ONNX model
├── src/
│   ├── sahi_model.py               # SAHI DetectionModel wrapper for ONNX CUDA
│   ├── benchmark_runner.py         # SAHI benchmark loop + live samdef_summary.json comparator
│   ├── run_logger.py               # GPU CSV summarizer, logs/ archiver, and runs_history.csv writer
│   ├── post_processor.py           # Visualizer drawing SAHI *_results.json -> *_annotated.png
│   └── report_generator.py         # Generates results/comparison_report.md
├── logs/
│   ├── runs_history.csv            # Cumulative multi-run benchmark table
│   ├── <run_id>_summary.json       # Archived per-run benchmark & GPU telemetry JSON
│   ├── <run_id>_report.md          # Archived per-run Markdown comparison report
│   └── <run_id>.log                # Raw stdout/stderr log (gitignored)
├── results/
│   ├── samdef_summary.json         # Latest live SAMDEF run telemetry
│   ├── samdef_gpu_metrics.json     # Latest SAMDEF nvidia-smi sample summary
│   ├── benchmark_summary.json      # Latest combined SAHI + SAMDEF benchmark JSON
│   ├── comparison_report.md        # Latest auto-generated comparison report
│   └── *_results.json              # Per-image detection JSONs (gitignored)
├── annotated_results/              # Rendered PNG overlays (gitignored)
├── run_full_comparison.sh          # Invokes ../apps_deploy/detector/run_detector.sh + run_benchmark.sh
├── run_benchmark.sh                # Runs SAHI benchmark + logs GPU metrics & run archive
└── run_post_processor.sh           # Annotates SAHI results using src/post_processor.py
```

## Usage

### Run Full Paired Comparison (Recommended)
Calls `../apps_deploy/detector/run_detector.sh` + `nvidia-smi`, runs SAHI with GPU telemetry, updates `results/`, and archives the run in `logs/`:
```bash
cd baseline_comparison_SAHI
./run_full_comparison.sh
./run_full_comparison.sh --batch 2 --tag b2 --no-build
```

### Run SAHI Standalone
```bash
cd baseline_comparison_SAHI
./run_benchmark.sh
./run_benchmark.sh --limit 10 --tag smoke
```

### Annotate SAHI Results (`*_results.json` -> `*_annotated.png`)
```bash
cd baseline_comparison_SAHI
./run_post_processor.sh --conf 0.25 --thickness 2 --labels
```
*(The Rust `apps_deploy/post_processor` crate also renders both SAMDEF and SAHI `*_results.json` files.)*
