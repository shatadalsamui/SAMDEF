#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Locate python with sahi and onnxruntime-gpu
if [ -d "$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv/bin/python3"
elif [ -d "$SCRIPT_DIR/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/venv/bin/python3"
else
    PYTHON_BIN="python3"
fi

echo "========================================================================"
echo " Starting SAHI Baseline Benchmark"
echo " Python Binary: $PYTHON_BIN"
echo " Working Dir  : $SCRIPT_DIR"
echo "========================================================================"

"$PYTHON_BIN" src/benchmark_runner.py --config configs/benchmark_config.yaml

echo "Benchmark suite finished successfully."


