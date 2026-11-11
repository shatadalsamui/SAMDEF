#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOGS_DIR="$SCRIPT_DIR/logs"

usage() {
    cat <<EOF
Usage: ./run_detector.sh [options]
  --input DIR         Override INPUT_DIR
  --output DIR        Override OUTPUT_DIR (default: .env or ./results)
  --model PATH        Override MODEL_PATH (default: .env or model/run15_FP16.onnx)
  --batch N           Override BATCH_SIZE (default: .env or 1)
  --parallel N        Override PRODUCER_PARALLELISM (default: .env or 4)
  --tag LABEL         Optional tag appended to run_id and history
  --summary-out PATH  Copy SamdefBenchmarkSummary JSON to PATH
  --gpu-out PATH      Write GPU telemetry JSON to PATH
  --no-build          Skip cargo build --release
  -h, --help          Show this help
EOF
}

ORIG_INPUT="${INPUT_DIR:-}"
ORIG_OUTPUT="${OUTPUT_DIR:-}"
ORIG_MODEL="${MODEL_PATH:-}"
ORIG_BATCH="${BATCH_SIZE:-}"
ORIG_PAR="${PRODUCER_PARALLELISM:-}"

if [ -f "$SCRIPT_DIR/.env" ]; then
    set -a
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/.env"
    set +a
fi

[ -n "$ORIG_INPUT" ] && INPUT_DIR="$ORIG_INPUT"
[ -n "$ORIG_OUTPUT" ] && OUTPUT_DIR="$ORIG_OUTPUT"
[ -n "$ORIG_MODEL" ] && MODEL_PATH="$ORIG_MODEL"
[ -n "$ORIG_BATCH" ] && BATCH_SIZE="$ORIG_BATCH"
[ -n "$ORIG_PAR" ] && PRODUCER_PARALLELISM="$ORIG_PAR"

TAG=""
SUMMARY_OUT=""
GPU_OUT=""
NO_BUILD=0

while [ $# -gt 0 ]; do
    case "$1" in
        --input) INPUT_DIR="$2"; shift 2 ;;
        --output) OUTPUT_DIR="$2"; shift 2 ;;
        --model) MODEL_PATH="$2"; shift 2 ;;
        --batch) BATCH_SIZE="$2"; shift 2 ;;
        --parallel) PRODUCER_PARALLELISM="$2"; shift 2 ;;
        --tag) TAG="$2"; shift 2 ;;
        --summary-out) SUMMARY_OUT="$2"; shift 2 ;;
        --gpu-out) GPU_OUT="$2"; shift 2 ;;
        --no-build) NO_BUILD=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    esac
done

INPUT_DIR="${INPUT_DIR:?INPUT_DIR must be set in .env or via --input}"
OUTPUT_DIR="${OUTPUT_DIR:-$SCRIPT_DIR/results}"
MODEL_PATH="${MODEL_PATH:-$SCRIPT_DIR/model/run15_FP16.onnx}"
BATCH_SIZE="${BATCH_SIZE:-1}"
PRODUCER_PARALLELISM="${PRODUCER_PARALLELISM:-4}"

if [ "$NO_BUILD" -eq 0 ]; then
    echo "Building samdef_detector (release)..."
    cargo build --release -p samdef_detector --manifest-path "$ROOT_DIR/Cargo.toml"
fi

BIN_PATH="$ROOT_DIR/target/release/samdef_detector"
if [ ! -x "$BIN_PATH" ]; then
    echo "Missing binary: $BIN_PATH" >&2
    exit 1
fi

mkdir -p "$LOGS_DIR" "$OUTPUT_DIR"
TS="$(date +"%Y%m%d_%H%M%S")"
TS_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
SAFE_TAG="$(printf '%s' "$TAG" | tr -c 'a-zA-Z0-9_-' '_')"
RUN_ID="run_${TS}_b${BATCH_SIZE}_p${PRODUCER_PARALLELISM}"
if [ -n "$SAFE_TAG" ]; then
    RUN_ID="${RUN_ID}_${SAFE_TAG}"
fi

LOG_FILE="$LOGS_DIR/${RUN_ID}.log"
SUMMARY_ARCHIVE="$LOGS_DIR/${RUN_ID}_summary.json"
RAW_SUMMARY_TMP="$(mktemp /tmp/samdef_summary_XXXXXX.json)"
GPU_CSV_TMP="$(mktemp /tmp/samdef_gpu_XXXXXX.csv)"
GPU_MON_PID=""

cleanup() {
    [ -n "$GPU_MON_PID" ] && kill "$GPU_MON_PID" 2>/dev/null || true
    rm -f "$RAW_SUMMARY_TMP" "$GPU_CSV_TMP"
}
trap cleanup EXIT

GPU_NAME="unknown"
if command -v nvidia-smi >/dev/null 2>&1; then
    GPU_NAME="$(nvidia-smi --query-gpu=name --format=csv,noheader | head -n 1 | xargs)"
    nvidia-smi --query-gpu=utilization.gpu,utilization.memory,memory.used --format=csv,noheader,nounits -lms 250 > "$GPU_CSV_TMP" &
    GPU_MON_PID=$!
fi

echo "Starting SAMDEF detector | run_id=$RUN_ID | batch=$BATCH_SIZE | parallel=$PRODUCER_PARALLELISM"

INPUT_DIR="$INPUT_DIR" \
OUTPUT_DIR="$OUTPUT_DIR" \
MODEL_PATH="$MODEL_PATH" \
BATCH_SIZE="$BATCH_SIZE" \
PRODUCER_PARALLELISM="$PRODUCER_PARALLELISM" \
BENCHMARK_SUMMARY_PATH="$RAW_SUMMARY_TMP" \
"$BIN_PATH" 2>&1 | tee "$LOG_FILE"

if [ -n "$GPU_MON_PID" ]; then
    kill "$GPU_MON_PID" 2>/dev/null || true
    wait "$GPU_MON_PID" 2>/dev/null || true
    GPU_MON_PID=""
fi

GIT_SHA="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo "unknown")"

python3 - "$RAW_SUMMARY_TMP" "$GPU_CSV_TMP" "$SUMMARY_ARCHIVE" "$LOGS_DIR/runs_history.csv" "$RUN_ID" "$TS_UTC" "$TAG" "$GIT_SHA" "$GPU_NAME" "$SUMMARY_OUT" "$GPU_OUT" << 'PY'
import csv, json, os, shutil, sys

(raw_path, csv_path, archive_path, history_path, run_id, ts_utc, tag, git_sha, gpu_name, summary_out, gpu_out) = sys.argv[1:12]

with open(raw_path, "r") as f:
    summary = json.load(f)

gpu_u, mem_u, mem_mb = [], [], []
if os.path.exists(csv_path):
    with open(csv_path, "r") as f:
        for row in csv.reader(f):
            if len(row) >= 3:
                try:
                    gpu_u.append(float(row[0].strip()))
                    mem_u.append(float(row[1].strip()))
                    mem_mb.append(float(row[2].strip()))
                except ValueError:
                    pass

gpu_metrics = {
    "gpu_name": gpu_name,
    "sample_interval_ms": 250,
    "samples_count": len(gpu_u),
    "mean_gpu_util_pct": round(sum(gpu_u) / len(gpu_u), 1) if gpu_u else 0.0,
    "max_gpu_util_pct": round(max(gpu_u), 1) if gpu_u else 0.0,
    "mean_mem_util_pct": round(sum(mem_u) / len(mem_u), 1) if mem_u else 0.0,
    "max_mem_util_pct": round(max(mem_u), 1) if mem_u else 0.0,
    "peak_vram_used_mib": int(max(mem_mb)) if mem_mb else 0,
}

if summary_out:
    os.makedirs(os.path.dirname(os.path.abspath(summary_out)), exist_ok=True)
    shutil.copyfile(raw_path, summary_out)

if gpu_out:
    os.makedirs(os.path.dirname(os.path.abspath(gpu_out)), exist_ok=True)
    with open(gpu_out, "w") as gf:
        json.dump(gpu_metrics, gf, indent=2)

archived = {
    "run_id": run_id,
    "timestamp_utc": ts_utc,
    "tag": tag,
    "git_sha": git_sha,
    **summary,
    "gpu_metrics": gpu_metrics,
}
with open(archive_path, "w") as af:
    json.dump(archived, af, indent=2)

header = [
    "run_id", "timestamp_utc", "tag", "git_sha", "model_file",
    "batch_size", "producer_parallelism", "total_images", "total_detections",
    "wall_clock_s", "mean_s_per_img", "img_per_s", "scenes_per_min",
    "mean_gpu_util_pct", "max_gpu_util_pct", "peak_vram_mib",
]
write_header = not os.path.exists(history_path) or os.path.getsize(history_path) == 0
with open(history_path, "a", newline="") as hf:
    writer = csv.writer(hf)
    if write_header:
        writer.writerow(header)
    writer.writerow([
        run_id, ts_utc, tag, git_sha, summary.get("model_file", ""),
        summary.get("batch_size", ""), summary.get("producer_parallelism", ""),
        summary.get("total_images", ""), summary.get("total_detections", ""),
        summary.get("total_wall_clock_seconds", ""), summary.get("mean_seconds_per_image", ""),
        summary.get("throughput_images_per_second", ""), summary.get("throughput_scenes_per_minute", ""),
        gpu_metrics["mean_gpu_util_pct"], gpu_metrics["max_gpu_util_pct"], gpu_metrics["peak_vram_used_mib"],
    ])

print(f"Run archive saved: {archive_path}")
print(f"History updated:   {history_path}")
print(f"Console log:       {archive_path.replace('_summary.json', '.log')}")
PY

