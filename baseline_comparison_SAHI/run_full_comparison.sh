#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PYTHON_BIN="$ROOT_DIR/apps_training/yolo_model_trainer/venv/bin/python3"
if [ ! -x "$PYTHON_BIN" ]; then
    PYTHON_BIN="python3"
fi

RESULTS_DIR="$SCRIPT_DIR/results"
DETECTOR_SCRIPT="$ROOT_DIR/apps_deploy/detector/run_detector.sh"
CONFIG_FILE="$SCRIPT_DIR/configs/benchmark_config.yaml"

BATCH_OVERRIDE=""
PARALLEL="4"
TAG="cmp"
NO_BUILD=0
LIMIT_ARG=""

while [ $# -gt 0 ]; do
    case "$1" in
        --batch|--batch-size) BATCH_OVERRIDE="$2"; shift 2 ;;
        --parallel) PARALLEL="$2"; shift 2 ;;
        --tag) TAG="$2"; shift 2 ;;
        --limit) LIMIT_ARG="$2"; shift 2 ;;
        --no-build) NO_BUILD=1; shift ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

read -r CFG_INPUT CFG_MODEL CFG_BATCH < <("$PYTHON_BIN" - "$CONFIG_FILE" "$SCRIPT_DIR" << 'PY'
import os, sys, yaml
cfg = yaml.safe_load(open(sys.argv[1]))
base = sys.argv[2]
print(cfg["dataset"]["input_dir"], os.path.abspath(os.path.join(base, cfg["model"]["path"])), cfg["slicing"].get("batch_size", 1))
PY
)
BATCH_SIZE="${BATCH_OVERRIDE:-$CFG_BATCH}"

SAMDEF_RAW_OUT="$(mktemp -d /tmp/samdef_bench_out_XXXXXX)"
trap 'rm -rf "$SAMDEF_RAW_OUT"' EXIT
mkdir -p "$RESULTS_DIR"

DET_ARGS=(
    --input "$CFG_INPUT"
    --output "$SAMDEF_RAW_OUT"
    --model "$CFG_MODEL"
    --batch "$BATCH_SIZE"
    --parallel "$PARALLEL"
    --tag "$TAG"
    --summary-out "$RESULTS_DIR/samdef_summary.json"
    --gpu-out "$RESULTS_DIR/samdef_gpu_metrics.json"
)
[ "$NO_BUILD" -eq 1 ] && DET_ARGS+=(--no-build)

echo "[1/2] Running SAMDEF detector + GPU telemetry (BATCH_SIZE=$BATCH_SIZE, PARALLEL=$PARALLEL)..."
"$DETECTOR_SCRIPT" "${DET_ARGS[@]}"

SAHI_ARGS=(
    --samdef-summary "$RESULTS_DIR/samdef_summary.json"
    --gpu-metrics "$RESULTS_DIR/samdef_gpu_metrics.json"
    --batch-size "$BATCH_SIZE"
    --tag "$TAG"
)
[ -n "$LIMIT_ARG" ] && SAHI_ARGS+=(--limit "$LIMIT_ARG")

echo "[2/2] Running SAHI baseline benchmark + comparison report..."
"$SCRIPT_DIR/run_benchmark.sh" "${SAHI_ARGS[@]}"
