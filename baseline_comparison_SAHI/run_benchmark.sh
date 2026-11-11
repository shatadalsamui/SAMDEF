#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -d "$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv/bin/python3"
elif [ -d "$SCRIPT_DIR/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/venv/bin/python3"
else
    PYTHON_BIN="python3"
fi

BATCH_STR=""
TAG_STR=""
PREV=""
for arg in "$@"; do
    case "$PREV" in
        --batch-size) BATCH_STR="$arg"; PREV=""; continue ;;
        --tag) TAG_STR="$arg"; PREV=""; continue ;;
    esac
    case "$arg" in
        --batch-size|--tag) PREV="$arg" ;;
    esac
done

if [ -z "$BATCH_STR" ]; then
    BATCH_STR="$("$PYTHON_BIN" -c 'import yaml; print(yaml.safe_load(open("configs/benchmark_config.yaml"))["slicing"].get("batch_size", 1))' 2>/dev/null || echo 1)"
fi

LOGS_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOGS_DIR"
TS="$(date +"%Y%m%d_%H%M%S")"
SAFE_TAG="$(printf '%s' "$TAG_STR" | tr -c 'a-zA-Z0-9_-' '_')"
RUN_ID="run_${TS}_b${BATCH_STR}"
if [ -n "$SAFE_TAG" ]; then
    RUN_ID="${RUN_ID}_${SAFE_TAG}"
fi

LOG_FILE="$LOGS_DIR/${RUN_ID}.log"
GPU_CSV_TMP="$(mktemp /tmp/sahi_gpu_XXXXXX.csv)"
GPU_MON_PID=""

cleanup() {
    [ -n "$GPU_MON_PID" ] && kill "$GPU_MON_PID" 2>/dev/null || true
    rm -f "$GPU_CSV_TMP"
}
trap cleanup EXIT

if command -v nvidia-smi >/dev/null 2>&1; then
    nvidia-smi --query-gpu=utilization.gpu,utilization.memory,memory.used --format=csv,noheader,nounits -lms 250 > "$GPU_CSV_TMP" &
    GPU_MON_PID=$!
fi

echo "Starting SAHI benchmark | run_id=$RUN_ID | python=$PYTHON_BIN"
"$PYTHON_BIN" src/benchmark_runner.py \
    --config configs/benchmark_config.yaml \
    --run-id "$RUN_ID" \
    --sahi-gpu-csv "$GPU_CSV_TMP" \
    "$@" 2>&1 | tee "$LOG_FILE"

if [ -n "$GPU_MON_PID" ]; then
    kill "$GPU_MON_PID" 2>/dev/null || true
    wait "$GPU_MON_PID" 2>/dev/null || true
    GPU_MON_PID=""
fi

echo "Console log saved:   $LOG_FILE"
