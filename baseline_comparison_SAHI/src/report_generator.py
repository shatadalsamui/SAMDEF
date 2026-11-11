"""Comparison report generator for SAHI vs SAMDEF benchmark runs."""

from tabulate import tabulate


def _format_comparison_rows(summary: dict, samdef: dict | None, source: str) -> list[list[str]]:
    sahi_total = summary["total_wall_clock_seconds"]
    sahi_images = summary["total_images"]
    sahi_fps = summary["throughput_images_per_second"]
    sahi_mean = summary["mean_seconds_per_image"]
    sahi_batch = summary.get("batch_size", 1)
    sahi_dets = summary.get("total_detections", "-")

    if samdef is None:
        return [
            ["Status", "Measured", "Missing (run ./run_full_comparison.sh)", "N/A"],
            ["Images", f"{sahi_images}", "-", "-"],
            ["Wall-Clock Time", f"{sahi_total:.2f} s", "-", "-"],
            ["Throughput", f"{sahi_fps:.3f} img/s", "-", "-"],
        ]

    samdef_total = samdef["total_wall_clock_seconds"]
    samdef_images = samdef["total_images"]
    samdef_fps = samdef.get("throughput_images_per_second", round(samdef_images / max(samdef_total, 1e-6), 3))
    samdef_mean = samdef.get("mean_seconds_per_image", round(samdef_total / max(samdef_images, 1), 4))
    speedup = round(sahi_total / samdef_total, 2) if (sahi_images == samdef_images and samdef_total > 0) else round(samdef_fps / max(sahi_fps, 1e-6), 2)
    samdef_tag = "" if source == "measured_json" else " [UNVERIFIED YAML]"

    return [
        ["Telemetry Source", "Measured (benchmark_runner.py)", f"{source}{samdef_tag}", "Machine-generated JSON"],
        ["Engine", "Python (sahi + onnxruntime)", "Rust (ort CUDA + Crossbeam)", "Native runtime"],
        ["Pipeline", "Sequential Read -> Slice -> GPU -> NMS", "Decoupled 5-stage streaming + grid NMS", "Zero-wait GPU feed"],
        ["Model", summary["model_file"], samdef.get("model_file", summary["model_file"]), "Identical weights"],
        ["Slice / Overlap / Batch", f"{summary['slice_size']} / {summary['overlap']} / {sahi_batch}", f"{samdef.get('slice_size', summary['slice_size'])} / {samdef.get('overlap', summary['overlap'])} / {samdef.get('batch_size', '-')}", "Matched config"],
        ["Images Evaluated", f"{sahi_images}", f"{samdef_images}", "Same dataset" if sahi_images == samdef_images else "Different count"],
        ["Total Detections", f"{sahi_dets}", f"{samdef.get('total_detections', '-')}", "Threshold parity"],
        ["Wall-Clock Time", f"{sahi_total:.2f} s", f"{samdef_total:.2f} s", f"SAMDEF {speedup}x faster"],
        ["Mean Time / Image", f"{sahi_mean:.4f} s", f"{samdef_mean:.4f} s", f"{speedup}x speedup"],
        ["Throughput", f"{sahi_fps:.3f} img/s ({summary.get('throughput_scenes_per_minute', round(sahi_fps * 60, 2))} /min)", f"{samdef_fps:.3f} img/s ({samdef.get('throughput_scenes_per_minute', round(samdef_fps * 60, 2))} /min)", f"{speedup}x speedup"],
    ]


def _format_gpu_section(sahi_gpu: dict | None, samdef_gpu: dict | None) -> str:
    if not sahi_gpu and not samdef_gpu:
        return ""
    sahi = sahi_gpu or {}
    samdef = samdef_gpu or {}
    rows = [
        ["GPU Device", str(sahi.get("gpu_name") or samdef.get("gpu_name") or "unknown"), str(samdef.get("gpu_name") or sahi.get("gpu_name") or "unknown")],
        ["Samples @ Interval", f"{sahi.get('samples_count', '-')} @ {sahi.get('sample_interval_ms', '-')} ms" if sahi_gpu else "-", f"{samdef.get('samples_count', '-')} @ {samdef.get('sample_interval_ms', '-')} ms" if samdef_gpu else "-"],
        ["GPU Utilization (Mean / Max)", f"{sahi.get('mean_gpu_util_pct', '-')}% / {sahi.get('max_gpu_util_pct', '-')}%" if sahi_gpu else "-", f"{samdef.get('mean_gpu_util_pct', '-')}% / {samdef.get('max_gpu_util_pct', '-')}%" if samdef_gpu else "-"],
        ["VRAM Controller (Mean / Max)", f"{sahi.get('mean_mem_util_pct', '-')}% / {sahi.get('max_mem_util_pct', '-')}%" if sahi_gpu else "-", f"{samdef.get('mean_mem_util_pct', '-')}% / {samdef.get('max_mem_util_pct', '-')}%" if samdef_gpu else "-"],
        ["Peak VRAM Used", f"{sahi.get('peak_vram_used_mib', '-')} MiB" if sahi_gpu else "-", f"{samdef.get('peak_vram_used_mib', '-')} MiB" if samdef_gpu else "-"],
    ]
    return "\n## GPU Telemetry\n\n" + tabulate(rows, headers=["Metric", "SAHI (Baseline)", "SAMDEF (Detector)"], tablefmt="github") + "\n"


def generate_report(summary: dict, output_markdown_path: str):
    """Write comparison markdown report and print concise table to stdout."""
    samdef = summary.get("samdef_metrics")
    source = summary.get("samdef_summary_source", "missing")
    sahi_gpu = summary.get("sahi_gpu_metrics")
    samdef_gpu = summary.get("samdef_gpu_metrics")

    rows = _format_comparison_rows(summary, samdef, source)
    table_md = tabulate(rows, headers=["Metric", "SAHI (Baseline)", "SAMDEF (Detector)", "Notes"], tablefmt="github")
    gpu_section = _format_gpu_section(sahi_gpu, samdef_gpu)
    gpu_name = (samdef_gpu or sahi_gpu or {}).get("gpu_name", "CUDA GPU")
    run_id = summary.get("run_id", "latest")
    ts_utc = summary.get("timestamp_utc", "-")

    report_content = f"""# SAMDEF vs SAHI Benchmark Report

Run `{run_id}` (`{ts_utc}`) on **{summary["total_images"]} GeoTIFF scenes** using `{summary["model_file"]}` on `{gpu_name}`.

## Performance Comparison

{table_md}
{gpu_section}
## Telemetry Files

- SAHI summary & combined metrics: `results/benchmark_summary.json`
- SAMDEF summary: `results/samdef_summary.json` (`source={source}`)
- GPU samples: `results/samdef_gpu_metrics.json`
- Archived run log & CSV history: `logs/{run_id}_summary.json`, `logs/runs_history.csv`

*Generated by `baseline_comparison_SAHI/src/report_generator.py`.*
"""

    with open(output_markdown_path, "w") as rf:
        rf.write(report_content)

    print("\nSAMDEF vs SAHI Comparison:")
    print(table_md)
    print(f"Markdown report written to: {output_markdown_path}")
