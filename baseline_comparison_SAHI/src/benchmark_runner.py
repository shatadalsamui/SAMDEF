"""
Benchmark Runner: SAHI on High-Resolution GeoTIFFs
Runs full end-to-end benchmark across 281 scenes using SAHI and YOLO ONNX on CUDA.
"""

import argparse
import glob
import json
import os
import sys
import time
from pathlib import Path
import numpy as np
from osgeo import gdal
gdal.UseExceptions()
from sahi.predict import get_sliced_prediction
from tqdm import tqdm
import yaml

# Add src to pythonpath
sys.path.insert(0, str(Path(__file__).parent.parent))
from src.sahi_model import SamdefYoloOnnxDetectionModel
from src.report_generator import generate_report


def load_geotiff(image_path: str) -> np.ndarray:
    """
    Load a GeoTIFF using GDAL and convert to RGB HWC uint8 array.
    Uses unified ReadAsArray for optimal I/O throughput.
    """
    ds = gdal.Open(image_path, gdal.GA_ReadOnly)
    if ds is None:
        raise ValueError(f"Could not open GeoTIFF at {image_path}")

    arr = ds.ReadAsArray()
    if arr.ndim == 3:
        # Multi-band: shape is (bands, height, width)
        rgb = np.transpose(arr[:3], (1, 2, 0))
    elif arr.ndim == 2:
        # Single band: repeat across 3 channels
        rgb = np.stack([arr, arr, arr], axis=-1)
    else:
        raise ValueError(f"Unexpected GeoTIFF array shape: {arr.shape}")

    if rgb.dtype != np.uint8:
        rgb = np.clip(rgb, 0, 255).astype(np.uint8)

    return rgb



def run_benchmark(config_path: str):
    with open(config_path, "r") as f:
        config = yaml.safe_load(f)

    base_dir = Path(config_path).parent.parent
    model_path = os.path.join(base_dir, config["model"]["path"])
    input_dir = config["dataset"]["input_dir"]
    output_dir = os.path.join(base_dir, config["dataset"]["output_dir"])
    os.makedirs(output_dir, exist_ok=True)

    print("=" * 70)
    print(" SAMDEF vs SAHI Performance Benchmark")
    print(f" Model Path : {model_path}")

    print(f" Input Dir  : {input_dir}")
    print(f" Output Dir : {output_dir}")
    print(f" Tile Size  : {config['slicing']['slice_height']}x{config['slicing']['slice_width']}")
    print(f" Overlap    : {config['slicing']['overlap_height_ratio'] * 100:.0f}%")
    print("=" * 70)

    # 1. Initialize SAHI ONNX Model
    print("\n[1/3] Initializing SAHI Detection Model with CUDAExecutionProvider...")
    model = SamdefYoloOnnxDetectionModel(
        model_path=model_path,
        confidence_threshold=config["postprocess"]["confidence_threshold"],
        device=config["model"]["device"],
        image_size=config["model"]["imgsz"],
    )

    # 2. Gather Images
    image_paths = sorted(
        glob.glob(os.path.join(input_dir, "*.tif"))
        + glob.glob(os.path.join(input_dir, "*.tiff"))
    )
    if config["dataset"]["limit"] is not None:
        image_paths = image_paths[: config["dataset"]["limit"]]

    total_images = len(image_paths)
    print(f"[2/3] Found {total_images} GeoTIFF scenes to process.")

    # 3. Benchmark Loop
    print("\n[3/3] Running SAHI Slicing & Inference Loop...")
    per_image_times = []
    total_detections_count = 0
    total_slices_processed = 0

    benchmark_start = time.perf_counter()

    for idx, img_path in enumerate(tqdm(image_paths, desc="Processing Scenes")):
        t0 = time.perf_counter()

        # Load GeoTIFF
        rgb_image = load_geotiff(img_path)

        # Execute SAHI Slicing & Inference
        result = get_sliced_prediction(
            image=rgb_image,
            detection_model=model,
            slice_height=config["slicing"]["slice_height"],
            slice_width=config["slicing"]["slice_width"],
            overlap_height_ratio=config["slicing"]["overlap_height_ratio"],
            overlap_width_ratio=config["slicing"]["overlap_width_ratio"],
            perform_standard_pred=config["slicing"].get("perform_standard_pred", False),
            batch_size=config["slicing"].get("batch_size", 1),
            postprocess_type=config["postprocess"]["type"],
            postprocess_match_metric=config["postprocess"]["match_metric"],
            postprocess_match_threshold=config["postprocess"]["match_threshold"],
            verbose=0,
        )

        t1 = time.perf_counter()
        elapsed = t1 - t0
        per_image_times.append(elapsed)

        preds = result.object_prediction_list
        total_detections_count += len(preds)
        total_slices_processed += len(result.durations_in_seconds) if hasattr(result, "durations_in_seconds") else 0

        # Save structured results JSON matching SAMDEF format
        stem = Path(img_path).stem
        output_json_path = os.path.join(output_dir, f"{stem}_results.json")

        formatted_detections = []
        for p in preds:
            bbox = p.bbox.to_xyxy()
            formatted_detections.append(
                {
                    "bbox": {
                        "x_min": float(bbox[0]),
                        "y_min": float(bbox[1]),
                        "x_max": float(bbox[2]),
                        "y_max": float(bbox[3]),
                    },
                    "class_id": int(p.category.id),
                    "confidence": float(p.score.value),
                }
            )

        output_payload = {
            "source_image": img_path,
            "source_width": rgb_image.shape[1],
            "source_height": rgb_image.shape[0],
            "elapsed_seconds": elapsed,
            "detections_count": len(formatted_detections),
            "detections": formatted_detections,
        }

        with open(output_json_path, "w") as jf:
            json.dump(output_payload, jf)

    benchmark_end = time.perf_counter()
    total_wall_clock = benchmark_end - benchmark_start

    # Summary Statistics
    summary = {
        "framework": "SAHI (Slicing Aided Hyper Inference)",
        "model_file": Path(model_path).name,
        "slice_size": f"{config['slicing']['slice_height']}x{config['slicing']['slice_width']}",
        "overlap": f"{config['slicing']['overlap_height_ratio'] * 100:.0f}%",
        "total_images": total_images,
        "total_wall_clock_seconds": round(total_wall_clock, 2),
        "mean_seconds_per_image": round(float(np.mean(per_image_times)), 4),
        "median_seconds_per_image": round(float(np.median(per_image_times)), 4),
        "p95_seconds_per_image": round(float(np.percentile(per_image_times, 95)), 4),
        "throughput_images_per_second": round(total_images / total_wall_clock, 3),
        "throughput_scenes_per_minute": round((total_images / total_wall_clock) * 60, 2),
        "total_detections": total_detections_count,
        "samdef_baseline_seconds": config["samdef_baseline"]["runtime_seconds"],
        "samdef_baseline_scenes": config["samdef_baseline"]["scenes_count"],
    }

    summary_path = os.path.join(output_dir, "benchmark_summary.json")
    with open(summary_path, "w") as sf:
        json.dump(summary, sf, indent=2)

    print("\n Benchmark Complete!")
    print(f" Summary saved to: {summary_path}")

    # Generate Markdown Report
    report_path = os.path.join(output_dir, "comparison_report.md")
    generate_report(summary, report_path)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run SAHI baseline benchmark on GeoTIFFs")
    parser.add_argument(
        "--config",
        type=str,
        default="configs/benchmark_config.yaml",
        help="Path to YAML configuration",
    )
    args = parser.parse_args()
    run_benchmark(args.config)
