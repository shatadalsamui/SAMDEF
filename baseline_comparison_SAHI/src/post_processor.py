"""
SAHI Baseline Post-Processor / Visualizer
Annotates SAHI GeoTIFF detection results (`*_results.json`) with the 15-class
SAMDEF neon color palette, ignoring benchmark metadata JSONs cleanly.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import os
from pathlib import Path
import time
from typing import Optional

import cv2
import numpy as np
from osgeo import gdal
from tqdm import tqdm
import yaml

gdal.UseExceptions()

CATEGORY_NAMES = {
    0: "Light-Vehicle",
    1: "Boxy-Truck",
    2: "Long-Trucks",
    3: "Small-Boat",
    4: "Large-Ship",
    5: "Fixed-Wing",
    6: "Helicopter",
    7: "Building",
    8: "Other-Structure",
    9: "Storage-Tank",
    10: "Container-Lot",
    11: "Construction-Site",
    12: "Railway",
    13: "Engineering-Machinery",
    14: "Tower-Pylon",
}

NEON_RGB = [
    (0, 229, 255),   # 0: Light-Vehicle
    (57, 255, 20),   # 1: Boxy-Truck
    (0, 255, 136),   # 2: Long-Trucks
    (41, 121, 255),  # 3: Small-Boat
    (101, 31, 255),  # 4: Large-Ship
    (255, 255, 0),   # 5: Fixed-Wing
    (255, 109, 0),   # 6: Helicopter
    (255, 23, 68),   # 7: Building
    (255, 82, 82),   # 8: Other-Structure
    (213, 0, 249),   # 9: Storage-Tank
    (255, 214, 0),   # 10: Container-Lot
    (255, 145, 0),   # 11: Construction-Site
    (0, 191, 165),   # 12: Railway
    (245, 0, 87),    # 13: Engineering-Machinery
    (224, 64, 251),  # 14: Tower-Pylon
]


def load_geotiff_bgr(image_path: str) -> np.ndarray:
    """Loads a GeoTIFF via GDAL and returns a contiguous uint8 BGR image for OpenCV."""
    ds = gdal.Open(image_path, gdal.GA_ReadOnly)
    if ds is None:
        raise ValueError(f"Could not open GeoTIFF at {image_path}")

    arr = ds.ReadAsArray()
    if arr.ndim == 3:
        rgb = np.transpose(arr[:3], (1, 2, 0))
    elif arr.ndim == 2:
        rgb = np.stack([arr, arr, arr], axis=-1)
    else:
        raise ValueError(f"Unexpected GeoTIFF shape: {arr.shape}")

    if rgb.dtype != np.uint8:
        rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    return cv2.cvtColor(rgb, cv2.COLOR_RGB2BGR)


def resolve_source_image(payload: dict, json_path: Path, fallback_input_dir: Optional[str]) -> Optional[Path]:
    """Resolves the source GeoTIFF from SAHI JSON or falls back to dataset input_dir by stem."""
    raw = payload.get("source_image")
    if isinstance(raw, str) and raw:
        direct = Path(raw)
        if direct.exists():
            return direct

    if fallback_input_dir:
        stem = json_path.stem.removesuffix("_results")
        for ext in (".tif", ".tiff"):
            candidate = Path(fallback_input_dir) / f"{stem}{ext}"
            if candidate.exists():
                return candidate
    return None


def annotate_scene(
    json_path: Path,
    output_dir: Path,
    conf_threshold: float,
    thickness: int,
    draw_labels: bool,
    fallback_input_dir: Optional[str],
) -> tuple[bool, int]:
    """Renders detections from one SAHI `*_results.json` file onto `<stem>_annotated.png`."""
    with open(json_path, "r") as f:
        payload = json.load(f)

    if not isinstance(payload, dict) or "detections" not in payload or "source_image" not in payload:
        return (False, 0)

    img_path = resolve_source_image(payload, json_path, fallback_input_dir)
    if img_path is None:
        raise FileNotFoundError(f"Missing source GeoTIFF for {json_path.name}")

    canvas = load_geotiff_bgr(str(img_path))
    height, width = canvas.shape[:2]
    drawn = 0

    for det in payload["detections"]:
        conf = float(det.get("confidence", 0.0))
        if conf < conf_threshold:
            continue
        bbox = det.get("bbox", {})
        x1 = max(0, min(width - 1, round(float(bbox["x_min"]))))
        y1 = max(0, min(height - 1, round(float(bbox["y_min"]))))
        x2 = max(x1 + 1, min(width - 1, round(float(bbox["x_max"]))))
        y2 = max(y1 + 1, min(height - 1, round(float(bbox["y_max"]))))

        class_id = int(det.get("class_id", 0))
        r, g, b = NEON_RGB[class_id % len(NEON_RGB)]
        color_bgr = (b, g, r)
        cv2.rectangle(canvas, (x1, y1), (x2, y2), color_bgr, thickness)

        if draw_labels:
            label = f"{CATEGORY_NAMES.get(class_id, class_id)} {conf:.2f}"
            cv2.putText(
                canvas,
                label,
                (x1, max(12, y1 - 4)),
                cv2.FONT_HERSHEY_SIMPLEX,
                0.4,
                color_bgr,
                1,
                cv2.LINE_AA,
            )
        drawn += 1

    out_path = output_dir / f"{img_path.stem}_annotated.png"
    if not cv2.imwrite(str(out_path), canvas):
        raise RuntimeError(f"Failed writing {out_path}")
    return (True, drawn)


def collect_result_files(json_dir: Path, limit: Optional[int]) -> list[Path]:
    """Collects per-scene `*_results.json` files while skipping summary metadata JSONs."""
    files = sorted(p for p in json_dir.glob("*_results.json") if p.is_file())
    return files[:limit] if limit is not None else files


def resolve_dirs(args: argparse.Namespace) -> tuple[Path, Path, Optional[str]]:
    base_dir = Path(__file__).resolve().parent.parent
    config_path = Path(args.config) if Path(args.config).is_absolute() else (base_dir / args.config)
    config = {}
    if config_path.exists():
        with open(config_path, "r") as cf:
            config = yaml.safe_load(cf) or {}

    ds_cfg = config.get("dataset", {})
    default_json = base_dir / ds_cfg.get("output_dir", "results")
    json_dir = Path(args.json_dir) if args.json_dir else default_json
    output_dir = Path(args.output_dir) if args.output_dir else (base_dir / "annotated_results")
    fallback_input = ds_cfg.get("input_dir")
    return json_dir, output_dir, fallback_input


def main():
    parser = argparse.ArgumentParser(description="Annotate SAHI baseline *_results.json outputs")
    parser.add_argument("--config", default="configs/benchmark_config.yaml", help="Benchmark YAML path")
    parser.add_argument("--json-dir", default=None, help="Directory holding SAHI *_results.json files")
    parser.add_argument("--output-dir", default=None, help="Output directory for *_annotated.png")
    parser.add_argument("--conf", type=float, default=0.0, help="Minimum confidence threshold (default: 0.0)")
    parser.add_argument("--thickness", type=int, default=1, help="Bounding box stroke thickness")
    parser.add_argument("--labels", action="store_true", help="Draw class name and confidence text")
    parser.add_argument("--workers", type=int, default=min(8, os.cpu_count() or 4), help="Worker thread count")
    parser.add_argument("--limit", type=int, default=None, help="Optional max scenes to annotate")
    args = parser.parse_args()

    json_dir, output_dir, fallback_input = resolve_dirs(args)
    output_dir.mkdir(parents=True, exist_ok=True)
    result_files = collect_result_files(json_dir, args.limit)
    if not result_files:
        raise SystemExit(f"No *_results.json files found in {json_dir}")

    t0 = time.perf_counter()
    annotated, total_boxes = 0, 0
    with ThreadPoolExecutor(max_workers=max(1, args.workers)) as pool:
        futures = {
            pool.submit(
                annotate_scene,
                path,
                output_dir,
                args.conf,
                args.thickness,
                args.labels,
                fallback_input,
            ): path
            for path in result_files
        }
        for fut in tqdm(as_completed(futures), total=len(futures), desc="Annotating SAHI Scenes"):
            ok, count = fut.result()
            if ok:
                annotated += 1
                total_boxes += count

    elapsed = time.perf_counter() - t0
    print(f"Annotated {annotated} scenes ({total_boxes} boxes, conf>={args.conf}) in {elapsed:.2f}s -> {output_dir}")


if __name__ == "__main__":
    main()

