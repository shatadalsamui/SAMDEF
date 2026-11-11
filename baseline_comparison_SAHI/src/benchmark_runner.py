"""
Benchmark Runner: SAHI on High-Resolution GeoTIFFs
Runs SAHI baseline benchmark across GeoTIFF scenes and archives telemetry into logs/.
"""

import argparse
import datetime
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

sys.path.insert(0, str(Path(__file__).parent.parent))
from src.sahi_model import SamdefYoloOnnxDetectionModel
from src.report_generator import generate_report
from src.run_logger import (
    archive_benchmark_run,
    git_short_sha,
    load_samdef_reference,
    summarize_gpu_csv,
    utc_iso_now,
)


def load_geotiff(image_path: str) -> tuple[np.ndarray, list[float]]:
    """Load a GeoTIFF via GDAL as RGB HWC uint8 plus its 6-param GeoTransform."""
    ds = gdal.Open(image_path, gdal.GA_ReadOnly)
    if ds is None:
        raise ValueError(f"Could not open GeoTIFF at {image_path}")

    geo_transform = list(ds.GetGeoTransform() or (0.0, 1.0, 0.0, 0.0, 0.0, 1.0))
    arr = ds.ReadAsArray()
    if arr.ndim == 3:
        rgb = np.transpose(arr[:3], (1, 2, 0))
    elif arr.ndim == 2:
        rgb = np.stack([arr, arr, arr], axis=-1)
    else:
        raise ValueError(f"Unexpected GeoTIFF array shape: {arr.shape}")

    if rgb.dtype != np.uint8:
        rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    return rgb, geo_transform


def run_benchmark(
    config_path: str,
    samdef_summary_path: str | None = None,
    gpu_metrics_path: str | None = None,
    limit: int | None = None,
    batch_override: int | None = None,
    run_id: str | None = None,
    tag: str = "",
    sahi_gpu_csv: str | None = None,
):
    with open(config_path, "r") as f:
        config = yaml.safe_load(f)

    base_dir = Path(config_path).resolve().parent.parent
    repo_root = base_dir.parent
    model_path = os.path.join(base_dir, config["model"]["path"])
    input_dir = config["dataset"]["input_dir"]
    output_dir = os.path.join(base_dir, config["dataset"]["output_dir"])
    os.makedirs(output_dir, exist_ok=True)

    batch_size = int(batch_override or config["slicing"].get("batch_size", 1))
    effective_limit = limit if limit is not None else config["dataset"]["limit"]
    if not run_id:
        ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        run_id = f"run_{ts}_b{batch_size}" + (f"_{tag}" if tag else "")

    print(f"SAHI Benchmark | run_id={run_id} | model={Path(model_path).name} | input={input_dir}")
    print(f"Config: tile={config['slicing']['slice_height']}x{config['slicing']['slice_width']}, overlap={config['slicing']['overlap_height_ratio'] * 100:.0f}%, batch={batch_size}, limit={effective_limit}")

    print("[1/3] Initializing SAHI Detection Model with CUDAExecutionProvider...")
    model = SamdefYoloOnnxDetectionModel(
        model_path=model_path,
        confidence_threshold=config["postprocess"]["confidence_threshold"],
        device=config["model"]["device"],
        image_size=config["model"]["imgsz"],
    )

    image_paths = sorted(glob.glob(os.path.join(input_dir, "*.tif")) + glob.glob(os.path.join(input_dir, "*.tiff")))
    if effective_limit is not None:
        image_paths = image_paths[:effective_limit]

    total_images = len(image_paths)
    print(f"[2/3] Found {total_images} GeoTIFF scenes to process.")
    print("[3/3] Running SAHI Slicing & Inference Loop...")

    per_image_times = []
    total_detections_count = 0
    benchmark_start = time.perf_counter()

    for img_path in tqdm(image_paths, desc="Processing Scenes"):
        t0 = time.perf_counter()
        rgb_image, geo_transform = load_geotiff(img_path)

        result = get_sliced_prediction(
            image=rgb_image,
            detection_model=model,
            slice_height=config["slicing"]["slice_height"],
            slice_width=config["slicing"]["slice_width"],
            overlap_height_ratio=config["slicing"]["overlap_height_ratio"],
            overlap_width_ratio=config["slicing"]["overlap_width_ratio"],
            perform_standard_pred=config["slicing"].get("perform_standard_pred", False),
            batch_size=batch_size,
            postprocess_type=config["postprocess"]["type"],
            postprocess_match_metric=config["postprocess"]["match_metric"],
            postprocess_match_threshold=config["postprocess"]["match_threshold"],
            verbose=0,
        )

        elapsed = time.perf_counter() - t0
        per_image_times.append(elapsed)
        preds = result.object_prediction_list
        total_detections_count += len(preds)

        output_json_path = os.path.join(output_dir, f"{Path(img_path).stem}_results.json")
        formatted_detections = [
            {
                "bbox": {"x_min": float(b[0]), "y_min": float(b[1]), "x_max": float(b[2]), "y_max": float(b[3])},
                "class_id": int(p.category.id),
                "confidence": float(p.score.value),
            }
            for p in preds
            for b in (p.bbox.to_xyxy(),)
        ]
        payload = {
            "source_image": img_path,
            "geo_transform": geo_transform,
            "source_width": rgb_image.shape[1],
            "source_height": rgb_image.shape[0],
            "elapsed_seconds": elapsed,
            "detections_count": len(formatted_detections),
            "detections": formatted_detections,
        }
        with open(output_json_path, "w") as jf:
            json.dump(payload, jf)

    total_wall_clock = time.perf_counter() - benchmark_start
    samdef_metrics, samdef_gpu_metrics, samdef_source = load_samdef_reference(
        config, base_dir, output_dir, samdef_summary_path, gpu_metrics_path
    )
    sahi_gpu_metrics = summarize_gpu_csv(sahi_gpu_csv)

    summary = {
        "run_id": run_id,
        "timestamp_utc": utc_iso_now(),
        "tag": tag,
        "git_sha": git_short_sha(repo_root),
        "framework": "SAHI (Slicing Aided Hyper Inference)",
        "model_file": Path(model_path).name,
        "slice_size": f"{config['slicing']['slice_height']}x{config['slicing']['slice_width']}",
        "overlap": f"{config['slicing']['overlap_height_ratio'] * 100:.0f}%",
        "batch_size": batch_size,
        "dataset_limit": effective_limit,
        "total_images": total_images,
        "total_wall_clock_seconds": round(total_wall_clock, 2),
        "mean_seconds_per_image": round(float(np.mean(per_image_times)), 4) if per_image_times else 0.0,
        "median_seconds_per_image": round(float(np.median(per_image_times)), 4) if per_image_times else 0.0,
        "p95_seconds_per_image": round(float(np.percentile(per_image_times, 95)), 4) if per_image_times else 0.0,
        "throughput_images_per_second": round(total_images / max(total_wall_clock, 1e-6), 3),
        "throughput_scenes_per_minute": round((total_images / max(total_wall_clock, 1e-6)) * 60, 2),
        "total_detections": total_detections_count,
        "sahi_gpu_metrics": sahi_gpu_metrics,
        "samdef_summary_source": samdef_source,
        "samdef_metrics": samdef_metrics,
        "samdef_gpu_metrics": samdef_gpu_metrics,
    }
    if samdef_metrics is not None:
        summary["samdef_baseline_seconds"] = samdef_metrics["total_wall_clock_seconds"]
        summary["samdef_baseline_scenes"] = samdef_metrics["total_images"]

    summary_path = os.path.join(output_dir, "benchmark_summary.json")
    with open(summary_path, "w") as sf:
        json.dump(summary, sf, indent=2)

    report_path = os.path.join(output_dir, "comparison_report.md")
    generate_report(summary, report_path)
    s_arch, r_arch, hist = archive_benchmark_run(base_dir, run_id, summary, report_path)
    print(f"Summary written to:  {summary_path}")
    print(f"Run archive written: {s_arch} | {r_arch}")
    print(f"History updated:     {hist}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run SAHI baseline benchmark on GeoTIFFs")
    parser.add_argument("--config", type=str, default="configs/benchmark_config.yaml", help="Path to YAML config")
    parser.add_argument("--samdef-summary", type=str, default=None, help="Path to measured samdef_summary.json")
    parser.add_argument("--gpu-metrics", type=str, default=None, help="Path to measured samdef_gpu_metrics.json")
    parser.add_argument("--limit", type=int, default=None, help="Optional scene limit override")
    parser.add_argument("--batch-size", type=int, default=None, help="Optional SAHI batch_size override")
    parser.add_argument("--run-id", type=str, default=None, help="Optional run identifier for logs/")
    parser.add_argument("--tag", type=str, default="", help="Optional run tag saved in logs/")
    parser.add_argument("--sahi-gpu-csv", type=str, default=None, help="Optional nvidia-smi CSV trace for SAHI run")
    args = parser.parse_args()
    run_benchmark(
        args.config,
        args.samdef_summary,
        args.gpu_metrics,
        args.limit,
        args.batch_size,
        args.run_id,
        args.tag,
        args.sahi_gpu_csv,
    )
