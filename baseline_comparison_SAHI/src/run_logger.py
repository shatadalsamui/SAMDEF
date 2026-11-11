"""Run logging, GPU CSV parsing, and SAMDEF reference helpers for benchmarks."""

import csv
import datetime
import json
import os
import shutil
import subprocess
from pathlib import Path


def utc_iso_now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def git_short_sha(root_dir: Path) -> str:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(root_dir), "rev-parse", "--short", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        return out.strip()
    except Exception:
        return "unknown"


def query_gpu_name() -> str:
    try:
        out = subprocess.check_output(
            ["nvidia-smi", "--query-gpu=name", "--format=csv,noheader"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        return out.strip().splitlines()[0].strip()
    except Exception:
        return "unknown"


def summarize_gpu_csv(csv_path: str | Path | None, interval_ms: int = 250) -> dict | None:
    if not csv_path or not os.path.exists(csv_path):
        return None
    gpu_u, mem_u, mem_mb = [], [], []
    with open(csv_path, "r") as f:
        for row in csv.reader(f):
            if len(row) >= 3:
                try:
                    gpu_u.append(float(row[0].strip()))
                    mem_u.append(float(row[1].strip()))
                    mem_mb.append(float(row[2].strip()))
                except ValueError:
                    pass
    if not gpu_u:
        return None
    return {
        "gpu_name": query_gpu_name(),
        "sample_interval_ms": interval_ms,
        "samples_count": len(gpu_u),
        "mean_gpu_util_pct": round(sum(gpu_u) / len(gpu_u), 1),
        "max_gpu_util_pct": round(max(gpu_u), 1),
        "mean_mem_util_pct": round(sum(mem_u) / len(mem_u), 1),
        "max_mem_util_pct": round(max(mem_u), 1),
        "peak_vram_used_mib": int(max(mem_mb)),
    }


def _load_optional_json(path: str | Path | None) -> dict | None:
    if path and os.path.exists(path):
        with open(path, "r") as f:
            return json.load(f)
    return None


def _resolve_path(base_dir: Path, path_str: str | None) -> str | None:
    if not path_str:
        return None
    p = Path(path_str)
    return str(p if p.is_absolute() else (base_dir / p))


def load_samdef_reference(config: dict, base_dir: Path, output_dir: str, cli_summary: str | None, cli_gpu: str | None):
    samdef_cfg = config.get("samdef", {})
    summary_path = cli_summary or _resolve_path(base_dir, samdef_cfg.get("summary_path")) or os.path.join(output_dir, "samdef_summary.json")
    gpu_path = cli_gpu or _resolve_path(base_dir, samdef_cfg.get("gpu_metrics_path")) or os.path.join(output_dir, "samdef_gpu_metrics.json")

    gpu_metrics = _load_optional_json(gpu_path)
    measured = _load_optional_json(summary_path)
    if measured is not None:
        return measured, gpu_metrics, "measured_json"

    legacy = config.get("samdef_baseline", {})
    fb_secs = samdef_cfg.get("fallback_runtime_seconds", legacy.get("runtime_seconds"))
    fb_scenes = samdef_cfg.get("fallback_scenes_count", legacy.get("scenes_count"))
    if fb_secs is not None and fb_scenes:
        fallback = {"framework": "SAMDEF (YAML fallback)", "total_images": int(fb_scenes), "total_wall_clock_seconds": float(fb_secs)}
        return fallback, gpu_metrics, "yaml_fallback_unverified"
    return None, gpu_metrics, "missing"


def archive_benchmark_run(base_dir: Path, run_id: str, summary: dict, report_path: str | Path) -> tuple[Path, Path, Path]:
    logs_dir = base_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    summary_archive = logs_dir / f"{run_id}_summary.json"
    report_archive = logs_dir / f"{run_id}_report.md"
    history_path = logs_dir / "runs_history.csv"

    with open(summary_archive, "w") as sf:
        json.dump(summary, sf, indent=2)
    shutil.copyfile(report_path, report_archive)

    samdef = summary.get("samdef_metrics") or {}
    samdef_gpu = summary.get("samdef_gpu_metrics") or {}
    sahi_gpu = summary.get("sahi_gpu_metrics") or {}

    sahi_t = summary.get("total_wall_clock_seconds") or 0
    samdef_t = samdef.get("total_wall_clock_seconds") or 0
    sahi_fps = summary.get("throughput_images_per_second") or 0
    samdef_fps = samdef.get("throughput_images_per_second") or 0
    if samdef_t > 0 and summary.get("total_images") == samdef.get("total_images"):
        speedup = round(sahi_t / samdef_t, 2)
    elif sahi_fps > 0 and samdef_fps > 0:
        speedup = round(samdef_fps / sahi_fps, 2)
    else:
        speedup = ""

    header = [
        "run_id", "timestamp_utc", "tag", "git_sha", "model_file", "batch_size", "dataset_limit",
        "total_images", "sahi_detections", "samdef_detections", "sahi_wall_clock_s", "samdef_wall_clock_s",
        "speedup_x", "sahi_img_per_s", "samdef_img_per_s", "sahi_mean_gpu_pct", "sahi_peak_vram_mib",
        "samdef_mean_gpu_pct", "samdef_peak_vram_mib", "samdef_source",
    ]
    write_header = not history_path.exists() or history_path.stat().st_size == 0
    with open(history_path, "a", newline="") as hf:
        w = csv.writer(hf)
        if write_header:
            w.writerow(header)
        w.writerow([
            run_id, summary.get("timestamp_utc", ""), summary.get("tag", ""), summary.get("git_sha", ""),
            summary.get("model_file", ""), summary.get("batch_size", ""), summary.get("dataset_limit", ""),
            summary.get("total_images", ""), summary.get("total_detections", ""), samdef.get("total_detections", ""),
            summary.get("total_wall_clock_seconds", ""), samdef.get("total_wall_clock_seconds", ""), speedup,
            summary.get("throughput_images_per_second", ""), samdef.get("throughput_images_per_second", ""),
            sahi_gpu.get("mean_gpu_util_pct", ""), sahi_gpu.get("peak_vram_used_mib", ""),
            samdef_gpu.get("mean_gpu_util_pct", ""), samdef_gpu.get("peak_vram_used_mib", ""),
            summary.get("samdef_summary_source", ""),
        ])

    return summary_archive, report_archive, history_path

